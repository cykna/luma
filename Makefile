build-wasm:
	cargo build --target wasm32-unknown-unknown --release
	wasm-bindgen target/wasm32-unknown-unknown/release/luma.wasm --out-dir pkg --target web

run-wasm:
	cargo build --target wasm32-unknown-unknown --release
	wasm-bindgen target/wasm32-unknown-unknown/release/luma.wasm --out-dir lumajs --target web
	echo "<!DOCTYPE html><html lang="en"><head><meta charset="UTF-8"><title>Luma WASM Demo</title></head><body><h1>Luma WASM Demo</h1><canvas id=\"canvas\"></canvas><script type=\"module\">import init from './luma.js';async function main() {await init().catch(e => {if(e.message.startsWith('Using exceptions')){}else throw e;});console.log('WASM loaded!');}main();</script></body></html>" > lumajs/index.html
	cd lumajs && python -m http.server 8080
