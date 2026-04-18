# Developer Guide

This guide covers building **pe** from source, cross-compiling for multiple platforms, and understanding the codebase structure.

## Table of Contents

- [Prerequisites](#prerequisites)
- [Building](#building)
- [Cross-Compilation](#cross-compilation)
- [Project Structure](#project-structure)
- [Architecture](#architecture)
- [Adding a New Subcommand](#adding-a-new-subcommand)
- [Configuration Schema](#configuration-schema)

---

## Prerequisites

- [Rust](https://rustup.rs/) (stable toolchain, edition 2024)
- Cargo (included with Rust)

For cross-compilation (all four targets):

| Target | Required tool | Install |
|--------|--------------|---------|
| `x86_64-unknown-linux-musl` | `musl-cross` | `brew install FiloSottile/musl-cross/musl-cross` |
| `x86_64-pc-windows-gnu` | `mingw-w64` | `brew install mingw-w64` |

Rust target toolchains are added automatically by the build script on first use.

---

## Building

### Debug build (current platform)

```bash
cargo build
./target/debug/pe --help
```

### Release build (current platform)

```bash
cargo build --release
./target/release/pe --help
```

### All platforms + install to ~/bin

```bash
./build-release.sh
```

This script:
1. Builds release binaries for all four targets
2. Copies the platform-appropriate binary into `dist/`
3. Installs it as `~/bin/pe`

Targets built:

| Target | Output file |
|--------|-------------|
| `aarch64-apple-darwin` | `dist/pe-macos-arm64` |
| `x86_64-apple-darwin` | `dist/pe-macos-x86_64` |
| `x86_64-unknown-linux-musl` | `dist/pe-linux-x86_64` |
| `x86_64-pc-windows-gnu` | `dist/pe-windows-x86_64.exe` |

---

## Cross-Compilation

### Linker configuration

Cross-compilation linkers are configured in `.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-musl]
linker = "x86_64-linux-musl-gcc"

[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

### Manual cross-build

```bash
# Linux (static musl binary)
cargo build --release --target x86_64-unknown-linux-musl

# Windows
cargo build --release --target x86_64-pc-windows-gnu

# macOS Intel
cargo build --release --target x86_64-apple-darwin

# macOS Apple Silicon
cargo build --release --target aarch64-apple-darwin
```

---

## Project Structure

```
pe/
├── src/
│   ├── main.rs          # CLI definition (clap), command dispatch
│   ├── config.rs        # Config file schema, load/save, token lookup
│   ├── git_status.rs    # `status` subcommand implementation
│   ├── github.rs        # GitHub REST API client (PRs, checks, reviews, workflows)
│   ├── prs.rs           # `prs` subcommand — iterates groups, calls github.rs
│   ├── sync.rs          # `sync` subcommand — fetch, ff-merge, branch status
│   └── workflows.rs     # `workflows` subcommand — iterates groups, calls github.rs
├── docs/
│   ├── user/            # User-facing documentation
│   └── developer/       # This guide
├── .cargo/
│   └── config.toml      # Cross-compilation linker settings
├── Cargo.toml           # Dependencies and package metadata
├── build-release.sh     # Multi-platform release build script
└── README.md
```

---

## Architecture

### CLI layer (`main.rs`)

All subcommands are defined as variants of the `Commands` enum using [clap](https://docs.rs/clap)'s derive API. `main()` parses the CLI, loads the config, and dispatches to the appropriate module.

Global flags (`--verbose`, `--config-file`) are declared with `global = true` on the `Cli` struct and are accessible in all subcommand handlers via `cli.verbose` and `cli.config_file`.

### Config layer (`config.rs`)

The `Config` struct is deserialized from YAML using [serde](https://docs.rs/serde) and [serde_yaml](https://docs.rs/serde_yaml). All fields have `#[serde(default)]` so missing keys fall back gracefully.

`save()` applies a post-processing step to ensure URL-like keys in `github_tokens` are double-quoted in the YAML output (serde_yaml does not quote them by default).

`fetch_github_token()` performs prefix matching against the token map — the first key that is a prefix of the given remote URL wins.

### GitHub API layer (`github.rs`)

All GitHub REST API calls are made via [ureq](https://docs.rs/ureq). The `api_get()` helper handles authentication headers and maps common HTTP errors to descriptive messages.

`parse_github_repo()` parses both SSH (`git@hostname:owner/repo`) and HTTPS (`https://hostname/owner/repo`) remote URLs and derives the correct API base URL:

- `github.com` → `https://api.github.com`
- any other hostname → `https://HOSTNAME/api/v3` (GitHub Enterprise Server)

### Group iteration pattern

`sync`, `prs`, and `workflows` all follow the same pattern:

1. Accept an optional group name
2. If given, look up the group in `cfg.monitored_repos` and operate on it
3. If omitted, iterate over all groups
4. For each repo in the group, construct the full path (`group.path / repo_name`), check it exists, then call the per-repo logic

---

## Adding a New Subcommand

1. **Add a variant** to the `Commands` enum in `src/main.rs`:

```rust
/// One-line description shown in --help
MyCommand {
    #[arg(short = 'g', long = "group")]
    group: Option<String>,
},
```

2. **Create a module** `src/my_command.rs`:

```rust
use crate::config::Config;
use colored::Colorize;

pub fn run(cfg: &Config, group_name: Option<&str>) {
    // implementation
}
```

3. **Declare the module** at the top of `src/main.rs`:

```rust
mod my_command;
```

4. **Add a match arm** in `main()`:

```rust
Some(Commands::MyCommand { group }) => {
    my_command::run(&cfg, group.as_deref());
}
```

5. If your command makes GitHub API calls, call `crate::github::` functions and pass the token obtained from:

```rust
let env_token = std::env::var("GITHUB_TOKEN").ok();
let token = crate::config::fetch_github_token(&url, &cfg.github_tokens)
    .or(env_token.as_deref());
```

---

## Configuration Schema

Defined in `src/config.rs`:

```rust
pub struct Config {
    pub editor: String,                            // default: "code"
    pub projects_dir: Option<String>,              // default: None
    pub default_repository_settings: RepositorySettings,
    pub github_tokens: HashMap<String, String>,    // prefix → token
    pub monitored_repos: Vec<RepoGroup>,
}

pub struct RepositorySettings {
    pub main_branch: String,   // default: "main"
    pub remote: String,        // default: "origin"
}

pub struct RepoGroup {
    pub name: String,
    pub path: String,
    pub repos: Vec<String>,    // subfolder names relative to path
}
```

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing (derive API) |
| `colored` | Terminal colour output |
| `serde` + `serde_yaml` | Config file serialisation |
| `dirs-next` | Platform home directory lookup |
| `ureq` | HTTP client for GitHub API calls |
