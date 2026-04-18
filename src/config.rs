use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct RepositorySettings {
    /// The primary branch name to check against (e.g. "main", "master")
    #[serde(default = "default_main_branch")]
    pub main_branch: String,

    /// The default remote name
    #[serde(default = "default_remote")]
    pub remote: String,
}

fn default_main_branch() -> String {
    "main".to_string()
}

fn default_remote() -> String {
    "origin".to_string()
}

impl Default for RepositorySettings {
    fn default() -> Self {
        RepositorySettings {
            main_branch: default_main_branch(),
            remote: default_remote(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoGroup {
    /// Display name of the group
    pub name: String,
    /// Root folder on the local machine that contains the repo subfolders
    pub path: String,
    /// Names of subfolders within `path` that each contain a git repo
    #[serde(default)]
    pub repos: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// Default editor to use with the `open` command
    #[serde(default = "default_editor")]
    pub editor: String,

    /// Default directory to search for projects
    #[serde(default)]
    pub projects_dir: Option<String>,

    /// Default repository settings applied to all repositories
    #[serde(default)]
    pub default_repository_settings: RepositorySettings,

    /// Map of remote URL prefix → GitHub personal access token.
    /// The key is matched against the start of the remote URL.
    /// Example:
    ///   github_tokens:
    ///     "https://github.com/myorg": ghp_xxxxxx
    #[serde(default)]
    pub github_tokens: HashMap<String, String>,

    /// Groups of monitored local repositories
    #[serde(default)]
    pub monitored_repos: Vec<RepoGroup>,
}

fn default_editor() -> String {
    "code".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Config {
            editor: default_editor(),
            projects_dir: None,
            default_repository_settings: RepositorySettings::default(),
            github_tokens: HashMap::new(),
            monitored_repos: Vec::new(),
        }
    }
}

pub fn default_path() -> PathBuf {
    dirs_next::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("startcode")
        .join("config.yml")
}

pub fn load(path: Option<&str>) -> Config {
    let config_path = match path {
        Some(p) => PathBuf::from(p),
        None => default_path(),
    };

    if !config_path.exists() {
        if path.is_some() {
            eprintln!(
                "{} Config file not found: {}",
                "Warning:".yellow().bold(),
                config_path.display()
            );
        }
        return Config::default();
    }

    match load_from(&config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!(
                "{} Failed to parse config {}: {}",
                "Warning:".yellow().bold(),
                config_path.display(),
                e
            );
            Config::default()
        }
    }
}

pub fn save(config: &Config, path: Option<&str>) -> Result<(), String> {
    let config_path = match path {
        Some(p) => PathBuf::from(p),
        None => default_path(),
    };
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut contents = serde_yaml::to_string(config).map_err(|e| e.to_string())?;

    // serde_yaml does not quote URL-like keys by default. Post-process to
    // wrap every github_tokens key in double quotes for safety.
    for key in config.github_tokens.keys() {
        let escaped = key.replace('\\', "\\\\").replace('"', "\\\"");
        let unquoted = format!("\n  {}: ", key);
        let quoted = format!("\n  \"{}\": ", escaped);
        contents = contents.replace(&unquoted, &quoted);
    }

    std::fs::write(&config_path, contents).map_err(|e| e.to_string())
}

fn load_from(path: &Path) -> Result<Config, String> {
    let contents = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_yaml::from_str(&contents).map_err(|e| e.to_string())
}

/// Find a GitHub token for the given remote URL by matching against the
/// configured prefix patterns. Falls back to the GITHUB_TOKEN env var.
pub fn fetch_github_token<'a>(
    remote_url: &str,
    tokens: &'a HashMap<String, String>,
) -> Option<&'a str> {
    tokens
        .iter()
        .find(|(prefix, _)| remote_url.starts_with(prefix.as_str()))
        .map(|(_, token)| token.as_str())
}
