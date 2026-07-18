use serde::{Deserialize, Serialize};
use std::path::{Component, Path};
use std::process::Command;

use crate::commands::filesystem::clean_path;
use crate::commands::process::hide_console;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFile {
    pub path: String,
    pub status: String, // "staged", "unstaged", "untracked", "conflicted", "deleted", "renamed"
    pub diff_target: String, // "staged", "unstaged", "untracked", "conflicted"
    pub original_path: Option<String>, // for renamed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitFileDiff {
    pub path: String,
    pub diff: String,
    pub binary: bool,
    pub truncated: bool,
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
    let (owner, repo) = remote_url
        .as_ref()
        .and_then(|url| parse_github_url(url))
        .unzip();

    // 5. Ahead and Behind tracking branch counts
    let mut ahead = 0;
    let mut behind = 0;
    if let Ok(ab_str) = run_git(
        repo_path,
        &["rev-list", "--left-right", "--count", "HEAD...@{u}"],
    ) {
        let parts: Vec<&str> = ab_str.split_whitespace().collect();
        if parts.len() == 2 {
            ahead = parts[0].parse().unwrap_or(0);
            behind = parts[1].parse().unwrap_or(0);
        }
    }

    // 6. Modified Files and Statuses
    let mut files = Vec::new();
    if let Ok(status_out) = run_git_raw(
        repo_path,
        &["status", "--porcelain=v1", "--untracked-files=all"],
    ) {
        for line in status_out.lines() {
            if line.len() < 4 {
                continue;
            }
            let xy = &line[0..2];
            let path_part = &line[3..];

            let x = xy.chars().next().unwrap_or(' ');
            let y = xy.chars().nth(1).unwrap_or(' ');

            let is_conflicted = matches!(
                (x, y),
                ('D', 'D')
                    | ('A', 'U')
                    | ('U', 'D')
                    | ('U', 'A')
                    | ('D', 'U')
                    | ('A', 'A')
                    | ('U', 'U')
            );

            if is_conflicted {
                files.push(GitFile {
                    path: path_part.to_string(),
                    status: "conflicted".to_string(),
                    diff_target: "conflicted".to_string(),
                    original_path: None,
                });
            } else if x == '?' && y == '?' {
                files.push(GitFile {
                    path: path_part.to_string(),
                    status: "untracked".to_string(),
                    diff_target: "untracked".to_string(),
                    original_path: None,
                });
            } else if x == 'R' || y == 'R' {
                let diff_target = if x == 'R' { "staged" } else { "unstaged" };
                let parts: Vec<&str> = path_part.split(" -> ").collect();
                if parts.len() == 2 {
                    files.push(GitFile {
                        path: parts[1].trim_matches('"').to_string(),
                        status: "renamed".to_string(),
                        diff_target: diff_target.to_string(),
                        original_path: Some(parts[0].trim_matches('"').to_string()),
                    });
                } else {
                    files.push(GitFile {
                        path: path_part.trim_matches('"').to_string(),
                        status: "renamed".to_string(),
                        diff_target: diff_target.to_string(),
                        original_path: None,
                    });
                }
            } else {
                if x != ' ' && x != '?' {
                    files.push(GitFile {
                        path: path_part.to_string(),
                        status: if x == 'D' { "deleted" } else { "staged" }.to_string(),
                        diff_target: "staged".to_string(),
                        original_path: None,
                    });
                }
                if y != ' ' && y != '?' {
                    files.push(GitFile {
                        path: path_part.to_string(),
                        status: if y == 'D' { "deleted" } else { "unstaged" }.to_string(),
                        diff_target: "unstaged".to_string(),
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

const MAX_DIFF_BYTES: usize = 512 * 1024;

pub fn get_git_file_diff(
    repo_path: &Path,
    path: &str,
    diff_target: &str,
) -> Result<GitFileDiff, String> {
    validate_git_path(path)?;

    let output = match diff_target {
        "staged" => run_diff(
            repo_path,
            &[
                "diff",
                "--cached",
                "--no-ext-diff",
                "--no-textconv",
                "--no-color",
                "--",
                path,
            ],
            false,
        )?,
        "unstaged" | "conflicted" => run_diff(
            repo_path,
            &[
                "diff",
                "--no-ext-diff",
                "--no-textconv",
                "--no-color",
                "--",
                path,
            ],
            false,
        )?,
        "untracked" => {
            let canonical_root = clean_path(
                &repo_path
                    .canonicalize()
                    .map_err(|_| "Repository is unavailable".to_string())?,
            );
            let candidate = canonical_root.join(path);
            let canonical_file = clean_path(
                &candidate
                    .canonicalize()
                    .map_err(|_| "Untracked file no longer exists".to_string())?,
            );
            if !canonical_file.starts_with(&canonical_root) || !canonical_file.is_file() {
                return Err("Untracked file is outside the repository".to_string());
            }
            let display_path = format!("./{}", path.replace('\\', "/"));
            run_diff(
                &canonical_root,
                &[
                    "diff",
                    "--no-index",
                    "--no-ext-diff",
                    "--no-textconv",
                    "--no-color",
                    "--",
                    null_device(),
                    &display_path,
                ],
                true,
            )?
        }
        _ => return Err("Unsupported Git diff target".to_string()),
    };

    let binary = output.lines().any(|line| line.starts_with("Binary files "));
    let (diff, truncated) = truncate_diff(output);
    Ok(GitFileDiff {
        path: path.to_string(),
        diff,
        binary,
        truncated,
    })
}

fn validate_git_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.chars().any(char::is_control)
        || path.starts_with(['/', '\\'])
        || path.as_bytes().get(1) == Some(&b':')
        || path.split(['/', '\\']).any(|segment| segment == "..")
    {
        return Err("Invalid repository-relative path".to_string());
    }

    let candidate = Path::new(path);
    if candidate.is_absolute()
        || candidate.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err("Invalid repository-relative path".to_string());
    }
    Ok(())
}

fn run_diff(repo_path: &Path, args: &[&str], accept_difference: bool) -> Result<String, String> {
    let mut command = Command::new("git");
    command.arg("-C").arg(repo_path).args(args);
    hide_console(&mut command);
    let output = command.output().map_err(|error| error.to_string())?;
    let code = output.status.code();
    if output.status.success() || (accept_difference && code == Some(1)) {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if detail.is_empty() {
            "Git could not produce a diff for this file".to_string()
        } else {
            detail
        })
    }
}

#[cfg(windows)]
fn null_device() -> &'static str {
    "NUL"
}

#[cfg(not(windows))]
fn null_device() -> &'static str {
    "/dev/null"
}

fn truncate_diff(mut diff: String) -> (String, bool) {
    if diff.len() <= MAX_DIFF_BYTES {
        return (diff, false);
    }
    let mut end = MAX_DIFF_BYTES;
    while !diff.is_char_boundary(end) {
        end -= 1;
    }
    diff.truncate(end);
    (diff, true)
}

fn parse_github_url(url: &str) -> Option<(String, String)> {
    let trimmed = url.trim().trim_end_matches(".git");
    if trimmed.contains("github.com") {
        if let Some(parts) = trimmed.split("github.com").nth(1) {
            let clean_parts = parts.trim_start_matches(['/', ':']);
            let mut split = clean_parts.split('/');
            let owner = split.next()?.to_string();
            let repo = split.next()?.to_string();
            return Some((owner, repo));
        }
    }
    None
}

fn run_git(repo_path: &str, args: &[&str]) -> Result<String, String> {
    run_git_raw(repo_path, args).map(|output| output.trim().to_string())
}

fn run_git_raw(repo_path: &str, args: &[&str]) -> Result<String, String> {
    let mut command = Command::new("git");
    command.args(["-C", repo_path]);
    command.args(args);
    hide_console(&mut command);
    let output = command.output().map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn git(repo: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn repository() -> tempfile::TempDir {
        let dir = tempdir().unwrap();
        git(dir.path(), &["init"]);
        git(dir.path(), &["config", "user.email", "tests@example.com"]);
        git(dir.path(), &["config", "user.name", "LMBrain tests"]);
        fs::write(dir.path().join("tracked.txt"), "before\n").unwrap();
        git(dir.path(), &["add", "tracked.txt"]);
        git(dir.path(), &["commit", "-m", "baseline"]);
        dir
    }

    #[test]
    fn rejects_paths_and_unknown_targets_that_could_escape_git_scope() {
        let dir = repository();
        for path in [
            "",
            "../outside",
            r"..\outside",
            "/absolute",
            "C:\\absolute",
            "bad\npath",
        ] {
            assert!(get_git_file_diff(dir.path(), path, "unstaged").is_err());
        }
        assert!(get_git_file_diff(dir.path(), "tracked.txt", "mystery").is_err());
    }

    #[test]
    fn returns_staged_and_unstaged_diffs_from_the_correct_git_area() {
        let dir = repository();
        fs::write(dir.path().join("tracked.txt"), "staged\n").unwrap();
        git(dir.path(), &["add", "tracked.txt"]);
        fs::write(dir.path().join("tracked.txt"), "unstaged\n").unwrap();

        let staged = get_git_file_diff(dir.path(), "tracked.txt", "staged").unwrap();
        assert!(staged.diff.contains("+staged"));
        assert!(!staged.diff.contains("+unstaged"));

        let unstaged = get_git_file_diff(dir.path(), "tracked.txt", "unstaged").unwrap();
        assert!(unstaged.diff.contains("+unstaged"));
        assert!(unstaged.diff.contains("-staged"));
    }

    #[test]
    fn renders_untracked_files_as_additions() {
        let dir = repository();
        fs::write(dir.path().join("new file.txt"), "first\nsecond\n").unwrap();

        let diff = get_git_file_diff(dir.path(), "new file.txt", "untracked").unwrap();
        assert!(diff.diff.contains("+first"));
        assert!(diff.diff.contains("+second"));
        assert!(!diff.binary);
        assert!(!diff.truncated);
    }

    #[test]
    fn classifies_renames_and_deletions_with_the_correct_diff_target() {
        let dir = repository();
        git(dir.path(), &["mv", "tracked.txt", "renamed.txt"]);
        let renamed = get_git_details(&dir.path().to_string_lossy())
            .unwrap()
            .files
            .into_iter()
            .find(|file| file.path == "renamed.txt")
            .unwrap();
        assert_eq!(renamed.status, "renamed");
        assert_eq!(renamed.diff_target, "staged");

        git(dir.path(), &["reset", "--hard", "HEAD"]);
        fs::remove_file(dir.path().join("tracked.txt")).unwrap();
        let worktree_deleted = get_git_details(&dir.path().to_string_lossy())
            .unwrap()
            .files
            .into_iter()
            .find(|file| file.path == "tracked.txt")
            .unwrap();
        assert_eq!(worktree_deleted.status, "deleted");
        assert_eq!(worktree_deleted.diff_target, "unstaged");

        git(dir.path(), &["add", "-u"]);
        let staged_deleted = get_git_details(&dir.path().to_string_lossy())
            .unwrap()
            .files
            .into_iter()
            .find(|file| file.path == "tracked.txt")
            .unwrap();
        assert_eq!(staged_deleted.status, "deleted");
        assert_eq!(staged_deleted.diff_target, "staged");
    }

    #[test]
    fn conflicted_target_reads_the_worktree_diff() {
        let dir = repository();
        fs::write(dir.path().join("tracked.txt"), "changed\n").unwrap();
        let diff = get_git_file_diff(dir.path(), "tracked.txt", "conflicted").unwrap();
        assert!(diff.diff.contains("+changed"));
    }

    #[test]
    fn truncation_preserves_utf8_boundaries() {
        let source = "é".repeat(MAX_DIFF_BYTES);
        let (diff, truncated) = truncate_diff(source);
        assert!(truncated);
        assert!(diff.len() <= MAX_DIFF_BYTES);
        assert!(diff.is_char_boundary(diff.len()));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_an_untracked_symlink_that_escapes_the_repository() {
        use std::os::unix::fs::symlink;

        let dir = repository();
        let outside = tempdir().unwrap();
        fs::write(outside.path().join("secret.txt"), "outside\n").unwrap();
        symlink(
            outside.path().join("secret.txt"),
            dir.path().join("escape.txt"),
        )
        .unwrap();

        let error = get_git_file_diff(dir.path(), "escape.txt", "untracked").unwrap_err();
        assert!(error.contains("outside the repository"));
    }
}
