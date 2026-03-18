use proc_macro::TokenStream;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{
    Block, Expr, FnArg, Ident, ItemFn, ItemStruct, ReturnType, Stmt, Token, Type,
    parse::{Parse, ParseStream},
    parse_macro_input, token,
};

struct ShaderBlock {
    name: Ident,
    _brace_token: token::Brace,
    items: Vec<ShaderItem>,
}

enum ShaderItem {
    Struct(ItemStruct),
    Fn(ItemFn),
}

impl Parse for ShaderBlock {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let content;
        let _brace_token = syn::braced!(content in input);
        let mut items = Vec::new();
        while !content.is_empty() {
            if content.peek(Token![struct]) {
                items.push(ShaderItem::Struct(content.parse()?));
            } else if content.peek(Token![fn]) || content.peek(Token![#]) {
                items.push(ShaderItem::Fn(content.parse()?));
            } else {
                return Err(content.error("expected struct or fn"));
            }
        }
        Ok(ShaderBlock {
            name,
            _brace_token,
            items,
        })
    }
}

#[proc_macro]
pub fn shader(input: TokenStream) -> TokenStream {
    let block = parse_macro_input!(input as ShaderBlock);

    let shader_name = &block.name;
    let mut wgsl_code = String::new();
    let mut rust_items = Vec::new();
    let mut vertex_ty = quote! { () };
    let mut result_ty = quote! { () };

    let mut struct_fields = HashMap::new();

    for item in &block.items {
        if let ShaderItem::Struct(s) = item {
            let fields: Vec<Ident> = s
                .fields
                .iter()
                .map(|f| f.ident.clone().expect("Only named fields supported"))
                .collect();
            struct_fields.insert(s.ident.to_string(), fields);

            if s.ident == "Vertex" {
                let rust_name = format_ident!("{}{}", shader_name, s.ident);
                vertex_ty = quote! { #rust_name };
            } else if s.ident == "VertexResult" {
                let rust_name = format_ident!("{}{}", shader_name, s.ident);
                result_ty = quote! { #rust_name };
            }
        }
    }

    for item in &block.items {
        match item {
            ShaderItem::Struct(s) => {
                let s_name = &s.ident;
                let rust_struct_name = format_ident!("{}{}", shader_name, s_name);

                wgsl_code.push_str(&format!("struct {} {{\n", s_name));
                let mut rust_fields = Vec::new();
                for field in &s.fields {
                    let f_name = &field.ident;
                    let f_ty = &field.ty;

                    for attr in &field.attrs {
                        if attr.path().is_ident("builtin") {
                            let meta: Ident = attr.parse_args().expect("expected ident in builtin");
                            wgsl_code.push_str(&format!("  @builtin({}) ", meta));
                        }
                    }

                    let (wgsl_ty, rust_ty) = translate_type_dual(f_ty);
                    wgsl_code.push_str(&format!("{}: {};\n", f_name.as_ref().unwrap(), wgsl_ty));

                    rust_fields.push(quote! { pub #f_name: #rust_ty });
                }
                wgsl_code.push_str("};\n");

                rust_items.push(quote! {
                    #[derive(Debug, Clone, Copy, ::bytemuck::Pod, ::bytemuck::Zeroable)]
                    #[repr(C)]
                    pub struct #rust_struct_name {
                        #(#rust_fields,)*
                    }
                });
            }
            ShaderItem::Fn(f) => {
                let f_name = &f.sig.ident;

                for attr in &f.attrs {
                    if attr.path().is_ident("vertex") {
                        wgsl_code.push_str("@vertex ");
                    } else if attr.path().is_ident("fragment") {
                        wgsl_code.push_str("@fragment ");
                    }
                }

                let args: Vec<String> = f
                    .sig
                    .inputs
                    .iter()
                    .map(|arg| {
                        if let FnArg::Typed(pat) = arg {
                            let pat_ident = &pat.pat;
                            let name = quote!(#pat_ident).to_string();
                            let (wgsl_ty, _) = translate_type_dual(&pat.ty);
                            format!("{}: {}", name, wgsl_ty)
                        } else {
                            String::new()
                        }
                    })
                    .collect();

                let ret_ty = match &f.sig.output {
                    ReturnType::Default => String::new(),
                    ReturnType::Type(_, ty) => {
                        let (wgsl_ty, _) = translate_type_dual(ty.as_ref());
                        if f.attrs.iter().any(|a| a.path().is_ident("fragment"))
                            && wgsl_ty == "vec4<f32>"
                        {
                            format!("-> @location(0) {}", wgsl_ty)
                        } else {
                            format!("-> {}", wgsl_ty)
                        }
                    }
                };

                wgsl_code.push_str(&format!("fn {}({}) {} ", f_name, args.join(", "), ret_ty));

                let body = translate_block(&f.block, &struct_fields);
                wgsl_code.push_str(&body);
                wgsl_code.push_str("\n\n");
            }
        }
    }

    let shader_name_str = shader_name.to_string();
    let expanded = quote! {
        pub struct #shader_name;

        #(#rust_items)*

        impl LumaShader for #shader_name {
            type Vertex = #vertex_ty;
            type Result = #result_ty;
            const WGSL: &'static str = #wgsl_code;
            const SHADER_NAME: &'static str = #shader_name_str;
        }
    };

    TokenStream::from(expanded)
}

fn translate_type_dual(ty: &Type) -> (String, proc_macro2::TokenStream) {
    let type_str = quote!(#ty).to_string().replace(" ", "");
    match type_str.as_str() {
        "Vec2<f32>" => ("vec2<f32>".to_string(), quote! { [f32; 2] }),
        "Vec3<f32>" => ("vec3<f32>".to_string(), quote! { [f32; 3] }),
        "Vec4<f32>" => ("vec4<f32>".to_string(), quote! { [f32; 4] }),
        "f32" => ("f32".to_string(), quote! { f32 }),
        _ => (type_str, quote! { #ty }),
    }
}

fn translate_block(block: &Block, struct_fields: &HashMap<String, Vec<Ident>>) -> String {
    let mut wgsl_block = String::new();
    wgsl_block.push_str("{\n");

    for (i, stmt) in block.stmts.iter().enumerate() {
        let is_last = i == block.stmts.len() - 1;
        match stmt {
            Stmt::Expr(expr, None) if is_last => {
                let expr_str = translate_expr(expr, struct_fields);
                wgsl_block.push_str(&format!("  return {};\n", expr_str));
            }
            Stmt::Expr(expr, semi) => {
                let expr_str = translate_expr(expr, struct_fields);
                wgsl_block.push_str(&format!(
                    "  {}{}\n",
                    expr_str,
                    if semi.is_some() { ";" } else { "" }
                ));
            }
            Stmt::Local(local) => {
                let pat = &local.pat;
                let init = local
                    .init
                    .as_ref()
                    .map(|i| translate_expr(&i.expr, struct_fields))
                    .unwrap_or_default();
                wgsl_block.push_str(&format!("  var {} = {};\n", quote!(#pat), init));
            }
            _ => {
                let mut s = quote!(#stmt).to_string();
                fix_symbols(&mut s);
                wgsl_block.push_str(&format!("  {}\n", s));
            }
        }
    }

    wgsl_block.push_str("}");
    wgsl_block
}

fn translate_expr(expr: &Expr, struct_fields: &HashMap<String, Vec<Ident>>) -> String {
    match expr {
        Expr::Struct(s) => {
            let struct_name = s.path.segments.last().unwrap().ident.to_string();
            if let Some(fields) = struct_fields.get(&struct_name) {
                let mut field_map = HashMap::new();
                for f in &s.fields {
                    if let syn::Member::Named(ident) = &f.member {
                        field_map.insert(ident.to_string(), translate_expr(&f.expr, struct_fields));
                    }
                }
                let ordered_args: Vec<String> = fields
                    .iter()
                    .map(|f| {
                        field_map
                            .get(&f.to_string())
                            .cloned()
                            .unwrap_or_else(|| "0.0".to_string())
                    })
                    .collect();
                format!("{}({})", struct_name, ordered_args.join(", "))
            } else {
                let mut s = quote!(#expr).to_string();
                fix_symbols(&mut s);
                s
            }
        }
        Expr::Call(call) => {
            let func_expr = &*call.func;
            let mut func_name = quote!(#func_expr).to_string().replace(" ", "");
            fix_symbols(&mut func_name);
            let args: Vec<String> = call
                .args
                .iter()
                .map(|a| translate_expr(a, struct_fields))
                .collect();
            format!("{}({})", func_name, args.join(", "))
        }
        Expr::Field(field) => {
            let base = translate_expr(&field.base, struct_fields);
            let member = match &field.member {
                syn::Member::Named(ident) => ident.to_string(),
                syn::Member::Unnamed(index) => index.index.to_string(),
            };
            format!("{}.{}", base, member)
        }
        Expr::Path(path) => {
            let mut s = quote!(#path).to_string().replace(" ", "");
            fix_symbols(&mut s);
            s
        }
        _ => {
            let mut s = quote!(#expr).to_string();
            fix_symbols(&mut s);
            s
        }
    }
}

fn fix_symbols(s: &mut String) {
    *s = s.replace("Vec4", "vec4<f32>");
    *s = s.replace("Vec3", "vec3<f32>");
    *s = s.replace("Vec2", "vec2<f32>");
}
