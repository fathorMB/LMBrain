use keyring::{Entry, Error as KeyringError};
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::{Serialize, Deserialize};

const SERVICE_NAME: &str = "lmbrain";
const USERNAME: &str = "github_pat";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubPullRequest {
    pub number: u64,
    pub title: String,
    pub html_url: String,
    pub state: String,
    pub user: String,
    pub draft: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubWorkflowRun {
    pub id: u64,
    pub name: String,
    pub display_title: String,
    pub head_branch: String,
    pub head_sha: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub event: String,
    pub run_number: u64,
    pub run_attempt: u64,
    pub actor: Option<String>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: String,
    pub run_started_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubBranch {
    pub name: String,
    pub sha: String,
    pub protected: bool,
    pub commits: Vec<GitHubCommit>,
    pub merge_base_sha: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubCommit {
    pub sha: String,
    pub message: String,
    pub author: Option<String>,
    pub date: Option<String>,
    pub parents: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubDashboard {
    pub has_token: bool,
    pub default_branch: Option<String>,
    pub branches: Vec<GitHubBranch>,
    pub branches_error: Option<String>,
    pub pull_requests: Vec<GitHubPullRequest>,
    pub workflow_runs: Vec<GitHubWorkflowRun>,
}

fn github_pat_entry() -> Result<Entry, String> {
    Entry::new(SERVICE_NAME, USERNAME)
        .map_err(|error| format!("Could not open the operating-system credential store: {error}"))
}

pub fn get_github_pat() -> Result<Option<String>, String> {
    match github_pat_entry()?.get_password() {
        Ok(token) if token.trim().is_empty() => Ok(None),
        Ok(token) => Ok(Some(token)),
        Err(KeyringError::NoEntry) => Ok(None),
        Err(error) => Err(format!("Could not read the GitHub PAT from the credential store: {error}")),
    }
}

pub fn save_github_pat(token: &str) -> Result<(), String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("The GitHub PAT cannot be empty.".to_string());
    }

    github_pat_entry()?
        .set_password(token)
        .map_err(|error| format!("Could not save the GitHub PAT in the credential store: {error}"))?;

    // A keyring build without a native backend silently uses an in-memory mock.
    // Re-open the entry so a non-persistent write cannot report false success.
    if get_github_pat()?.as_deref() != Some(token) {
        return Err("The credential store accepted the GitHub PAT but did not persist it.".to_string());
    }

    Ok(())
}

pub fn delete_github_pat() -> Result<(), String> {
    match github_pat_entry()?.delete_credential() {
        Ok(()) | Err(KeyringError::NoEntry) => Ok(()),
        Err(error) => Err(format!("Could not delete the GitHub PAT from the credential store: {error}")),
    }
}

pub fn get_github_dashboard(owner: &str, repo: &str) -> Result<GitHubDashboard, String> {
    let token = get_github_pat()?;
    let has_token = token.is_some();

    let prs = fetch_pull_requests(owner, repo, token.as_deref())?;
    let runs = fetch_workflow_runs(owner, repo, token.as_deref())?;
    let default_branch = fetch_default_branch(owner, repo, token.as_deref()).ok();
    let (mut branches, branches_error) = match fetch_branches(owner, repo, token.as_deref()) {
        Ok(branches) => (branches, None),
        Err(error) => (Vec::new(), Some(error)),
    };
    for branch in branches.iter_mut().take(24) {
        if let Ok((commits, merge_base_sha)) = fetch_branch_commits(owner, repo, &branch.name, token.as_deref()) {
            branch.commits = commits;
            branch.merge_base_sha = merge_base_sha;
        }
    }

    Ok(GitHubDashboard {
        has_token,
        default_branch,
        branches,
        branches_error,
        pull_requests: prs,
        workflow_runs: runs,
    })
}

fn fetch_default_branch(owner: &str, repo: &str, token: Option<&str>) -> Result<String, String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}");
    let mut req = ureq::get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "lmbrain");
    if let Some(t) = token.filter(|value| !value.is_empty()) {
        req = req.set("Authorization", &format!("Bearer {t}"));
    }
    let response = req
        .call()
        .map_err(|error| format!("GitHub API repository request failed: {error}"))?;
    let json: serde_json::Value = response
        .into_json()
        .map_err(|error| format!("Failed to parse repository JSON: {error}"))?;
    json.get("default_branch")
        .and_then(|value| value.as_str())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "GitHub repository response did not include a default branch".to_string())
}

fn fetch_branches(owner: &str, repo: &str, token: Option<&str>) -> Result<Vec<GitHubBranch>, String> {
    let url = format!("https://api.github.com/repos/{owner}/{repo}/branches?per_page=100");
    let mut req = ureq::get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "lmbrain");
    if let Some(t) = token.filter(|value| !value.is_empty()) {
        req = req.set("Authorization", &format!("Bearer {t}"));
    }
    let response = req
        .call()
        .map_err(|error| format!("GitHub API branches request failed: {error}"))?;
    let json: serde_json::Value = response
        .into_json()
        .map_err(|error| format!("Failed to parse branches JSON: {error}"))?;
    let array = json
        .as_array()
        .ok_or_else(|| "GitHub branches response was not an array".to_string())?;

    Ok(array
        .iter()
        .filter_map(|item| {
            let name = item.get("name")?.as_str()?.to_owned();
            let sha = item.get("commit")?.get("sha")?.as_str()?.to_owned();
            Some(GitHubBranch {
                name,
                sha,
                protected: item.get("protected").and_then(|value| value.as_bool()).unwrap_or(false),
                commits: Vec::new(),
                merge_base_sha: None,
            })
        })
        .collect())
}

fn fetch_branch_commits(owner: &str, repo: &str, branch: &str, token: Option<&str>) -> Result<(Vec<GitHubCommit>, Option<String>), String> {
    let encoded_branch = utf8_percent_encode(branch, NON_ALPHANUMERIC).to_string();
    let url = format!("https://api.github.com/repos/{owner}/{repo}/commits?sha={encoded_branch}&per_page=50");
    let mut req = ureq::get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "lmbrain");
    if let Some(t) = token.filter(|value| !value.is_empty()) {
        req = req.set("Authorization", &format!("Bearer {t}"));
    }
    let response = req
        .call()
        .map_err(|error| format!("GitHub API commits request failed: {error}"))?;
    let json: serde_json::Value = response
        .into_json()
        .map_err(|error| format!("Failed to parse commits JSON: {error}"))?;
    let array = json.as_array().ok_or_else(|| "GitHub commits response was not an array".to_string())?;

    let mut commits: Vec<GitHubCommit> = array
        .iter()
        .filter_map(|item| {
            let sha = item.get("sha")?.as_str()?.to_owned();
            let commit = item.get("commit")?;
            Some(GitHubCommit {
                sha,
                message: commit
                    .get("message")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_owned(),
                author: commit
                    .get("author")
                    .and_then(|value| value.get("name"))
                    .and_then(|value| value.as_str())
                    .map(str::to_owned),
                date: commit
                    .get("author")
                    .and_then(|value| value.get("date"))
                    .and_then(|value| value.as_str())
                    .map(str::to_owned),
                parents: item
                    .get("parents")
                    .and_then(|value| value.as_array())
                    .map(|parents| {
                        parents
                            .iter()
                            .filter_map(|parent| parent.get("sha").and_then(|value| value.as_str()))
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default(),
            })
        })
        .collect();
    commits.truncate(50);
    Ok((commits, None))
}

fn fetch_pull_requests(owner: &str, repo: &str, token: Option<&str>) -> Result<Vec<GitHubPullRequest>, String> {
    let url = format!("https://api.github.com/repos/{}/{}/pulls?state=open&per_page=30", owner, repo);
    let mut req = ureq::get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "lmbrain");

    if let Some(t) = token {
        if !t.is_empty() {
            req = req.set("Authorization", &format!("Bearer {}", t));
        }
    }

    let response = match req.call() {
        Ok(res) => res,
        Err(ureq::Error::Status(status, res)) => {
            let body = res.into_string().unwrap_or_default();
            return Err(format!("GitHub API Pulls request failed with status {status}: {body}"));
        }
        Err(e) => {
            return Err(format!("GitHub API Pulls connection failed: {e}"));
        }
    };

    let json_array: serde_json::Value = response.into_json()
        .map_err(|e| format!("Failed to parse Pull Requests JSON: {e}"))?;

    let mut prs = Vec::new();
    if let Some(arr) = json_array.as_array() {
        for item in arr {
            let number = item.get("number").and_then(|v| v.as_u64()).unwrap_or(0);
            let title = item.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let html_url = item.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let state = item.get("state").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let user = item.get("user")
                .and_then(|u| u.get("login"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let draft = item.get("draft").and_then(|v| v.as_bool()).unwrap_or(false);
            let created_at = item.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let updated_at = item.get("updated_at").and_then(|v| v.as_str()).unwrap_or("").to_string();

            prs.push(GitHubPullRequest {
                number,
                title,
                html_url,
                state,
                user,
                draft,
                created_at,
                updated_at,
            });
        }
    }

    Ok(prs)
}

fn fetch_workflow_runs(owner: &str, repo: &str, token: Option<&str>) -> Result<Vec<GitHubWorkflowRun>, String> {
    // Intentionally unfiltered: the dashboard must surface every outcome
    // (success, failure, cancelled, skipped, queued, in-progress, ...).
    let url = format!("https://api.github.com/repos/{}/{}/actions/runs?per_page=30", owner, repo);
    let mut req = ureq::get(&url)
        .set("Accept", "application/vnd.github+json")
        .set("User-Agent", "lmbrain");

    if let Some(t) = token {
        if !t.is_empty() {
            req = req.set("Authorization", &format!("Bearer {}", t));
        }
    }

    let response = match req.call() {
        Ok(res) => res,
        Err(ureq::Error::Status(status, res)) => {
            let body = res.into_string().unwrap_or_default();
            return Err(format!("GitHub API Runs request failed with status {status}: {body}"));
        }
        Err(e) => {
            return Err(format!("GitHub API Runs connection failed: {e}"));
        }
    };

    let json_response: serde_json::Value = response.into_json()
        .map_err(|e| format!("Failed to parse Workflow Runs JSON: {e}"))?;

    let mut runs = Vec::new();
    if let Some(runs_arr) = json_response.get("workflow_runs").and_then(|v| v.as_array()) {
        for item in runs_arr {
            let id = item.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let display_title = item.get("display_title").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let head_branch = item.get("head_branch").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let head_sha = item.get("head_sha").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let conclusion = item.get("conclusion").and_then(|v| v.as_str()).map(|s| s.to_string());
            let event = item.get("event").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let run_number = item.get("run_number").and_then(|v| v.as_u64()).unwrap_or(0);
            let run_attempt = item.get("run_attempt").and_then(|v| v.as_u64()).unwrap_or(1);
            let actor = item.get("actor")
                .and_then(|a| a.get("login"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let html_url = item.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let created_at = item.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let updated_at = item.get("updated_at").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let run_started_at = item.get("run_started_at").and_then(|v| v.as_str()).map(|s| s.to_string());

            runs.push(GitHubWorkflowRun {
                id,
                name,
                display_title,
                head_branch,
                head_sha,
                status,
                conclusion,
                event,
                run_number,
                run_attempt,
                actor,
                html_url,
                created_at,
                updated_at,
                run_started_at,
            });
        }
    }

    Ok(runs)
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "windows")]
    #[test]
    fn native_windows_credential_store_persists_across_entries() {
        let service = format!("lmbrain-keyring-test-{}", uuid::Uuid::new_v4());
        let username = "github_pat_test";
        let token = "temporary-test-token";

        keyring::Entry::new(&service, username)
            .expect("create credential entry")
            .set_password(token)
            .expect("write temporary credential");

        let second_entry = keyring::Entry::new(&service, username).expect("re-open credential entry");
        let readback = second_entry.get_password();
        let cleanup = second_entry.delete_credential();

        assert_eq!(readback.expect("read temporary credential"), token);
        cleanup.expect("delete temporary credential");
    }
}
