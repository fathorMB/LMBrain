use keyring::Entry;
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
    pub head_branch: String,
    pub head_sha: String,
    pub status: String,
    pub conclusion: Option<String>,
    pub html_url: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubDashboard {
    pub has_token: bool,
    pub pull_requests: Vec<GitHubPullRequest>,
    pub workflow_runs: Vec<GitHubWorkflowRun>,
}

pub fn get_github_pat() -> Option<String> {
    if let Ok(entry) = Entry::new(SERVICE_NAME, USERNAME) {
        entry.get_password().ok()
    } else {
        None
    }
}

pub fn save_github_pat(token: &str) -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, USERNAME).map_err(|e| e.to_string())?;
    entry.set_password(token).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_github_pat() -> Result<(), String> {
    let entry = Entry::new(SERVICE_NAME, USERNAME).map_err(|e| e.to_string())?;
    entry.delete_credential().map_err(|e| e.to_string())?;
    Ok(())
}

pub fn get_github_dashboard(owner: &str, repo: &str) -> Result<GitHubDashboard, String> {
    let token = get_github_pat();
    let has_token = token.is_some() && !token.as_ref().unwrap().is_empty();

    let prs = fetch_pull_requests(owner, repo, token.as_deref())?;
    let runs = fetch_workflow_runs(owner, repo, token.as_deref())?;

    Ok(GitHubDashboard {
        has_token,
        pull_requests: prs,
        workflow_runs: runs,
    })
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
    let url = format!("https://api.github.com/repos/{}/{}/actions/runs?per_page=10", owner, repo);
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
            let head_branch = item.get("head_branch").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let head_sha = item.get("head_sha").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let conclusion = item.get("conclusion").and_then(|v| v.as_str()).map(|s| s.to_string());
            let html_url = item.get("html_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let created_at = item.get("created_at").and_then(|v| v.as_str()).unwrap_or("").to_string();

            runs.push(GitHubWorkflowRun {
                id,
                name,
                head_branch,
                head_sha,
                status,
                conclusion,
                html_url,
                created_at,
            });
        }
    }

    Ok(runs)
}
