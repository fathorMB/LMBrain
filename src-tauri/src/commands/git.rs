use std::process::Command;

use crate::commands::process::hide_console;
use crate::models::file::GitInfo;

/// Read-only Git metadata reader.
/// Uses `git` CLI commands to read branch and status information.
pub fn get_git_info(repo_path: &str) -> GitInfo {
    let branch = get_branch(repo_path);
    let is_clean = get_is_clean(repo_path);
    let current_commit = get_current_commit(repo_path);

    GitInfo {
        branch,
        is_clean,
        current_commit,
    }
}

fn get_branch(repo_path: &str) -> Option<String> {
    let mut command = Command::new("git");
    command.args(["-C", repo_path, "rev-parse", "--abbrev-ref", "HEAD"]);
    hide_console(&mut command);
    command.output().ok().and_then(|o| {
        if o.status.success() {
            String::from_utf8(o.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        }
    })
}

fn get_is_clean(repo_path: &str) -> Option<bool> {
    let mut command = Command::new("git");
    command.args(["-C", repo_path, "status", "--porcelain"]);
    hide_console(&mut command);
    let status = command.output().ok()?;

    if !status.status.success() {
        return None;
    }

    let output = String::from_utf8(status.stdout).ok()?;
    Some(output.trim().is_empty())
}

fn get_current_commit(repo_path: &str) -> Option<String> {
    let mut command = Command::new("git");
    command.args(["-C", repo_path, "rev-parse", "--short", "HEAD"]);
    hide_console(&mut command);
    command.output().ok().and_then(|o| {
        if o.status.success() {
            String::from_utf8(o.stdout)
                .ok()
                .map(|s| s.trim().to_string())
        } else {
            None
        }
    })
}
