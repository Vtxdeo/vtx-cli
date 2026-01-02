# @vtx/cli

Install the VTX CLI with npm and forward all commands to the bundled Rust binary.

## Install

```sh
npm install -g @vtx/cli
```

## Usage

```sh
vtx --help
```

## Environment

- `VTX_CLI_VERSION` or `VERSION`: install a specific tag (default: latest)
- `VTX_CLI_REPO` or `REPO`: override GitHub repo (default: vtxdeo/vtx-cli)
- `GITHUB_TOKEN`: GitHub token to avoid API rate limits
