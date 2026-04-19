use crate::config::{Config, RepoGroup};
use colored::Colorize;
use std::io::Write as IoWrite;
use std::path::Path;
use std::process::Command;

// ── Branch info ───────────────────────────────────────────────────────────────

struct UpstreamInfo {
    remote: String,
    remote_branch: String,
    ahead: usize,
    behind: usize,
}

struct BranchInfo {
    local_name: String,
    upstream: Option<UpstreamInfo>,
    raw_line: String,
    is_current: bool,
}

fn parse_branches(repo_path: &Path) -> Vec<BranchInfo> {
    let output = match Command::new("git")
        .args(["branch", "-vv"])
        .current_dir(repo_path)
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let text = String::from_utf8_lossy(&output.stdout);
    let mut branches = Vec::new();

    for line in text.lines() {
        if line.len() < 2 {
            continue;
        }
        let is_current = line.starts_with('*');
        let rest = &line[2..]; // strip leading "* " or "  "
        let local_name = rest.split_whitespace().next().unwrap_or("").to_string();
        if local_name.is_empty() {
            continue;
        }

        let upstream = match (line.find('['), line.find(']')) {
            (Some(start), Some(end)) if start < end => {
                let inner = &line[start + 1..end]; // e.g. "origin/main: behind 3"
                let (ref_part, status_part) = match inner.find(':') {
                    Some(pos) => (&inner[..pos], &inner[pos + 1..]),
                    None => (inner, ""),
                };
                let (remote, remote_branch) = match ref_part.find('/') {
                    Some(pos) => (ref_part[..pos].to_string(), ref_part[pos + 1..].to_string()),
                    None => (ref_part.to_string(), String::new()),
                };
                let mut ahead = 0usize;
                let mut behind = 0usize;
                for part in status_part.split(',') {
                    let part = part.trim();
                    if let Some(n) = part.strip_prefix("ahead ") {
                        ahead = n.trim().parse().unwrap_or(0);
                    } else if let Some(n) = part.strip_prefix("behind ") {
                        behind = n.trim().parse().unwrap_or(0);
                    }
                }
                Some(UpstreamInfo {
                    remote,
                    remote_branch,
                    ahead,
                    behind,
                })
            }
            _ => None,
        };

        branches.push(BranchInfo {
            local_name,
            upstream,
            raw_line: line.to_string(),
            is_current,
        });
    }

    branches
}

// ── Display branch status ─────────────────────────────────────────────────────
// Returns:
//   .0  ff candidates  (local, remote, remote_branch) — not current, only behind
//   .1  pull candidates (local, remote, remote_branch) — current branch, only behind

#[allow(clippy::type_complexity)]
fn print_branch_status<'a>(
    repo_name: &str,
    repo_path: &Path,
    branches: &'a [BranchInfo],
) -> (
    Vec<(&'a str, &'a str, &'a str)>,
    Vec<(&'a str, &'a str, &'a str)>,
) {
    let mut ff_candidates: Vec<(&str, &str, &str)> = Vec::new();
    let mut pull_candidates: Vec<(&str, &str, &str)> = Vec::new();

    let in_sync = branches
        .iter()
        .filter(|b| matches!(&b.upstream, Some(up) if up.ahead == 0 && up.behind == 0))
        .count();

    let current_branch = branches
        .iter()
        .find(|b| b.is_current)
        .map(|b| b.local_name.as_str());

    let mut header = format!("  {}", repo_name.cyan().bold());
    if let Some(name) = current_branch {
        header.push_str(&format!("  {}", format!("[{}]", name).bold()));
    }
    if in_sync > 0 {
        header.push_str(&format!(
            "  {}",
            format!(
                "({} branch{} in sync)",
                in_sync,
                if in_sync == 1 { "" } else { "es" }
            )
            .dimmed()
        ));
    }
    println!("{}", header);

    for branch in branches {
        match &branch.upstream {
            Some(up) if up.ahead == 0 && up.behind == 0 => {
                // suppressed — counted in header
            }
            Some(up) if up.ahead == 0 && up.behind > 0 => {
                println!("    {}", branch.raw_line);
                if branch.is_current {
                    pull_candidates.push((&branch.local_name, &up.remote, &up.remote_branch));
                } else {
                    ff_candidates.push((&branch.local_name, &up.remote, &up.remote_branch));
                }
            }
            None => {
                println!(
                    "    {}  {}",
                    branch.raw_line,
                    "no upstream".truecolor(255, 165, 0)
                );
            }
            _ => {
                println!("    {}", branch.raw_line);
            }
        }
    }

    // Show local changes if present
    if let Ok(out) = Command::new("git")
        .args(["status", "--porcelain"])
        .current_dir(repo_path)
        .output()
    {
        let porcelain = String::from_utf8_lossy(&out.stdout);
        let lines: Vec<&str> = porcelain.lines().filter(|l| !l.trim().is_empty()).collect();
        if !lines.is_empty() {
            println!("    {}", "Local changes:".truecolor(255, 165, 0).bold());
            for line in lines {
                println!("      {}", line.truecolor(255, 165, 0));
            }
        }
    }

    (ff_candidates, pull_candidates)
}

// ── Fetch ─────────────────────────────────────────────────────────────────────

struct FetchResult {
    updated_branches: usize,
    files_changed: usize,
}

fn fetch_repo(repo_path: &Path) -> Result<FetchResult, String> {
    let output = Command::new("git")
        .args(["fetch", "--all", "--prune"])
        .current_dir(repo_path)
        .output()
        .map_err(|e| e.to_string())?;

    let stderr = String::from_utf8_lossy(&output.stderr);
    let mut updated_branches = 0;
    let mut files_changed = 0;

    for line in stderr.lines() {
        let token = line.split_whitespace().next().unwrap_or("");
        if !token.contains("..") {
            continue;
        }
        let mut parts = token.splitn(2, "..");
        let old_sha = parts.next().unwrap_or("");
        let new_sha = parts.next().unwrap_or("");
        if old_sha.is_empty() || new_sha.is_empty() {
            continue;
        }
        updated_branches += 1;
        let count = Command::new("git")
            .args(["diff", "--name-only", &format!("{}..{}", old_sha, new_sha)])
            .current_dir(repo_path)
            .output()
            .ok()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout)
                    .lines()
                    .filter(|l| !l.trim().is_empty())
                    .count()
            })
            .unwrap_or(0);
        files_changed += count;
    }

    Ok(FetchResult {
        updated_branches,
        files_changed,
    })
}

// ── Group sync ────────────────────────────────────────────────────────────────

fn sync_group(group: &RepoGroup) {
    println!();
    println!("{} {}", "Group:".bold().green(), group.name.cyan().bold());
    println!("{}", "─────────────────────".dimmed());

    if group.repos.is_empty() {
        println!("  {}", "No repos configured".dimmed());
        return;
    }

    // ── Phase 1: fetch ────────────────────────────────────────────────────────
    let mut valid_repos: Vec<&String> = Vec::new();

    for repo_name in &group.repos {
        let repo_path = Path::new(&group.path).join(repo_name);

        if !repo_path.exists() {
            println!(
                "  {} {} {}",
                "⚠".yellow(),
                repo_name.yellow(),
                "— path not found".dimmed()
            );
            continue;
        }

        print!("  {} {}… ", "Fetching".dimmed(), repo_name.cyan());
        let _ = std::io::stdout().flush();

        match fetch_repo(&repo_path) {
            Ok(result) => {
                if result.updated_branches == 0 {
                    println!("{}", "already up to date".dimmed());
                } else {
                    println!(
                        "{} branch{} updated, {} file{} changed",
                        result.updated_branches.to_string().green().bold(),
                        if result.updated_branches == 1 {
                            ""
                        } else {
                            "es"
                        },
                        result.files_changed.to_string().yellow().bold(),
                        if result.files_changed == 1 { "" } else { "s" },
                    );
                }
                valid_repos.push(repo_name);
            }
            Err(e) => println!("{} {}", "failed:".red().bold(), e.dimmed()),
        }
    }

    // ── Phase 2: branch status + fast-forward candidates ─────────────────────
    println!();
    println!("{}", "Branch Status:".bold().green());
    println!("{}", "─────────────────────".dimmed());

    // Collect (repo_path, local, remote, remote_branch) for ff and pull candidates
    let mut all_ff: Vec<(String, String, String, String)> = Vec::new();
    let mut all_pull: Vec<(String, String, String, String)> = Vec::new();

    for repo_name in &valid_repos {
        let repo_path = Path::new(&group.path).join(repo_name);

        let branches = parse_branches(&repo_path);
        let (ff_candidates, pull_candidates) =
            print_branch_status(repo_name, &repo_path, &branches);

        for (local, remote, remote_branch) in ff_candidates {
            all_ff.push((
                repo_path.to_string_lossy().to_string(),
                local.to_string(),
                remote.to_string(),
                remote_branch.to_string(),
            ));
        }
        for (local, remote, remote_branch) in pull_candidates {
            println!(
                "  {} {} branch {} should have a possible GIT PULL",
                "debug:".yellow().bold(),
                repo_name.cyan(),
                local.cyan()
            );
            all_pull.push((
                repo_path.to_string_lossy().to_string(),
                local.to_string(),
                remote.to_string(),
                remote_branch.to_string(),
            ));
        }
    }

    // ── Phase 3: fast-forward merges ──────────────────────────────────────────
    if all_ff.is_empty() && all_pull.is_empty() {
        return;
    }

    println!();
    println!(
        "{}",
        format!(
            "Identified {} branch{} that can be safely merged by fast-forward",
            all_ff.len(),
            if all_ff.len() == 1 { "" } else { "es" }
        )
        .bold()
        .green()
    );
    println!("{}", "─────────────────────".dimmed());

    for (repo_path_str, local, remote, remote_branch) in &all_ff {
        let repo_path = Path::new(repo_path_str);
        let repo_name = repo_path.file_name().unwrap_or_default().to_string_lossy();

        print!(
            "  {} {}/{} → {}… ",
            "Merging".dimmed(),
            remote.cyan(),
            remote_branch.cyan(),
            format!("{}/{}", repo_name, local).yellow()
        );
        let _ = std::io::stdout().flush();

        let refspec = format!("{}:{}", remote_branch, local);
        let result = Command::new("git")
            .args(["fetch", remote, &refspec])
            .current_dir(repo_path)
            .output();

        match result {
            Ok(out) if out.status.success() => println!("{}", "done".green().bold()),
            Ok(out) => {
                let msg = String::from_utf8_lossy(&out.stderr);
                println!("{} {}", "failed:".red().bold(), msg.trim().dimmed());
            }
            Err(e) => println!("{} {}", "failed:".red().bold(), e.to_string().dimmed()),
        }
    }

    // ── Phase 3b: pull current branches that are only behind ─────────────────
    // Only attempt if there are no local changes in the working tree
    if !all_pull.is_empty() {
        println!();
        println!(
            "{}",
            "Pulling current branches that are behind:".bold().green()
        );
        println!("{}", "─────────────────────".dimmed());

        for (repo_path_str, local, _remote, _remote_branch) in &all_pull {
            let repo_path = Path::new(repo_path_str);
            let repo_name = repo_path.file_name().unwrap_or_default().to_string_lossy();

            // Check for local changes
            let dirty = Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(repo_path)
                .output()
                .ok()
                .map(|o| !String::from_utf8_lossy(&o.stdout).trim().is_empty())
                .unwrap_or(true);

            if dirty {
                println!(
                    "  {} {}/{} — {}",
                    "⚠".yellow(),
                    repo_name,
                    local.cyan(),
                    "skipped (local changes present)".dimmed()
                );
                continue;
            }

            print!("  {} {}/{}… ", "Pulling".dimmed(), repo_name, local.cyan());
            let _ = std::io::stdout().flush();

            let result = Command::new("git")
                .args(["pull", "--ff-only"])
                .current_dir(repo_path)
                .output();

            match result {
                Ok(out) if out.status.success() => println!("{}", "done".green().bold()),
                Ok(out) => {
                    let msg = String::from_utf8_lossy(&out.stderr);
                    println!("{} {}", "failed:".red().bold(), msg.trim().dimmed());
                }
                Err(e) => println!("{} {}", "failed:".red().bold(), e.to_string().dimmed()),
            }
        }
    }

    // ── Phase 4: final branch status ──────────────────────────────────────────
    println!();
    println!("{}", "Branch Status (after merge):".bold().green());
    println!("{}", "─────────────────────".dimmed());

    for repo_name in &valid_repos {
        let repo_path = Path::new(&group.path).join(repo_name);
        let branches = parse_branches(&repo_path);
        print_branch_status(repo_name, &repo_path, &branches);
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(cfg: &Config, group_name: Option<&str>) {
    match group_name {
        Some(name) => match cfg.monitored_repos.iter().find(|g| g.name == name) {
            None => eprintln!("{} Group {} not found", "Error:".red().bold(), name.cyan()),
            Some(group) => sync_group(group),
        },
        None => {
            if cfg.monitored_repos.is_empty() {
                println!("{}", "No groups configured".dimmed());
            } else {
                for group in &cfg.monitored_repos {
                    sync_group(group);
                }
            }
        }
    }
}
