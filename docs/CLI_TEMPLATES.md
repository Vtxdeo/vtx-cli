# VTX CLI Templates (Rust/TS/Python)

This document provides example project structures and `vtx.toml` templates
for Rust, TypeScript, and Python plugins.

## Rust

Example structure:

```text
my-rust-plugin/
  Cargo.toml
  src/
    lib.rs
  vtx.toml
```

`vtx.toml`:

```toml
[project]
name = "my-rust-plugin"
language = "rust"

[build]
cmd = "cargo build --target wasm32-wasip1 --release"
output_dir = "target/wasm32-wasip1/release"
artifact = "my_rust_plugin.wasm"

[sdk]
version = "0.1.8"
```

Notes:
- `artifact` uses Rust crate naming (`-` becomes `_`) for the wasm output.
- If you omit `[build]`, the CLI will use the Rust backend defaults.
 - `vtx init --lang rust --name my-rust-plugin` generates this layout.

## TypeScript

Example structure:

```text
my-ts-plugin/
  package.json
  src/
    index.ts
  dist/
    my-ts-plugin.wasm
  vtx.toml
```

`vtx.toml`:

```toml
[project]
name = "my-ts-plugin"
language = "ts"

[build]
cmd = "npm run build"
output_dir = "dist"
artifact = "my-ts-plugin.wasm"

[sdk]
version = "0.2.0"
```

Notes:
- `build.cmd` is required if your build toolchain is non-standard.
- The CLI will use `output_dir` + `artifact` for exact output resolution.
 - `vtx init --lang ts --name my-ts-plugin` generates this layout.

## Python

Example structure:

```text
my-py-plugin/
  pyproject.toml
  src/
    my_py_plugin/
      __init__.py
  dist/
    my-py-plugin.wasm
  vtx.toml
```

`vtx.toml`:

```toml
[project]
name = "my-py-plugin"
language = "python"

[build]
cmd = "componentize-py -d . -o dist/my-py-plugin.wasm my_py_plugin"
output_dir = "dist"
artifact = "my-py-plugin.wasm"

[sdk]
version = "0.2.0"
```

Notes:
- If `build.cmd` is omitted, the CLI defaults to `componentize-py`.
- `artifact` should match the generated wasm filename.
 - `vtx init --lang python --name my-py-plugin` generates this layout.
