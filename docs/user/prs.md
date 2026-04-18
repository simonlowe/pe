# prs

Show open pull requests for all repositories in a monitored group.

## Usage

```
pe prs [OPTIONS]
```

## Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--group <name>` | `-g` | Only show PRs for repos in this group. Omit to cover all groups. |

## Examples

```bash
# Show PRs for all groups
pe prs

# Show PRs for the "work" group only
pe prs -g work
```

## What It Shows

For each repository in the group, **pe** fetches open pull requests from the GitHub API and displays:

- **PR number and title** — with a `[DRAFT]` tag for draft PRs
- **Description** — the first non-empty line of the PR body (truncated at 120 characters)
- **Merge status** — one of:
  - `Ready to merge` (green)
  - `Has merge conflicts` (red)
  - `Checks failing` (yellow)
  - `Blocked by branch protection` (red)
  - `Branch behind base branch` (yellow)
  - `Draft` (dimmed)
- **CI checks** — summary of check runs: passed, running, and failed counts. Failed check names are listed explicitly.
- **Reviews** — approved reviewers, users who have requested changes, and users whose review is still pending.

If a repository has no open pull requests, a concise `No open pull requests` message is shown on the same line as the repository name.

## Example Output

```
Group: work
─────────────────────
  api-service
  #42 Add pagination to /items endpoint
    Implements cursor-based pagination
    Merge status: Ready to merge
    Checks: ✓ 5 passed
    Reviews: ✓ 2 approved (alice, bob)

  #39 Fix timeout on large payloads [DRAFT]
    Merge status: Draft
    Checks: ✓ 3 passed  ⏳ 1 running
    Reviews: No reviews yet

  frontend  No open pull requests
  shared-lib  No open pull requests
```

## GitHub Token

A token must be configured for each remote to fetch PR data. Without a token, public repositories may work but private repositories will return an error.

See [config](config.md) for instructions on adding a token.

## See Also

- [User Manual](README.md)
- [config](config.md) — configure GitHub tokens and groups
- [workflows](workflows.md) — check CI pipeline status
- [status](status.md) — single-repo status including PRs
