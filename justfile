check:
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo build --workspace --target wasm32-unknown-unknown

wasm:
    cargo build --release --target wasm32-unknown-unknown -p konekode-wasm
    wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/konekode_wasm.wasm
