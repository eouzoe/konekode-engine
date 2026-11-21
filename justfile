check:
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo build --workspace --target wasm32-unknown-unknown

wasm:
    cargo build --release --target wasm32-unknown-unknown -p konekode-wasm
    wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/konekode_wasm.wasm

cov:
    PATH="$NIGHTLY/bin:$PATH" cargo llvm-cov --workspace --branch --summary-only

mutants crate:
    cargo mutants --package {{crate}}

fuzz target seconds="60":
    cd fuzz && PATH="$NIGHTLY/bin:$PATH" cargo fuzz run {{target}} -- -max_total_time={{seconds}}
