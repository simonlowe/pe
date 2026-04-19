use colored::Colorize;
use serde::Deserialize;
use std::collections::HashMap;

// ── API response types ────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GhPr {
    number: u64,
    title: String,
    body: Option<String>,
    draft: bool,
    mergeable: Option<bool>,
    mergeable_state: Option<String>,
    head: GhRef,
    requested_reviewers: Vec<GhUser>,
}

#[derive(Deserialize)]
struct GhRef {
    sha: String,
}

#[derive(Deserialize)]
struct GhUser {
    login: String,
}

#[derive(Deserialize, Default)]
struct GhCheckRunsPage {
    check_runs: Vec<GhCheckRun>,
}

#[derive(Deserialize)]
struct GhCheckRun {
    name: String,
    status: String,
    conclusion: Option<String>,
}

#[derive(Deserialize)]
struct GhReview {
    state: String,
    user: GhUser,
}

#[derive(Deserialize)]
struct GhWorkflowsPage {
    workflows: Vec<GhWorkflow>,
}

#[derive(Deserialize)]
struct GhWorkflow {
    id: u64,
    name: String,
}

#[derive(Deserialize, Default)]
struct GhRunsPage {
    workflow_runs: Vec<GhRun>,
}

#[derive(Deserialize)]
struct GhRun {
    status: String,
    conclusion: Option<String>,
    run_started_at: Option<String>,
}

// ── HTTP helpers ──────────────────────────────────────────────────────────────

fn build_request(url: &str, token: Option<&str>) -> ureq::Request {
    let req = ureq::get(url)
        .set("Accept", "application/vnd.github+json")
        .set("X-GitHub-Api-Version", "2022-11-28")
        .set("User-Agent", "startcode");
    match token {
        Some(t) => req.set("Authorization", &format!("Bearer {}", t)),
        None => req,
    }
}

fn api_get<T: for<'de> Deserialize<'de>>(url: &str, token: Option<&str>) -> Result<T, String> {
    match build_request(url, token).call() {
        Ok(resp) => resp.into_json().map_err(|e| e.to_string()),
        Err(ureq::Error::Status(401, _)) => {
            Err("Unauthorized — check your GitHub token".to_string())
        }
        Err(ureq::Error::Status(404, _)) => {
            Err("Repository not found or insufficient access".to_string())
        }
        Err(ureq::Error::Status(code, _)) => Err(format!("HTTP {}", code)),
        Err(e) => Err(e.to_string()),
    }
}

// ── URL parsing ───────────────────────────────────────────────────────────────

struct ParsedRepo {
    owner: String,
    repo: String,
    api_base: String,
}

fn api_base_for(hostname: &str) -> String {
    if hostname == "github.com" {
        "https://api.github.com".to_string()
    } else {
        // GitHub Enterprise Server: https://HOSTNAME/api/v3
        format!("https://{}/api/v3", hostname)
    }
}

fn parse_github_repo(remote_url: &str) -> Option<ParsedRepo> {
    let url = remote_url.trim();

    if url.starts_with("git@") {
        // git@hostname:owner/repo[.git]
        let rest = url.trim_start_matches("git@");
        let mut parts = rest.splitn(2, ':');
        let hostname = parts.next()?;
        let path = parts.next()?.trim_end_matches(".git");
        let mut path_parts = path.splitn(2, '/');
        let owner = path_parts.next()?.to_string();
        let repo = path_parts.next()?.to_string();
        Some(ParsedRepo { owner, repo, api_base: api_base_for(hostname) })
    } else if url.starts_with("https://") || url.starts_with("http://") {
        // https://hostname/owner/repo[.git]
        let rest = url.split_once("://")?.1;
        let mut parts = rest.splitn(2, '/');
        let hostname = parts.next()?;
        let path = parts.next()?.trim_end_matches(".git");
        let mut path_parts = path.splitn(2, '/');
        let owner = path_parts.next()?.to_string();
        let repo = path_parts.next()?.to_string();
        Some(ParsedRepo { owner, repo, api_base: api_base_for(hostname) })
    } else {
        None
    }
}

// ── Display helpers ───────────────────────────────────────────────────────────

fn merge_status_label(pr: &GhPr) -> String {
    if pr.draft {
        return "Draft".dimmed().to_string();
    }
    match pr.mergeable_state.as_deref() {
        Some("clean") => "Ready to merge".green().to_string(),
        Some("dirty") => "Has merge conflicts".red().to_string(),
        Some("unstable") => "Checks failing".yellow().to_string(),
        Some("blocked") => "Blocked by branch protection".red().to_string(),
        Some("behind") => "Branch behind base branch".yellow().to_string(),
        _ => match pr.mergeable {
            Some(true) => "Mergeable".green().to_string(),
            Some(false) => "Not mergeable".red().to_string(),
            None => "Computing merge status…".dimmed().to_string(),
        },
    }
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Returns true if any open PRs were found and printed, false otherwise.
pub fn print_pull_requests(remote_url: &str, token: Option<&str>, show_banner: bool) -> bool {
    let parsed = match parse_github_repo(remote_url) {
        Some(r) => r,
        None => return false, // Unrecognised remote format — skip silently
    };

    let base = format!("{}/repos/{}/{}", parsed.api_base, parsed.owner, parsed.repo);

    // List open PRs
    let pr_list: Vec<GhPr> = match api_get(&format!("{}/pulls?state=open&per_page=100", base), token) {
        Ok(p) => p,
        Err(e) => {
            if show_banner {
                println!();
                println!("{}", "Open Pull Requests:".bold().green());
                println!("{}", "─────────────────────".dimmed());
            }
            println!("  {} {}", "Could not fetch pull requests:".red(), e.dimmed());
            return false;
        }
    };

    if pr_list.is_empty() {
        if show_banner {
            println!();
            println!("{}", "Open Pull Requests:".bold().green());
            println!("{}", "─────────────────────".dimmed());
            println!("  {}", "No open pull requests".dimmed());
        }
        return false;
    }

    if show_banner {
        println!();
        println!("{}", "Open Pull Requests:".bold().green());
        println!("{}", "─────────────────────".dimmed());
    } else {
        // End the repo-name line that the caller left open
        println!();
    }

    for summary in &pr_list {
        // Fetch individual PR to get mergeable / mergeable_state
        let pr: GhPr =
            match api_get(&format!("{}/pulls/{}", base, summary.number), token) {
                Ok(p) => p,
                Err(e) => {
                    println!(
                        "  {} #{}: {}",
                        "Could not fetch PR".red(),
                        summary.number,
                        e.dimmed()
                    );
                    continue;
                }
            };

        // ── Title ────────────────────────────────────────────────────���───────
        let draft_tag = if pr.draft {
            format!(" {}", "[DRAFT]".dimmed())
        } else {
            String::new()
        };
        println!(
            "  {} {}{}",
            format!("#{}", pr.number).cyan().bold(),
            pr.title.bold(),
            draft_tag
        );

        // ── Description (first meaningful line) ──────────────────────────────
        if let Some(body) = &pr.body {
            let first = body
                .lines()
                .map(str::trim)
                .find(|l| !l.is_empty())
                .unwrap_or("");
            if !first.is_empty() {
                let display = if first.len() > 120 {
                    format!("{}…", &first[..120])
                } else {
                    first.to_string()
                };
                println!("    {}", display.dimmed());
            }
        }

        // ── Merge status ─────────────────────────────────────────────────────
        println!("    {} {}", "Merge status:".normal(), merge_status_label(&pr));

        // ── Check runs ───────────────────────────────────────────────────────
        let checks: GhCheckRunsPage = api_get(
            &format!("{}/commits/{}/check-runs?per_page=100", base, pr.head.sha),
            token,
        )
        .unwrap_or_default();

        // Deduplicate by name — last occurrence wins (most recent rerun)
        let mut by_name: HashMap<&str, &GhCheckRun> = HashMap::new();
        for run in &checks.check_runs {
            by_name.insert(&run.name, run);
        }

        if by_name.is_empty() {
            println!("    {} {}", "Checks:".normal(), "None".dimmed());
        } else {
            let failed: Vec<&str> = by_name
                .values()
                .filter(|r| {
                    matches!(
                        r.conclusion.as_deref(),
                        Some("failure") | Some("timed_out") | Some("action_required")
                    )
                })
                .map(|r| r.name.as_str())
                .collect();
            let running = by_name
                .values()
                .filter(|r| r.status != "completed")
                .count();
            let passed = by_name
                .values()
                .filter(|r| {
                    matches!(
                        r.conclusion.as_deref(),
                        Some("success") | Some("neutral") | Some("skipped")
                    )
                })
                .count();

            let mut parts: Vec<String> = Vec::new();
            if passed > 0 {
                parts.push(format!("✓ {} passed", passed).green().to_string());
            }
            if running > 0 {
                parts.push(format!("⏳ {} running", running).yellow().to_string());
            }
            if !failed.is_empty() {
                parts.push(
                    format!("✗ {} failed: {}", failed.len(), failed.join(", "))
                        .red()
                        .to_string(),
                );
            }
            println!("    {} {}", "Checks:".normal(), parts.join("  "));
        }

        // ── Reviews ──────────────────────────────────────────────────────────
        let reviews: Vec<GhReview> = api_get(
            &format!("{}/pulls/{}/reviews?per_page=100", base, pr.number),
            token,
        )
        .unwrap_or_default();

        // Latest non-comment review state per user
        let mut latest: HashMap<&str, &str> = HashMap::new();
        for review in &reviews {
            if review.state != "COMMENTED" && review.state != "DISMISSED" {
                latest.insert(&review.user.login, &review.state);
            }
        }

        let approved: Vec<&str> = latest
            .iter()
            .filter(|(_, s)| **s == "APPROVED")
            .map(|(u, _)| *u)
            .collect();
        let changes: Vec<&str> = latest
            .iter()
            .filter(|(_, s)| **s == "CHANGES_REQUESTED")
            .map(|(u, _)| *u)
            .collect();
        let pending: Vec<&str> = pr
            .requested_reviewers
            .iter()
            .map(|u| u.login.as_str())
            .collect();

        let mut parts: Vec<String> = Vec::new();
        if !approved.is_empty() {
            parts.push(
                format!("✓ {} approved ({})", approved.len(), approved.join(", "))
                    .green()
                    .to_string(),
            );
        }
        if !changes.is_empty() {
            parts.push(
                format!("✗ changes requested by: {}", changes.join(", "))
                    .red()
                    .to_string(),
            );
        }
        if !pending.is_empty() {
            parts.push(
                format!("⏳ awaiting review from: {}", pending.join(", "))
                    .yellow()
                    .to_string(),
            );
        }
        if parts.is_empty() {
            println!("    {} {}", "Reviews:".normal(), "No reviews yet".dimmed());
        } else {
            println!("    {} {}", "Reviews:".normal(), parts.join("  "));
        }

        println!(); // blank line between PRs
    }

    true
}

// ── Workflow helpers ──────────────────────────────────────────────────────────

fn is_leap(y: i64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

fn days_in_month(m: i64, y: i64) -> i64 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap(y) { 29 } else { 28 },
        _ => 0,
    }
}

fn iso8601_to_epoch(s: &str) -> Option<i64> {
    let s = s.split('+').next()?.trim_end_matches('Z');
    let (date_part, time_part) = s.split_once('T')?;
    let mut d = date_part.split('-');
    let year: i64 = d.next()?.parse().ok()?;
    let month: i64 = d.next()?.parse().ok()?;
    let day: i64 = d.next()?.parse().ok()?;
    let mut t = time_part.split(':');
    let hour: i64 = t.next()?.parse().ok()?;
    let min: i64 = t.next()?.parse().ok()?;
    let sec: i64 = t.next()
        .and_then(|s| s.split('.').next())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let mut days: i64 = 0;
    for y in 1970..year {
        days += if is_leap(y) { 366 } else { 365 };
    }
    for m in 1..month {
        days += days_in_month(m, year);
    }
    days += day - 1;
    Some(days * 86400 + hour * 3600 + min * 60 + sec)
}

fn relative_time(secs: i64) -> String {
    match secs {
        s if s < 60 => "just now".to_string(),
        s if s < 120 => "one minute ago".to_string(),
        s if s < 3600 => format!("{} minutes ago", s / 60),
        s if s < 7200 => "one hour ago".to_string(),
        s if s < 86400 => format!("{} hours ago", s / 3600),
        s if s < 172800 => "yesterday".to_string(),
        s if s < 604800 => format!("{} days ago", s / 86400),
        s if s < 1209600 => "one week ago".to_string(),
        s if s < 2592000 => format!("{} weeks ago", s / 604800),
        s if s < 5184000 => "one month ago".to_string(),
        s => format!("{} months ago", s / 2592000),
    }
}

// ── Workflows entry point ─────────────────────────────────────────────────────

pub fn print_workflows(remote_url: &str, token: Option<&str>, verbose: bool) {
    let parsed = match parse_github_repo(remote_url) {
        Some(r) => r,
        None => return,
    };

    let base = format!("{}/repos/{}/{}", parsed.api_base, parsed.owner, parsed.repo);

    let page: GhWorkflowsPage =
        match api_get(&format!("{}/actions/workflows?per_page=100", base), token) {
            Ok(p) => p,
            Err(e) => {
                println!("    {} {}", "Could not fetch workflows:".red(), e.dimmed());
                return;
            }
        };

    if page.workflows.is_empty() {
        println!("    {}", "No workflows found".dimmed());
        return;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    for wf in &page.workflows {
        let runs: GhRunsPage = api_get(
            &format!("{}/actions/workflows/{}/runs?per_page=1", base, wf.id),
            token,
        )
        .unwrap_or_default();

        let latest = runs.workflow_runs.first();

        let (status_str, is_success) = match latest {
            None => ("never run".dimmed().to_string(), false),
            Some(run) => match run.conclusion.as_deref() {
                Some("success") => ("✓ success".green().to_string(), true),
                Some("failure") => ("✗ failure".red().to_string(), false),
                Some("timed_out") => ("✗ timed out".red().to_string(), false),
                Some("action_required") => ("⚠ action required".yellow().to_string(), false),
                Some("cancelled") => ("cancelled".dimmed().to_string(), false),
                Some("skipped") => ("skipped".dimmed().to_string(), false),
                Some("neutral") | Some("stale") => ("neutral".dimmed().to_string(), false),
                Some(other) => (other.to_string(), false),
                None => match run.status.as_str() {
                    "in_progress" => ("⏳ in progress".yellow().to_string(), false),
                    "queued" => ("⏳ queued".yellow().to_string(), false),
                    "waiting" | "pending" | "requested" => {
                        ("⏳ waiting".yellow().to_string(), false)
                    }
                    other => (other.to_string(), false),
                },
            },
        };

        if is_success && !verbose {
            continue;
        }

        let time_str = latest
            .and_then(|r| r.run_started_at.as_deref())
            .and_then(iso8601_to_epoch)
            .map(|epoch| relative_time(now - epoch))
            .unwrap_or_else(|| "—".to_string());

        println!(
            "    {:<50} {}  {}",
            wf.name,
            status_str,
            time_str.dimmed()
        );
    }
}
