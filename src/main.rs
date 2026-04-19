mod config;
mod git_status;
mod github;
mod prs;
mod sync;
mod workflows;

use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "pe", about = "Principal Engineer", version, author)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Path to config file (default: ~/.config/startcode/config.yml)
    #[arg(short = 'c', long = "config-file", global = true)]
    config_file: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Open a project in your editor
    Open {
        /// Path to the project directory
        path: Option<String>,

        /// Editor to use (overrides config)
        #[arg(short, long)]
        editor: Option<String>,
    },
    /// List available projects
    List {
        /// Filter projects by name
        #[arg(short, long)]
        filter: Option<String>,
    },
    /// Show git repository status
    Status,
    /// Fetch latest changes for all monitored repos
    Sync {
        /// Only sync repos in this group
        #[arg(short = 'g', long = "group")]
        group: Option<String>,
    },
    /// Show open pull requests for monitored repos
    Prs {
        /// Only show PRs for repos in this group
        #[arg(short = 'g', long = "group")]
        group: Option<String>,
    },
    /// Show GitHub Actions workflow status for monitored repos
    Workflows {
        /// Only show workflows for repos in this group
        #[arg(short = 'g', long = "group")]
        group: Option<String>,
    },
    /// Manage startcode configuration
    Config {
        /// Remote URL prefix pattern to associate a token with
        #[arg(short = 'p', long = "repo-pattern")]
        repo_pattern: Option<String>,

        /// GitHub token to store for the given pattern
        #[arg(short = 'k', long = "github_token")]
        github_token: Option<String>,

        /// Group name for monitored repo group management
        #[arg(short = 'g', long = "group")]
        group: Option<String>,

        /// Folder path for the group
        #[arg(short = 'f', long = "repo-folder")]
        repo_folder: Option<String>,

        /// Delete the token or group/repo
        #[arg(short = 'd', long = "delete")]
        delete: bool,

        /// List group details
        #[arg(short = 'l', long = "list")]
        list: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let cfg = config::load(cli.config_file.as_deref());

    println!("{}", "startcode".bold().cyan());
    println!("{}", "──────────────────".dimmed());

    match cli.command {
        Some(Commands::Open { path, editor }) => {
            let path = path.unwrap_or_else(|| ".".to_string());
            let editor = editor.unwrap_or(cfg.editor);

            println!(
                "{} {} with {}",
                "Opening".green().bold(),
                path.yellow(),
                editor.cyan()
            );

            if cli.verbose {
                println!("{} editor={}, path={}", "verbose:".dimmed(), editor, path);
            }
        }
        Some(Commands::List { filter }) => {
            println!("{}", "Projects:".green().bold());
            if let Some(f) = &filter {
                println!("{} {}", "Filter:".dimmed(), f.yellow());
            }
            // Placeholder — add project discovery logic here
            println!("  {}", "(none found)".dimmed());
        }
        Some(Commands::Status) => {
            git_status::run(&cfg);
        }
        Some(Commands::Sync { group }) => {
            sync::run(&cfg, group.as_deref());
        }
        Some(Commands::Prs { group }) => {
            prs::run(&cfg, group.as_deref());
        }
        Some(Commands::Workflows { group }) => {
            workflows::run(&cfg, group.as_deref(), cli.verbose);
        }
        Some(Commands::Config {
            repo_pattern,
            github_token,
            group,
            repo_folder,
            delete,
            list,
        }) => {
            match (repo_pattern, github_token, group, repo_folder, delete, list) {
                // ── Token management ──────────────────────────────────────────
                (Some(pattern), Some(token), None, None, false, false) => {
                    let mut cfg = cfg;
                    cfg.github_tokens.insert(pattern.clone(), token);
                    match config::save(&cfg, cli.config_file.as_deref()) {
                        Ok(_) => println!(
                            "{} token for pattern {}",
                            "Saved".green().bold(),
                            pattern.cyan()
                        ),
                        Err(e) => eprintln!("{} {}", "Error:".red().bold(), e),
                    }
                }
                (Some(pattern), None, None, None, true, false) => {
                    let mut cfg = cfg;
                    if cfg.github_tokens.remove(&pattern).is_some() {
                        match config::save(&cfg, cli.config_file.as_deref()) {
                            Ok(_) => println!(
                                "{} token for pattern {}",
                                "Deleted".green().bold(),
                                pattern.cyan()
                            ),
                            Err(e) => eprintln!("{} {}", "Error:".red().bold(), e),
                        }
                    } else {
                        eprintln!(
                            "{} No token found for pattern {}",
                            "Error:".red().bold(),
                            pattern.cyan()
                        );
                    }
                }
                (Some(_), Some(_), None, None, true, false) => {
                    eprintln!(
                        "{} --github_token and --delete cannot be used together",
                        "Error:".red().bold()
                    );
                }
                (Some(_), None, None, None, false, false) => {
                    eprintln!(
                        "{} provide either --github_token (-k) to set or --delete (-d) to remove",
                        "Error:".red().bold()
                    );
                }

                // ── Group management ──────────────────────────────────────────
                (None, None, Some(name), None, false, true) => {
                    // List group details
                    match cfg.monitored_repos.iter().find(|g| g.name == name) {
                        None => {
                            eprintln!("{} Group {} not found", "Error:".red().bold(), name.cyan())
                        }
                        Some(group) => {
                            println!("{} {}", "Group:".bold().green(), group.name.cyan().bold());
                            println!("  {} {}", "Path:".normal(), group.path.yellow());
                            if group.repos.is_empty() {
                                println!("  {}", "No repo subfolders configured".dimmed());
                            } else {
                                println!("  {}", "Repos:".normal());
                                for repo in &group.repos {
                                    println!("    {}", repo.cyan());
                                }
                            }
                        }
                    }
                }
                (None, None, Some(name), Some(folder), false, false) => {
                    let mut cfg = cfg;
                    match cfg.monitored_repos.iter_mut().find(|g| g.name == name) {
                        None => {
                            // Group doesn't exist — create it with folder as the root path
                            cfg.monitored_repos.push(config::RepoGroup {
                                name: name.clone(),
                                path: folder.clone(),
                                repos: Vec::new(),
                            });
                            match config::save(&cfg, cli.config_file.as_deref()) {
                                Ok(_) => println!(
                                    "{} group {} at {}",
                                    "Added".green().bold(),
                                    name.cyan(),
                                    folder.yellow()
                                ),
                                Err(e) => eprintln!("{} {}", "Error:".red().bold(), e),
                            }
                        }
                        Some(group) => {
                            // Group exists — store only the relative subfolder name
                            let relative = std::path::Path::new(&folder)
                                .strip_prefix(&group.path)
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|_| folder.clone());
                            if group.repos.contains(&relative) {
                                eprintln!(
                                    "{} Repo {} already in group {}",
                                    "Error:".red().bold(),
                                    relative.cyan(),
                                    name.cyan()
                                );
                            } else {
                                group.repos.push(relative.clone());
                                match config::save(&cfg, cli.config_file.as_deref()) {
                                    Ok(_) => println!(
                                        "{} repo {} to group {}",
                                        "Added".green().bold(),
                                        relative.cyan(),
                                        name.cyan()
                                    ),
                                    Err(e) => eprintln!("{} {}", "Error:".red().bold(), e),
                                }
                            }
                        }
                    }
                }
                (None, None, Some(name), None, true, false) => {
                    // Delete group
                    let mut cfg = cfg;
                    let before = cfg.monitored_repos.len();
                    cfg.monitored_repos.retain(|g| g.name != name);
                    if cfg.monitored_repos.len() == before {
                        eprintln!("{} Group {} not found", "Error:".red().bold(), name.cyan());
                    } else {
                        match config::save(&cfg, cli.config_file.as_deref()) {
                            Ok(_) => println!("{} group {}", "Deleted".green().bold(), name.cyan()),
                            Err(e) => eprintln!("{} {}", "Error:".red().bold(), e),
                        }
                    }
                }

                (_, _, _, _, _, _) => {
                    eprintln!(
                        "{} Invalid combination of flags. Use --help for usage.",
                        "Error:".red().bold()
                    );
                }
            }
        }
        None => {
            println!("{}", "No command given. Try --help for usage.".yellow());
        }
    }
}
