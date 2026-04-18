# pe User Manual

**pe** (Principal Engineer) is a command-line tool that helps engineering teams manage multiple Git repositories from a single interface. It provides repository status checks, pull request monitoring, branch synchronisation, and CI workflow health reporting.

## Table of Contents

- [Installation](#installation)
- [Global Flags](#global-flags)
- [Configuration](#configuration)
- [Commands](#commands)
  - [open](open.md) — Open a project in your editor
  - [list](list.md) — List available projects
  - [status](status.md) — Show git repository status for the current directory
  - [config](config.md) — Manage configuration (tokens, groups)
  - [sync](sync.md) — Fetch and fast-forward sync all repos in a group
  - [prs](prs.md) — Show open pull requests for monitored repos
  - [workflows](workflows.md) — Show GitHub Actions workflow status

---

## Installation

Download the appropriate binary for your platform from the `dist/` directory of the build output, or build from source (see the [Developer Guide](../developer/README.md)).

Place the binary in a directory on your `PATH`, for example `~/bin/pe`.

---

## Global Flags

These flags are available on every command.

| Flag | Short | Description |
|------|-------|-------------|
| `--verbose` | `-v` | Enable verbose output. In `workflows`, this shows successful runs in addition to failures. |
| `--config-file <path>` | `-c` | Path to a config file. Overrides the default location. |
| `--help` | `-h` | Print help for the current command. |
| `--version` | | Print the installed version. |

---

## Configuration

**pe** reads its configuration from a YAML file. The default location is:

```
~/.config/startcode/config.yml
```

You can override this with `-c <path>` on any command.

### Full Configuration Reference

```yaml
# Editor to open with the `open` command
editor: code

# Root directory to search for projects (used by `list`)
projects_dir: null

# Default git settings applied to all repositories
default_repository_settings:
  main_branch: main
  remote: origin

# GitHub personal access tokens, keyed by remote URL prefix.
# The longest matching prefix wins.
github_tokens:
  "https://github.com/myorg": ghp_xxxxxxxxxxxx
  "git@ghe.example.com:myorg": ghp_xxxxxxxxxxxx

# Groups of local repositories to monitor.
# Each group has a name, a root path, and a list of subfolder names.
monitored_repos:
  - name: work
    path: /Users/alice/projects/work
    repos:
      - api-service
      - frontend
      - shared-lib
  - name: personal
    path: /Users/alice/projects/personal
    repos:
      - blog
      - tools
```

### Managing Tokens

GitHub tokens are used to authenticate API calls for pull requests and workflows.
Use the [`config`](config.md) command to add and remove tokens without editing the file manually.

### Managing Groups

Repository groups allow commands like `sync`, `prs`, and `workflows` to operate across many repositories at once.
Use the [`config`](config.md) command to create groups and add repos to them.
