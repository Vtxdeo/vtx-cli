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
vtx_version = 1

[project]
name = "my-rust-plugin"
version = "0.1.0"
language = "rust"
authors = [{ name = "Your Name", email = "you@example.com" }]
description = "Short plugin summary"
license = "MIT"
homepage = "https://example.com"
repository = "https://example.com/repo"
keywords = ["vtx", "plugin"]
```

Notes:
- `[build]` is optional and only needed to override the default Rust backend.
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
vtx_version = 1

[project]
name = "my-ts-plugin"
version = "0.1.0"
language = "ts"
authors = [{ name = "Your Name", email = "you@example.com" }]
description = "Short plugin summary"
license = "MIT"
homepage = "https://example.com"
repository = "https://example.com/repo"
keywords = ["vtx", "plugin"]
```

Notes:
- `[build]` is optional and only needed to override the default TS backend.
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
vtx_version = 1

[project]
name = "my-py-plugin"
version = "0.1.0"
language = "python"
authors = [{ name = "Your Name", email = "you@example.com" }]
description = "Short plugin summary"
license = "MIT"
homepage = "https://example.com"
repository = "https://example.com/repo"
keywords = ["vtx", "plugin"]
```

Notes:
- `[build]` is optional and only needed to override the default Python backend.
 - `vtx init --lang python --name my-py-plugin` generates this layout.
