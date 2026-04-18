# list

List available projects.

## Usage

```
pe list [OPTIONS]
```

## Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--filter <string>` | `-f` | Filter the project list by name |

## Examples

```bash
# List all projects
pe list

# List projects whose names contain "api"
pe list --filter api
```

## Notes

Project discovery uses the `projects_dir` setting in the config file:

```yaml
projects_dir: /Users/alice/projects
```

## See Also

- [User Manual](README.md)
- [open](open.md) — open a project in your editor
