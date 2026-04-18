# status

Show a comprehensive git status report for the repository in the current working directory.

## Usage

```
pe status
```

## What It Shows

`status` performs a `git fetch --all --prune` first, then reports:

### Repository Health
- Whether a local `main` branch exists
- Whether each remote has a `main` branch

### Current Branch
- The name of the currently checked-out branch
- Displayed in **green** if it is `main`, **yellow** otherwise

### Working Tree State
- Count of unstaged files (untracked + modified)
- Count of staged files awaiting commit

### Branch Tracking
For each local branch:
- Whether it has a configured upstream
- Whether the upstream branch has a different name
- How many local commits have not been pushed

### Unmerged Remote Branches
For each remote, lists branches that contain commits not yet merged into `remote/main`, along with the unmerged commit count.

### Open Pull Requests
For each remote, fetches and displays open pull requests from the GitHub API, including:
- PR number, title, and draft status
- First line of the PR description
- Merge status (clean, conflicts, blocked, etc.)
- CI check run summary (passed, running, failed)
- Review status (approved, changes requested, awaiting review)

## Example Output

```
startcode
──────────────────
Fetching remotes… done
Git Repository Status
─────────────────────
  Local Branch is: main

  No unstaged files
  No staged files awaiting commit

Branch Tracking:
─────────────────────
  main is up to date with upstream
  feature/my-feature has 2 commits that have not been pushed

Unmerged Remote Branches on origin:
─────────────────────
  ⚠ feature/my-feature (2 unmerged commits)

Open Pull Requests:
─────────────────────
  #12 Add new API endpoint
    Implements the /v2/items endpoint
    Merge status: Ready to merge
    Checks: ✓ 4 passed
    Reviews: ✓ 1 approved (alice)
```

## GitHub Token

To display pull requests, a GitHub token must be configured for the remote URL.
See [config](config.md) for instructions on adding a token.

## See Also

- [User Manual](README.md)
- [config](config.md) — configure GitHub tokens
- [sync](sync.md) — fetch and sync all repos in a group
- [prs](prs.md) — show PRs across monitored groups
