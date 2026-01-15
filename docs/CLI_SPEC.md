# VTX CLI Spec (Draft)

This document defines the scope, configuration, and conventions for the
language-agnostic VTX CLI. The CLI is implemented in Rust, but it must build
plugins for all SDK-supported languages.

## Goals

- Provide one unified build entrypoint for all plugin languages.
- Keep SDKs language-specific and focused on runtime APIs.
- Make builds reproducible, CI-friendly, and easy to extend.

## Non-Goals

- The CLI does not define SDK APIs or runtime bindings.
- The CLI does not embed language SDKs; it only orchestrates builds.

## CLI Responsibilities

- Project discovery and config parsing (`vtx.toml`).
- Toolchain checks and dependency validation.
- Build orchestration by language backend.
- Output artifact discovery.
- Packaging: Wasm -> Component -> `.vtx`.
- Contract validation.

## SDK Responsibilities

- Provide language-specific APIs and runtime bindings.
- Provide optional metadata needed by the CLI (version, compatibility).

## Project Config (`vtx.toml`)

Minimum required fields:

```toml
vtx_version = 1

[project]
name = "my-plugin"
version = "0.1.0"
language = "rust" # rust|go|ts|python|php|lua|...
```

`vtx_version` is required to identify the config schema.
`[build]` is optional and only needed when you want to override the default build behavior.
The CLI injects `.vtx` metadata from `vtx.toml`; SDK version is auto-detected and does not need to be declared.

### Sections

- top-level
  - `vtx_version` (number, required): config schema version.
- `[project]`
  - `name` (string, required): plugin package name.
  - `version` (string, required): plugin version declared by the author.
  - `language` (string, required): build backend selector.
  - `author` (string, optional): legacy author field, kept for compatibility.
  - `authors` (array, optional): PEP 621-style author list.
  - `description` (string, optional): short summary.
  - `license` (string, optional): license identifier.
  - `homepage` (string, optional): project homepage URL.
  - `repository` (string, optional): source repository URL.
  - `keywords` (array, optional): search keywords.
- `[build]`
  - `cmd` (string, optional): full custom build command.
  - `output_dir` (string, optional): directory for artifact discovery.
  - `artifact` (string, optional): exact artifact filename.

### Resolution Order

- If `--package` is provided, it overrides `project.name`.
- If `build.cmd` exists, the CLI executes it and skips default build logic.
- If `build.output_dir` + `build.artifact` exist, use them directly.
- Otherwise, fall back to backend-specific defaults and discovery rules.

## CLI Commands

- `vtx init`: generate a template project and `vtx.toml`.
- `vtx check`: validate environment and configuration only.
- `vtx build`: compile source to Wasm and package as `.vtx`.
- `vtx package`: only package an existing Wasm output into `.vtx`.
- `vtx clean`: remove build artifacts.
- `vtx init`: generate a template project and `vtx.toml`.

## Build Backend Interface

Each language backend must implement:

- `check_env()`: verify toolchain availability.
- `build(package, target, release)`: build Wasm output.
- `find_output(package, target, release)`: locate artifact.

Backends must be stateless and only use filesystem side-effects.

## Artifact Conventions

- All backends must eventually produce a `.wasm` artifact.
- The CLI is responsible for Component encoding and `.vtx` packaging.
- When `build.artifact` is set, it must be used verbatim.

## Compatibility Checks

- The CLI may warn (or fail) on SDK version mismatch.
- The check is advisory; enforcement can be bypassed with `--force`.

## Extensibility

- New languages are added by registering a backend with a unique name.
- External builders can be supported via `build.cmd`.
## Init Templates

The CLI includes built-in templates for:

- Rust (`vtx init --lang rust`)
- TypeScript (`vtx init --lang ts`)
- Python (`vtx init --lang python`)

## Error Output

- Default: human-readable messages.
- Optional `--json` flag for structured errors (future).
