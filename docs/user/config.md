# config

Manage **pe** configuration from the command line. This covers two areas: GitHub authentication tokens and monitored repository groups.

## Usage

```
pe config [OPTIONS]
```

## Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--repo-pattern <prefix>` | `-p` | Remote URL prefix to associate a token with |
| `--github_token <token>` | `-k` | GitHub personal access token |
| `--group <name>` | `-g` | Repository group name |
| `--repo-folder <path>` | `-f` | Folder path (new group) or subfolder name (existing group) |
| `--delete` | `-d` | Delete a token or group |
| `--list` | `-l` | List the details of a group |

---

## GitHub Token Management

Tokens are stored as a map of remote URL prefixes to token strings. When making an API call, **pe** finds the token whose prefix matches the start of the remote URL.

### Add or update a token

```bash
pe config -p "https://github.com/myorg" -k ghp_xxxxxxxxxxxx
pe config -p "git@ghe.example.com:myorg" -k ghp_xxxxxxxxxxxx
```

### Delete a token

```bash
pe config -p "https://github.com/myorg" -d
```

### Token prefix matching

The prefix should be as specific as needed to target the right set of repositories. For example:

- `"https://github.com/myorg"` matches all repos under `myorg` on github.com
- `"git@ghe.example.com:myorg"` matches all repos under `myorg` on a GitHub Enterprise Server
- `"https://github.com"` would match all github.com remotes (use with care)

As a fallback, **pe** also checks the `GITHUB_TOKEN` environment variable.

---

## Repository Group Management

Groups allow `sync`, `prs`, and `workflows` to operate across many repositories at once. A group has a **name**, a **root path**, and a list of **repo subfolder names** beneath that path.

### Create a new group

Provide a group name and the absolute path to the folder that contains your repositories:

```bash
pe config -g work -f /Users/alice/projects/work
```

### Add a repo to an existing group

Once the group exists, use the same flags — `pe` detects that the group already exists and treats `-f` as a subfolder name relative to the group root:

```bash
pe config -g work -f api-service
pe config -g work -f frontend
pe config -g work -f shared-lib
```

You can also provide the full absolute path; **pe** will strip the group root prefix automatically:

```bash
pe config -g work -f /Users/alice/projects/work/api-service
```

### List group details

```bash
pe config -g work -l
```

Output:
```
Group: work
  Path: /Users/alice/projects/work
  Repos:
    api-service
    frontend
    shared-lib
```

### Delete a group

```bash
pe config -g work -d
```

---

## Config File Location

The default config file is `~/.config/startcode/config.yml`. Use the global `-c` flag to specify a different path:

```bash
pe -c /path/to/config.yml config -g work -l
```

## See Also

- [User Manual](README.md)
- [sync](sync.md)
- [prs](prs.md)
- [workflows](workflows.md)
