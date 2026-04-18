# workflows

Show the status and last run time of every GitHub Actions workflow across all repositories in a monitored group.

## Usage

```
pe workflows [OPTIONS]
```

## Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--group <name>` | `-g` | Only show workflows for repos in this group. Omit to cover all groups. |
| `--verbose` | `-v` | Also show workflows whose last run was successful. By default these are hidden. |

## Examples

```bash
# Show failing/non-green workflows for all groups
pe workflows

# Show failing workflows for the "work" group
pe workflows -g work

# Show all workflows including successful ones
pe workflows -g work -v
```

## What It Shows

For each repository, **pe** fetches the list of configured GitHub Actions workflows and the most recent run of each. It then displays:

- **Workflow name** — left-aligned
- **Status** — colour-coded (see below)
- **Last run time** — expressed in human-readable relative terms

By default, workflows whose last run was successful are hidden to reduce noise. Pass `-v` to show everything.

## Status Colours

| Status | Colour | Description |
|--------|--------|-------------|
| `✓ success` | Green | Last run completed successfully |
| `✗ failure` | Red | Last run failed |
| `✗ timed out` | Red | Last run exceeded its time limit |
| `⚠ action required` | Yellow | Manual intervention needed |
| `⏳ in progress` | Yellow | Currently running |
| `⏳ queued` | Yellow | Queued and waiting to start |
| `⏳ waiting` | Yellow | Waiting on a required check or approval |
| `cancelled` | Dimmed | Last run was cancelled |
| `skipped` | Dimmed | Last run was skipped |
| `never run` | Dimmed | No run history for this workflow |

## Relative Time Format

Last run times are displayed as human-readable relative durations:

| Elapsed | Display |
|---------|---------|
| < 1 minute | `just now` |
| 1 minute | `one minute ago` |
| 2–59 minutes | `N minutes ago` |
| 1 hour | `one hour ago` |
| 2–23 hours | `N hours ago` |
| Yesterday | `yesterday` |
| 2–6 days | `N days ago` |
| 1 week | `one week ago` |
| 2–4 weeks | `N weeks ago` |
| 1 month | `one month ago` |
| 2+ months | `N months ago` |

## Example Output

```
Group: work
─────────────────────
  api-service
    Build and Test                                     ✗ failure        two hours ago
    Deploy to Staging                                  ⏳ in progress   just now
    Security Scan                                      ⚠ action required  yesterday

  frontend
    CI                                                 ⏳ queued        just now

  shared-lib
    (all workflows passing — use -v to show)
```

## GitHub Token

A token must be configured for each remote to fetch workflow data.
See [config](config.md) for instructions.

## See Also

- [User Manual](README.md)
- [config](config.md) — configure GitHub tokens and groups
- [prs](prs.md) — view open pull requests
- [sync](sync.md) — fetch and sync branches
