set -e

cargo build --target wasm32-unknown-unknown --release --bin nextnext
wasm-bindgen --out-name nextnext   --out-dir build   --target web target/wasm32-unknown-unknown/release/nextnext.wasm
python3 -m http.server --directory build
