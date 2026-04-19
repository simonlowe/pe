use crate::config::Config;
use colored::Colorize;
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let output = Command::new("git").args(args).output().ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn git_lines(args: &[&str]) -> Vec<String> {
    git(args)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

pub fn run(cfg: &Config) {
    if git(&["rev-parse", "--is-inside-work-tree"]).is_none() {
        println!("{}", "Not a git repository.".red().bold());
        return;
    }

    print!("{}", "Fetching remotes…".dimmed());
    match Command::new("git")
        .args(["fetch", "--all", "--prune"])
        .output()
    {
        Ok(out) if out.status.success() => println!(" {}", "done".dimmed()),
        Ok(out) => println!(
            " {}",
            format!("warning: {}", String::from_utf8_lossy(&out.stderr).trim()).yellow()
        ),
        Err(e) => println!(" {}", format!("failed: {}", e).yellow()),
    }

    println!("{}", "Git Repository Status".bold().green());
    println!("{}", "─────────────────────".dimmed());

    let remotes = git_lines(&["remote"]);
    let local_branches = git_lines(&["branch", "--format=%(refname:short)"]);
    let has_commits = git(&["rev-parse", "HEAD"]).is_some();

    // Check local has 'main' (only meaningful once commits exist)
    if has_commits && !local_branches.contains(&"main".to_string()) {
        println!("  {}", "Error: No local branch named 'main'".red().bold());
    }

    // Check each remote has 'main'
    let remote_refs = git_lines(&["branch", "-r", "--format=%(refname:short)"]);
    for remote in &remotes {
        let has_main = remote_refs.iter().any(|b| b == &format!("{}/main", remote));
        if !has_main {
            println!(
                "  {} {} {}",
                "Error: Remote".red().bold(),
                remote.yellow(),
                "has no branch named 'main'".red()
            );
        }
    }

    // Current branch
    let current =
        git(&["branch", "--show-current"]).unwrap_or_else(|| "(detached HEAD)".to_string());
    let branch_display = if current == "main" {
        current.green().bold()
    } else {
        current.yellow().bold()
    };
    println!("  Local Branch is: {}", branch_display);

    println!();

    // Unstaged: untracked + modified-not-staged
    let untracked = git_lines(&["ls-files", "--others", "--exclude-standard"]).len();
    let modified = git_lines(&["diff", "--name-only"]).len();
    let unstaged = untracked + modified;
    if unstaged > 0 {
        println!(
            "  {}",
            format!("There are {} unstaged local files", unstaged).yellow()
        );
    } else {
        println!("  {}", "No unstaged files".dimmed());
    }

    // Staged but uncommitted
    let staged = git_lines(&["diff", "--cached", "--name-only"]).len();
    if staged > 0 {
        println!(
            "  {}",
            format!("There are {} staged files awaiting commit", staged).yellow()
        );
    } else {
        println!("  {}", "No staged files awaiting commit".dimmed());
    }

    println!();
    println!("{}", "Branch Tracking:".bold().green());
    println!("{}", "─────────────────────".dimmed());

    for branch in &local_branches {
        let upstream_remote = git(&["config", &format!("branch.{}.remote", branch)]);
        let upstream_merge = git(&["config", &format!("branch.{}.merge", branch)]);

        match (upstream_remote, upstream_merge) {
            (Some(remote), Some(merge)) => {
                let upstream_branch = merge.trim_start_matches("refs/heads/").to_string();

                if upstream_branch != *branch {
                    println!(
                        "  {} {} {}",
                        "Local branch".normal(),
                        branch.cyan(),
                        format!(
                            "has a different named upstream ({}/{})",
                            remote, upstream_branch
                        )
                        .yellow()
                    );
                }

                // Count unpushed commits
                let upstream_ref = format!("remotes/{}/{}", remote, branch);
                let unpushed =
                    git_lines(&["log", branch, "--not", &upstream_ref, "--oneline"]).len();
                if unpushed > 0 {
                    println!(
                        "  {} {} {}",
                        "Local branch".normal(),
                        branch.cyan(),
                        format!("has {} commits that have not been pushed", unpushed).red()
                    );
                } else {
                    println!(
                        "  {} {}",
                        branch.cyan(),
                        "is up to date with upstream".dimmed()
                    );
                }
            }
            _ => {
                println!(
                    "  {} {} {}",
                    "Local branch".normal(),
                    branch.cyan(),
                    "has no upstream".red()
                );
            }
        }
    }

    // Remote branches not merged into remote main
    for remote in &remotes {
        let remote_main = format!("{}/main", remote);
        // Skip if this remote has no main
        if !remote_refs.iter().any(|b| b == &remote_main) {
            continue;
        }

        let unmerged: Vec<String> = remote_refs
            .iter()
            .filter(|b| {
                // Skip main, HEAD aliases
                *b != &remote_main
                    && !b.ends_with("/HEAD")
                    && b.starts_with(&format!("{}/", remote))
            })
            .filter_map(|b| {
                let count =
                    git_lines(&["log", &format!("{}..{}", remote_main, b), "--oneline"]).len();
                if count > 0 {
                    let branch_name = b
                        .strip_prefix(&format!("{}/", remote))
                        .unwrap_or(b)
                        .to_string();
                    Some(format!(
                        "{} ({} unmerged commit{})",
                        branch_name,
                        count,
                        if count == 1 { "" } else { "s" }
                    ))
                } else {
                    None
                }
            })
            .collect();

        println!();
        println!(
            "{} {}{}",
            "Unmerged Remote Branches on".bold().green(),
            remote.cyan().bold(),
            ":".bold().green()
        );
        println!("{}", "─────────────────────".dimmed());
        if unmerged.is_empty() {
            println!("  {}", "All remote branches are merged into main".dimmed());
        } else {
            for entry in &unmerged {
                println!("  {} {}", "⚠".yellow(), entry.yellow());
            }
        }
    }

    // Pull requests
    let env_token = std::env::var("GITHUB_TOKEN").ok();

    for remote in &remotes {
        if let Some(url) = git(&["remote", "get-url", remote]) {
            let token = crate::config::fetch_github_token(&url, &cfg.github_tokens)
                .or(env_token.as_deref());
            let _ = crate::github::print_pull_requests(&url, token, true);
        }
    }
}
