# Installation

## Recommended (System Global)
The easiest way to get the standalone binary.
```bash
curl -fsSL [https://raw.githubusercontent.com/.../install.sh](https://raw.githubusercontent.com/.../install.sh) | sh
# OR
brew install vtxdeo/tap/vtx

```

## Developer Friendly (Using your existing tools)

Already have a language environment? You can install `vtx` using your preferred package manager.

### Node.js

```bash
npm install -g @vtxdeo/cli
# Or run once without installing:
npx @vtxdeo/cli init my-project

```

### Python

We recommend using [pipx](https://www.google.com/search?q=https://pypa.github.io/pipx/) to install CLI tools in isolation.

```bash
pipx install vtx-cli

```

### Rust

```bash
# Installs from source (compilation required)
cargo install vtx-cli

```
