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

fn workflows_for_group(group: &RepoGroup, cfg: &Config, verbose: bool) {
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

        println!("  {}", repo_name.cyan().bold());

        let remotes = git_in(&repo_path, &["remote"])
            .unwrap_or_default()
            .lines()
            .map(|l| l.trim().to_string())
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>();

        if remotes.is_empty() {
            println!("    {}", "No remotes configured".dimmed());
            continue;
        }

        for remote in &remotes {
            if let Some(url) = git_in(&repo_path, &["remote", "get-url", remote]) {
                let token = crate::config::fetch_github_token(&url, &cfg.github_tokens)
                    .or(env_token.as_deref());
                crate::github::print_workflows(&url, token, verbose);
            }
        }
    }
}

pub fn run(cfg: &Config, group_name: Option<&str>, verbose: bool) {
    match group_name {
        Some(name) => match cfg.monitored_repos.iter().find(|g| g.name == name) {
            None => eprintln!("{} Group {} not found", "Error:".red().bold(), name.cyan()),
            Some(group) => workflows_for_group(group, cfg, verbose),
        },
        None => {
            if cfg.monitored_repos.is_empty() {
                println!("{}", "No groups configured".dimmed());
            } else {
                for group in &cfg.monitored_repos {
                    workflows_for_group(group, cfg, verbose);
                }
            }
        }
    }
}
