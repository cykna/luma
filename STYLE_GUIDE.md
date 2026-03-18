# Style Guide

Este documento estabelece as convenções e padrões de código para o projeto Luma.

## 1. Documentação

### 1.1 Palavras-chave RFC 2119

A documentação **DEVE** utilizar as palavras-chave definidas na [RFC 2119](https://datatracker.ietf.org/doc/html/rfc2119) para indicar requisitos obrigatórios, recomendados e opcionais.

- **MUST** / **REQUIRED** / **SHALL**: Requisito obrigatório. O implementador DEVE atender para conformidade.
- **SHOULD** / **RECOMMENDED**: Recomendação forte. Deve ser implementada exceto se houver justificativa técnica.
- **MAY** / **OPTIONAL**: Funcionalidade opcional. A implementação pode decidir incluir ou não.

```rust
/// Cria um novo componente de transformação.
///
/// **MUST** ser chamado antes de qualquer operação de renderização.
/// **SHOULD** ser inicializado com a identidade como valor padrão.
/// **MAY** aceita valores personalizados para posição, rotação e escala.
///
/// # Arguments
/// * `position` - Posição no espaço 3D (padrão: [0, 0, 0])
/// * `rotation` - Rotação em ângulos Euler (padrão: [0, 0, 0])
/// * `scale` - Fator de escala (padrão: [1, 1, 1])
///
/// # Example
/// ```rust
/// let transform = Transform::default(); // identidade
/// let custom = Transform::new([1.0, 2.0, 0.0], [0.0, 0.0, 45.0], [2.0, 2.0, 1.0]);
/// ```
pub fn new(position: [f32; 3], rotation: [f32; 3], scale: [f32; 3]) -> Self;
```

### 1.2 Estrutura de Doc Comments

**MUST** seguir esta ordem para documentação de módulos, structs, traits e funções:

```rust
/// [Breve descrição duma linha]
///
/// [Descrição detalhada - propósito, contexto, comportamentos importantes]
///
/// # Arguments
/// * `nome_arg` - [descrição]
///
/// # Returns
/// [O que a função retorna]
///
/// # Errors
/// [Quando pode retornar erro]
///
/// # Example
/// ```rust
/// // código que compila
/// ```
///
/// # Notes
/// [Informações adicionais, caveats, considerações de performance]
```

## 2. ECS para 3D

O módulo 3D utiliza **Entity-Component-System (ECS)** com referência ao estilo three.js, adaptado ao modelo de ownership do Rust.

### 2.1 Componentes

Componentes **MUST** ser structs de dados puras, sem lógica de negócio:

```rust
/// Componente de transformação 3D.
///
/// **MUST** implementar `Default` para valores identidade.
/// **SHOULD** ser serializável para suporte a save/load.
#[derive(Clone, Debug, Default)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],  // Euler em graus
    pub scale:    [f32; 3],
}

/// Componente visual que referencia um recurso de malha.
#[derive(Clone, Debug)]
pub struct Mesh {
    pub mesh_id: MeshId,
}

/// Componente de material com parâmetros.
#[derive(Clone, Debug)]
pub struct Material {
    pub color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
}
```

### 2.2 Entidades e Builder API

Entidades **MUST** utilizar **Builder Pattern** estilo three.js:

```rust
/// Builder para criar entidades 3D com fluent API.
pub struct EntityBuilder<'a> {
    world: &'a mut World,
    entity: Entity,
}

impl<'a> EntityBuilder<'a> {
    /// Adiciona componente de transformação.
    pub fn with_transform(mut self, transform: Transform) -> Self {
        self.world.insert(self.entity, transform);
        self
    }

    /// Adiciona componente de malha.
    pub fn with_mesh(mut self, mesh_id: MeshId) -> Self {
        self.world.insert(self.entity, Mesh { mesh_id });
        self
    }

    /// Adiciona componente de material.
    pub fn with_material(mut self, material: Material) -> Self {
        self.world.insert(self.entity, material);
        self
    }

    /// Define nome para debug.
    pub fn name(mut self, name: &'a str) -> Self {
        self.world.insert(self.entity, Name(name.to_string()));
        self
    }

    /// Finaliza e retorna a entidade criada.
    pub fn build(self) -> Entity {
        self.entity
    }
}

/// Exemplo de uso:
/// ```rust
/// let cube = world.spawn()
///     .name("cube")
///     .with_transform(Transform::default())
///     .with_mesh(mesh_id)
///     .with_material(Material {
///         color: [1.0, 0.0, 0.0, 1.0],
///         metallic: 0.0,
///         roughness: 0.5,
///     })
///     .build();
/// ```
```

### 2.3 Sistemas

Sistemas **MUST** ser funções ou traits com lógica de processamento:

```rust
/// Sistema de transformação hierárquica.
///
/// **SHOULD** executar antes do sistema de renderização.
/// **MUST** processar pais antes de filhos.
pub fn transform_system(world: &World) {
    let mut query = world.query::<(&Parent, &Transform, &mut GlobalTransform)>();
    // Processa transformações em ordem hierárquica
}

/// Sistema de renderização 3D.
///
/// **MUST** ser chamado após `transform_system`.
pub fn render_system(world: &World, renderer: &mut LumaBackend) {
    // Query apenas entidades visíveis
    let query = world.query::<(&Mesh, &GlobalTransform, &Material)>();
    // Renderiza cada entidade
}

/// Trait para sistemas customizáveis.
/// **MAY** ser implementada para sistemas com estado.
pub trait System {
    fn run(&mut self, world: &World);
    fn name(&self) -> &'static str;
}
```

### 2.4 Armazenamento

**MUST** utilizar `slotmap` para Entity IDs (já incluso nas dependências):

```rust
use slotmap::{Entity, EntityBuilder as SlotmapBuilder, SlotMap};

// Armazenamento principal de entidades
pub struct World {
    entities: SlotMap<Entity, ()>,
    transforms: ComponentStorage<Transform>,
    meshes: ComponentStorage<Mesh>,
    materials: ComponentStorage<Material>,
    names: ComponentStorage<Name>,
}
```

## 3. Estilo Ratatui para 2D

A abstração 2D sobre vello **MUST** seguir API fluente estilo Ratatui com widgets composáveis.

### 3.1 API Fluente

```rust
/// Builder para cenas 2D com API fluente.
///
/// **MUST** permitir chaining de operações.
/// **SHOULD** usar métodos com nomes descritivos.
pub struct SceneBuilder {
    scene: Scene,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self { scene: Scene::new() }
    }

    /// Adiciona um retângulo填充.
    pub fn fill_rect(mut self, rect: Rect, color: Color) -> Self {
        self.scene.fill(
            vello::peniko::Brush::Solid(color.into()),
            Some(&vello::kurbo::Rect::new(
                rect.x as f64,
                rect.y as f64,
                (rect.x + rect.w) as f64,
                (rect.y + rect.h) as f64,
            )),
        );
        self
    }

    /// Adiciona texto na posição especificada.
    pub fn text(mut self, pos: Point, text: &str, style: TextStyle) -> Self {
        // Adiciona texto à cena
        self
    }

    /// Adiciona uma linha.
    pub fn line(mut self, from: Point, to: Point, stroke: Stroke) -> Self {
        self.scene.stroke(
            vello::peniko::Brush::Solid(stroke.color.into()),
            Some(&vello::kurbo::Shape::from_pod(vello::kurbo::Line::new(
                (from.x, from.y),
                (to.x, to.y),
            ))),
            stroke.width,
        );
        self
    }

    /// Adiciona um widget composed.
    pub fn widget(mut self, widget: impl Widget) -> Self {
        widget.render(&mut self.scene);
        self
    }

    /// Finaliza e retorna a cena.
    pub fn build(self) -> Scene {
        self.scene
    }
}

/// Trait para widgets composáveis.
/// **MUST** ser implementado por qualquer widget reutilizável.
pub trait Widget {
    fn render(&self, scene: &mut Scene);
}
```

### 3.2 Componentes Composáveis

```rust
/// Botão 2D com estados.
///
/// **SHOULD** suportar estados: default, hover, active, disabled.
/// **MAY** suportar callbacks de eventos.
#[derive(Clone)]
pub struct Button {
    rect: Rect,
    text: String,
    style: ButtonStyle,
}

#[derive(Clone, Default)]
pub struct ButtonStyle {
    pub background: Color,
    pub foreground: Color,
    pub border_color: Color,
    pub border_radius: f32,
}

impl Widget for Button {
    fn render(&self, scene: &mut Scene) {
        // Renderiza o botão
    }
}

/// Container com filhos posicionados.
///
/// **SHOULD** suportar diferentes layouts: stack, flex, grid.
pub struct Container {
    rect: Rect,
    layout: Layout,
    children: Vec<Box<dyn Widget>>,
}

impl Container {
    /// Cria container com layout vertical.
    pub fn vertical(rect: Rect) -> Self {
        Self {
            rect,
            layout: Layout::Vertical,
            children: Vec::new(),
        }
    }

    /// Adiciona filho ao container.
    pub fn child(mut self, widget: impl Widget + 'static) -> Self {
        self.children.push(Box::new(widget));
        self
    }

    /// Define espaçamento entre filhos.
    pub fn spacing(mut self, spacing: u32) -> Self {
        self
    }
}

impl Widget for Container {
    fn render(&self, scene: &mut Scene) {
        // Layout e renderização recursiva
    }
}

/// Exemplo de uso composed:
/// ```rust
/// let scene = SceneBuilder::new()
///     .fill_rect(screen_rect, Color::rgb(20, 20, 30))
///     .widget(
///         Container::vertical(Rect::new(10, 10, 200, 300))
///             .spacing(8)
///             .child(Button::new("Click me").on_click(|| { /* ... */ }))
///             .child(Label::new("Status: OK").color(Color::green()))
///             .child(
///                 Container::horizontal(Rect::new(0, 0, 180, 50))
///                     .child(Icon::new("settings"))
///                     .child(Text::new("Settings"))
///             )
///     )
///     .build();
/// ```
```

### 3.3 Tipos Essenciais

```rust
#[derive(Clone, Copy, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Clone)]
pub struct Stroke {
    pub color: Color,
    pub width: f32,
}

#[derive(Clone)]
pub struct TextStyle {
    pub color: Color,
    pub size: f32,
    pub font: Option<String>,
}
```

## 4. Convenções de Código

### 4.1 Nomenclatura

| Tipo | Convenção | Exemplo |
|------|------------|---------|
| Módulos | `snake_case` | `space/mod.rs`, `backend/mod.rs` |
| Structs/Enums | `PascalCase` | `LumaBackend`, `LumaEvent` |
| Funções | `snake_case` | `render_ui()`, `create_surface_texture()` |
| Variáveis | `snake_case` | `entity_id`, `mesh_handle` |
| Constantes | `SCREAMING_SNAKE_CASE` | `MAX_VERTS`, `DEFAULT_SIZE` |
| Traits | `PascalCase` | `LumaHandler`, `Widget` |
| Types públicos | `PascalCase` | `Result`, `Report` |

### 4.2 Estrutura de Módulos

**MUST** seguir esta estrutura:

```rust
// 1. Imports externos
use vello::Scene;
use winit::window::Window;

// 2. Imports internos
use crate::{backend::LumaBackend, Result};

// 3. Submódulos
mod config;
mod context;

// 4. Re-exports públicos
pub use config::*;
pub use context::*;

// 5. Código principal
pub struct LumaSpace<H> { ... }
```

### 4.3 Error Handling

**MUST** usar as aliases de Result definidas em `main.rs`:

```rust
#[cfg(target_arch = "wasm32")]
pub type Result<T> = anyhow::Result<T>;
#[cfg(target_arch = "wasm32")]
pub type Report = anyhow::Error;

#[cfg(not(target_arch = "wasm32"))]
pub type Result<T> = color_eyre::eyre::Result<T>;
#[cfg(not(target_arch = "wasm32"))]
pub type Report = color_eyre::Report;
```

**SHOULD** retornar erros contextuais:

```rust
pub fn new(window: Arc<Window>) -> Result<Self> {
    let surface = instance.create_surface(window)
        .context("Failed to create winit surface")?;
    // ...
}
```

### 4.4 Trait Handler Pattern

**MUST** implementar o padrão `LumaHandler` para separação de eventos:

```rust
/// Trait para manipular eventos e renderização.
///
/// **MUST** ser implementado pelo usuário da biblioteca.
/// **SHOULD** manter estado mínimo no handler.
pub trait LumaHandler {
    /// Configurações de janela.
    fn configs() -> LumaWindowConfigs {
        LumaWindowConfigs::default()
    }

    /// Chamado quando UI precisa ser re-renderizada.
    fn rerender(&mut self, ui: &LumaUI, renderer: &mut LumaBackend);

    /// Called when a window event is received.
    fn on_event(
        &mut self,
        event: LumaEvent,
        window: &Window,
        ui: &LumaUI,
        renderer: &mut LumaBackend,
    );
}
```

### 4.5 Formatação

- **MUST** usar `rustfmt` padrão (4 espaços, 100 colunas)
- **MUST** usar `#[inline]` para funções pequenas frequentemente chamadas
- **SHOULD** usar `?` ao invés de `match` para Results simples
- **MAY** usar `todo!()` ou `unimplemented!()` para código pendente com justificativa

### 4.6 Logging

**MUST** usar `tracing` para logging:

```rust
tracing::info!("Initialized Luma");
tracing::warn!("Resized to {}x{}", width, height);
tracing::error!("Failed to render: {:?}", error);
```

### 4.7 Atributos de Plataforma

**MUST** usar `#[cfg(target_arch = "wasm32")]` para código específico de plataforma:

```rust
#[cfg(not(target_arch = "wasm32"))]
fn init_logging() { ... }

#[cfg(target_arch = "wasm32")]
fn init_logging() { ... }

#[cfg(not(target_arch = "wasm32"))]
pub fn initialize(&mut self) { ... }

#[cfg(target_arch = "wasm32")]
pub fn initialize(&mut self) { ... }
```

## 5. Referências

- [RFC 2119 - Key words for use in RFCs](https://datatracker.ietf.org/doc/html/rfc2119)
- [three.js](https://threejs.org/) - API de referência para 3D
- [Ratatui](https://ratatui.rs/) - API de referência para widgets composáveis
- [Bevy ECS](https://bevyengine.org/) - Referência para ECS em Rust
