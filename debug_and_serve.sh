set -e

cargo build --target wasm32-unknown-unknown --bin nextnext
wasm-bindgen --out-name nextnext   --out-dir build   --target web target/wasm32-unknown-unknown/debug/nextnext.wasm
python3 -m http.server --directory build
