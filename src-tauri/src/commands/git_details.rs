use std::process::Command;
use serde::{Serialize, Deserialize};

use crate::commands::process::hide_console;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFile {
    pub path: String,
    pub status: String, // "staged", "unstaged", "untracked", "conflicted", "deleted", "renamed"
    pub original_path: Option<String>, // for renamed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitDetails {
    pub branch: String,
    pub current_commit: String,
    pub ahead: usize,
    pub behind: usize,
    pub remote_url: Option<String>,
    pub owner: Option<String>,
    pub repo: Option<String>,
    pub files: Vec<GitFile>,
}

pub fn get_git_details(repo_path: &str) -> Result<GitDetails, String> {
    // 1. Current Branch Name
    let branch = run_git(repo_path, &["rev-parse", "--abbrev-ref", "HEAD"])
        .unwrap_or_else(|_| "HEAD".to_string());

    // 2. Short SHA of the current commit
    let current_commit = run_git(repo_path, &["rev-parse", "--short", "HEAD"])
        .unwrap_or_else(|_| "unknown".to_string());

    // 3. Remote URL of 'origin' remote
    let remote_url = run_git(repo_path, &["remote", "get-url", "origin"])
        .ok()
        .map(|s| s.trim().to_string());

    // 4. Parse Owner and Repo from remote URL
    let (owner, repo) = remote_url.as_ref()
        .and_then(|url| parse_github_url(url))
        .unzip();

    // 5. Ahead and Behind tracking branch counts
    let mut ahead = 0;
    let mut behind = 0;
    if let Ok(ab_str) = run_git(repo_path, &["rev-list", "--left-right", "--count", "HEAD...@{u}"]) {
        let parts: Vec<&str> = ab_str.trim().split_whitespace().collect();
        if parts.len() == 2 {
            ahead = parts[0].parse().unwrap_or(0);
            behind = parts[1].parse().unwrap_or(0);
        }
    }

    // 6. Modified Files and Statuses
    let mut files = Vec::new();
    if let Ok(status_out) = run_git(repo_path, &["status", "--porcelain=v1"]) {
        for line in status_out.lines() {
            if line.len() < 4 {
                continue;
            }
            let xy = &line[0..2];
            let path_part = &line[3..];

            let x = xy.chars().next().unwrap_or(' ');
            let y = xy.chars().nth(1).unwrap_or(' ');

            let is_conflicted = matches!((x, y),
                ('D', 'D') | ('A', 'U') | ('U', 'D') | ('U', 'A') | ('D', 'U') | ('A', 'A') | ('U', 'U')
            );

            if is_conflicted {
                files.push(GitFile {
                    path: path_part.to_string(),
                    status: "conflicted".to_string(),
                    original_path: None,
                });
            } else if x == '?' && y == '?' {
                files.push(GitFile {
                    path: path_part.to_string(),
                    status: "untracked".to_string(),
                    original_path: None,
                });
            } else if x == 'R' || y == 'R' {
                let parts: Vec<&str> = path_part.split(" -> ").collect();
                if parts.len() == 2 {
                    files.push(GitFile {
                        path: parts[1].trim_matches('"').to_string(),
                        status: "renamed".to_string(),
                        original_path: Some(parts[0].trim_matches('"').to_string()),
                    });
                } else {
                    files.push(GitFile {
                        path: path_part.trim_matches('"').to_string(),
                        status: "renamed".to_string(),
                        original_path: None,
                    });
                }
            } else {
                if x != ' ' && x != '?' {
                    files.push(GitFile {
                        path: path_part.to_string(),
                        status: "staged".to_string(),
                        original_path: None,
                    });
                }
                if y != ' ' && y != '?' {
                    files.push(GitFile {
                        path: path_part.to_string(),
                        status: "unstaged".to_string(),
                        original_path: None,
                    });
                }
            }
        }
    }

    Ok(GitDetails {
        branch,
        current_commit,
        ahead,
        behind,
        remote_url,
        owner,
        repo,
        files,
    })
}

fn parse_github_url(url: &str) -> Option<(String, String)> {
    let trimmed = url.trim().trim_end_matches(".git");
    if trimmed.contains("github.com") {
        if let Some(parts) = trimmed.split("github.com").nth(1) {
            let clean_parts = parts.trim_start_matches(|c| c == '/' || c == ':');
            let mut split = clean_parts.split('/');
            let owner = split.next()?.to_string();
            let repo = split.next()?.to_string();
            return Some((owner, repo));
        }
    }
    None
}

fn run_git(repo_path: &str, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new("git");
    command.args(["-C", repo_path]);
    command.args(args);
    hide_console(&mut command);
    let output = command.output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}
