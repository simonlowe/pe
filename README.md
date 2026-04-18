# pe — Principal Engineer

A command-line tool for managing day-to-day engineering workflows: checking repository status, monitoring pull requests, syncing local branches, and tracking CI pipeline health across multiple projects.

## Quick Start

```bash
pe --help
pe status
pe sync -g my-group
pe prs -g my-group
pe workflows -g my-group
```

## Documentation

| Section | Description |
|---------|-------------|
| [User Manual](docs/user/README.md) | Full command reference and usage guide |
| [Developer Guide](docs/developer/README.md) | Building, cross-compiling, and extending the project |

## Global Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--verbose` | `-v` | Enable verbose output (shows additional detail in some commands) |
| `--config-file <path>` | `-c` | Override the default config file path |
| `--help` | `-h` | Print help |
| `--version` | | Print version |

## Configuration

The default configuration file is `~/.config/startcode/config.yml`.
See the [configuration section](docs/user/README.md#configuration) of the User Manual for full details.
