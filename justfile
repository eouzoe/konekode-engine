check:
    cargo fmt --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    cargo build --workspace --target wasm32-unknown-unknown
    just proofs

proofs:
    cd proofs && lake exe cache get && lake build

wasm:
    cargo build --release --target wasm32-unknown-unknown -p konekode-wasm
    wasm-bindgen --target web --out-dir pkg target/wasm32-unknown-unknown/release/konekode_wasm.wasm

version:
    #!/usr/bin/env bash
    set -euo pipefail
    id=$(cargo pkgid -p konekode-wasm)
    echo "${id##*@}"

package: wasm
    #!/usr/bin/env bash
    set -euo pipefail
    version=$(just version)
    stage=$(mktemp -d)
    trap 'rm -rf "$stage"' EXIT
    mkdir "$stage/package"
    cp pkg/konekode_wasm.js pkg/konekode_wasm.d.ts pkg/konekode_wasm_bg.wasm pkg/konekode_wasm_bg.wasm.d.ts \
      LICENSE-APACHE LICENSE-MIT "$stage/package/"
    cat > "$stage/package/package.json" <<EOF
    {
      "name": "konekode-engine",
      "version": "$version",
      "description": "The Konekode learning engine for the browser",
      "license": "MIT OR Apache-2.0",
      "repository": { "type": "git", "url": "git+https://github.com/eouzoe/konekode-engine.git" },
      "type": "module",
      "main": "./konekode_wasm.js",
      "types": "./konekode_wasm.d.ts",
      "exports": {
        ".": { "types": "./konekode_wasm.d.ts", "default": "./konekode_wasm.js" },
        "./konekode_wasm_bg.wasm": "./konekode_wasm_bg.wasm"
      },
      "files": [
        "konekode_wasm.js",
        "konekode_wasm.d.ts",
        "konekode_wasm_bg.wasm",
        "konekode_wasm_bg.wasm.d.ts",
        "LICENSE-APACHE",
        "LICENSE-MIT"
      ],
      "sideEffects": false
    }
    EOF
    mkdir -p dist
    tar --sort=name --mtime=@0 --owner=0 --group=0 --numeric-owner -C "$stage" -cf - package \
      | gzip -n -9 > "dist/konekode-engine-$version.tgz"
    echo "dist/konekode-engine-$version.tgz"

cov:
    PATH="$NIGHTLY/bin:$PATH" cargo llvm-cov --workspace --branch --summary-only

mutants crate *filter:
    cargo mutants --package {{crate}} --test-package {{crate}} --test-package konekode-probe {{filter}}

fuzz target seconds="60":
    cd fuzz && PATH="$NIGHTLY/bin:$PATH" cargo fuzz run {{target}} -- -max_total_time={{seconds}}
