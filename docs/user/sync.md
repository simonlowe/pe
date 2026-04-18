# sync

Fetch the latest changes from all remotes for every repository in a monitored group, then automatically fast-forward any branches that are safely mergeable.

## Usage

```
pe sync [OPTIONS]
```

## Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--group <name>` | `-g` | Only sync repos in this group. Omit to sync all groups. |

## Examples

```bash
# Sync all groups
pe sync

# Sync only the "work" group
pe sync -g work
```

## What It Does

`sync` runs through four phases for each group:

### Phase 1 — Fetch

Runs `git fetch --all --prune` in each repository. For each repo it reports:

- `already up to date` — no remote changes
- `N branches updated, N files changed` — new commits were received

Repositories whose path no longer exists on disk are reported with a warning and skipped.

### Phase 2 — Branch Status

For each repository, displays the currently checked-out branch name alongside the count of branches that are fully in sync. Branches that require attention are listed individually:

- Branches that are **ahead** of their upstream (unpushed commits)
- Branches that are **behind** their upstream (not yet merged)
- Branches with **no upstream** configured (shown in orange)

Branches that are in sync are suppressed from the list and counted in the header:

```
  api-service  [main]  (8 branches in sync)
    * main  a1b2c3d [origin/main: behind 3] Merge pull request #41
```

Any local uncommitted changes (staged or unstaged) are shown in orange below the branch list.

### Phase 3 — Fast-Forward Merges

Branches that meet all of the following criteria are automatically fast-forwarded:

- Are **behind** but **not ahead** of their upstream (no local commits)
- Are **not** the currently checked-out branch

For each such branch, runs:

```
git fetch <remote> <remotebranch>:<localbranch>
```

This updates the local branch ref without requiring a checkout.

### Phase 3b — Pull Current Branch

If the currently checked-out branch is behind but not ahead of its upstream, **and** the working tree has no local changes, **pe** runs:

```
git pull --ff-only
```

If local changes are present the branch is skipped with a warning.

### Phase 4 — Final Branch Status

Repeats the branch status display after all merges. Branches that were previously behind should now show as in sync.

## Notes

- Only branches that are purely behind (zero ahead commits) are eligible for automatic merging. Branches with local commits are never touched.
- The currently checked-out branch requires a clean working tree for an automatic pull.
- Large groups with many repositories will take longer due to network fetch times.

## See Also

- [User Manual](README.md)
- [config](config.md) — set up groups
- [prs](prs.md) — view open pull requests
- [status](status.md) — check a single repository
