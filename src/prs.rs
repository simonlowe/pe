use crate::config::{Config, RepoGroup};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

fn git_in(dir: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

fn prs_for_group(group: &RepoGroup, cfg: &Config) {
    println!();
    println!("{} {}", "Group:".bold().green(), group.name.cyan().bold());
    println!("{}", "─────────────────────".dimmed());

    if group.repos.is_empty() {
        println!("  {}", "No repos configured".dimmed());
        return;
    }

    let env_token = std::env::var("GITHUB_TOKEN").ok();

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

        let remotes = git_in(&repo_path, &["remote"])
            .unwrap_or_default()
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>();

        if remotes.is_empty() {
            println!(
                "  {}  {}",
                repo_name.cyan().bold(),
                "No remotes configured".dimmed()
            );
            continue;
        }

        for remote in &remotes {
            if let Some(url) = git_in(&repo_path, &["remote", "get-url", remote]) {
                let token = crate::config::fetch_github_token(&url, &cfg.github_tokens)
                    .or(env_token.as_deref());
                // Print repo name without newline; print_pull_requests will either
                // append "No open pull requests" on the same line or start a new line
                // before printing PRs.
                use std::io::Write;
                print!("  {}", repo_name.cyan().bold());
                let _ = std::io::stdout().flush();
                let had_prs = crate::github::print_pull_requests(&url, token, false);
                if !had_prs {
                    println!("  {}", "No open pull requests".dimmed());
                }
            }
        }
    }
}

pub fn run(cfg: &Config, group_name: Option<&str>) {
    match group_name {
        Some(name) => match cfg.monitored_repos.iter().find(|g| g.name == name) {
            None => eprintln!("{} Group {} not found", "Error:".red().bold(), name.cyan()),
            Some(group) => prs_for_group(group, cfg),
        },
        None => {
            if cfg.monitored_repos.is_empty() {
                println!("{}", "No groups configured".dimmed());
            } else {
                for group in &cfg.monitored_repos {
                    prs_for_group(group, cfg);
                }
            }
        }
    }
}
