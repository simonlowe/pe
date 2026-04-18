# open

Open a project directory in your configured editor.

## Usage

```
pe open [PATH] [OPTIONS]
```

## Arguments

| Argument | Description | Default |
|----------|-------------|---------|
| `PATH` | Path to the project directory to open | `.` (current directory) |

## Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--editor <name>` | `-e` | Editor command to use, overriding the configured default |

## Examples

```bash
# Open the current directory in the configured editor
pe open

# Open a specific path
pe open ~/projects/my-app

# Open using a specific editor
pe open ~/projects/my-app --editor vim
```

## Configuration

The default editor is set in the config file:

```yaml
editor: code
```

Common values: `code` (VS Code), `vim`, `nvim`, `nano`, `cursor`, `zed`.

## See Also

- [User Manual](README.md)
- [list](list.md) — list available projects
