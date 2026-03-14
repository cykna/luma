build-wasm:
	cargo build --target wasm32-unknown-unknown --release
	wasm-bindgen target/wasm32-unknown-unknown/release/luma.wasm --out-dir pkg --target web

define INDEX_HTML_CONTENT
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>Luma WASM Demo</title>
    <style>
        body, html { 
            margin: 0; padding: 0; width: 100%; height: 100%; 
            overflow: hidden; background: #fff; 
        }
        canvas { 
            display: block; width: 100vw; height: 100vh; 
        }
    </style>
</head>
<body>
    <canvas id="canvas"></canvas>
    <script type="module">
        import init from './luma.js';
        async function main() {
            try {
                await init();
                console.log('Luma WASM Loaded');
            } catch (e) {
                if (!e.message.startsWith('Using exceptions')) throw e;
            }
        }
        main();
    </script>
</body>
</html>
endef
export INDEX_HTML_CONTENT

run-wasm:
	cargo build --target wasm32-unknown-unknown --release
	wasm-bindgen target/wasm32-unknown-unknown/release/luma.wasm --out-dir lumajs --target web
	echo "$$INDEX_HTML_CONTENT" > lumajs/index.html
	cd lumajs && python -m http.server 8080
