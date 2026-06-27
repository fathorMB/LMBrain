//! Registers the repository-scoped `lmbrain-mcp` server with Codex by writing
//! project-local Codex configuration and ensuring Codex trusts the workspace.

use std::path::{Path, PathBuf};

use toml_edit::{value, Array, DocumentMut, Item, Table};

use crate::commands::filesystem::clean_path;
use crate::errors::AppError;

const AGENTS_BEGIN: &str = "<!-- lmbrain:begin -->";
const AGENTS_END: &str = "<!-- lmbrain:end -->";

/// Build the `.codex/config.toml` content that registers the `lmbrain` MCP
/// server, preserving unrelated project Codex settings.
pub fn build_codex_project_config(
    existing: Option<&str>,
    command: &str,
    root: &str,
) -> Result<String, AppError> {
    let mut doc = parse_toml_or_empty(existing)?;
    doc["mcp_servers"]["lmbrain"]["command"] = value(command);

    let mut args = Array::new();
    args.push("--root");
    args.push(root);
    doc["mcp_servers"]["lmbrain"]["args"] = value(args);

    Ok(doc.to_string())
}

/// Write/refresh `.codex/config.toml` at the workspace root.
pub fn register_codex_mcp_server(root: &Path, command: &str) -> Result<PathBuf, AppError> {
    let config_dir = root.join(".codex");
    std::fs::create_dir_all(&config_dir)?;
    let config_path = config_dir.join("config.toml");
    let existing = std::fs::read_to_string(&config_path).ok();
    let content =
        build_codex_project_config(existing.as_deref(), command, &root.to_string_lossy())?;
    write_if_changed(&config_path, existing.as_deref(), &content)?;
    Ok(config_path)
}

/// Ensure `$CODEX_HOME/config.toml` trusts this workspace. Existing trust entries
/// are matched case-insensitively on Windows and are never rewritten.
pub fn ensure_codex_workspace_trusted(root: &Path) -> Result<Option<PathBuf>, AppError> {
    let config_path = codex_user_config_path()?;
    let existing = std::fs::read_to_string(&config_path).ok();
    let (content, changed) = build_codex_user_config(existing.as_deref(), root)?;
    if !changed {
        return Ok(None);
    }

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    write_if_changed(&config_path, existing.as_deref(), &content)?;
    Ok(Some(config_path))
}

/// Build the user-level Codex config with a trusted project entry if one is not
/// already present. The boolean says whether a write is needed.
pub fn build_codex_user_config(
    existing: Option<&str>,
    root: &Path,
) -> Result<(String, bool), AppError> {
    let mut doc = parse_toml_or_empty(existing)?;
    let project_key = codex_project_key(root);

    if has_trusted_project(&doc, &project_key) {
        return Ok((existing.unwrap_or_default().to_string(), false));
    }

    let projects = ensure_table(&mut doc, "projects");
    let mut project = Table::new();
    project["trust_level"] = value("trusted");
    projects.insert(&project_key, Item::Table(project));

    Ok((doc.to_string(), true))
}

/// Build a root `AGENTS.md` with a concise LMBrain-managed pointer block.
pub fn build_agents_md(existing: Option<&str>) -> String {
    let block = managed_agents_block();
    let existing = existing.unwrap_or_default();

    if let (Some(begin), Some(end)) = (existing.find(AGENTS_BEGIN), existing.find(AGENTS_END)) {
        let end = end + AGENTS_END.len();
        let mut content = String::new();
        content.push_str(existing[..begin].trim_end());
        if !content.is_empty() {
            content.push_str("\n\n");
        }
        content.push_str(&block);
        let suffix = existing[end..].trim_start();
        if !suffix.is_empty() {
            content.push_str("\n\n");
            content.push_str(suffix);
        }
        ensure_trailing_newline(content)
    } else if existing.trim().is_empty() {
        ensure_trailing_newline(block)
    } else {
        let mut content = existing.trim_end().to_string();
        content.push_str("\n\n");
        content.push_str(&block);
        ensure_trailing_newline(content)
    }
}

/// Write/refresh root `AGENTS.md`, preserving user-authored content outside the
/// LMBrain managed block.
pub fn scaffold_agents_md(root: &Path) -> Result<PathBuf, AppError> {
    let path = root.join("AGENTS.md");
    let existing = std::fs::read_to_string(&path).ok();
    let content = build_agents_md(existing.as_deref());
    write_if_changed(&path, existing.as_deref(), &content)?;
    Ok(path)
}

fn managed_agents_block() -> String {
    format!(
        "{AGENTS_BEGIN}\n# LMBrain Agent Instructions\n\nUse `.lmbrain/AGENT.md` as the Project Lead operating contract for this workspace. Follow `.lmbrain/CONTRACT.md` for artifact states and `.lmbrain/QUALITY.md` for implementation quality. Use LMBrain MCP tools for controlled artifact mutations when available instead of editing managed frontmatter by hand.\n{AGENTS_END}"
    )
}

fn parse_toml_or_empty(existing: Option<&str>) -> Result<DocumentMut, AppError> {
    match existing {
        Some(text) if !text.trim().is_empty() => text
            .parse::<DocumentMut>()
            .map_err(|error| AppError::ParseError(error.to_string())),
        _ => Ok(DocumentMut::new()),
    }
}

fn ensure_table<'a>(doc: &'a mut DocumentMut, key: &str) -> &'a mut Table {
    if !doc.as_table().contains_key(key) || !doc[key].is_table() {
        doc[key] = Item::Table(Table::new());
    }
    doc[key].as_table_mut().expect("item is a table")
}

fn has_trusted_project(doc: &DocumentMut, project_key: &str) -> bool {
    let Some(projects) = doc.get("projects").and_then(Item::as_table) else {
        return false;
    };

    projects.iter().any(|(key, item)| {
        path_keys_equal(key, project_key)
            && item
                .get("trust_level")
                .and_then(Item::as_value)
                .and_then(|value| value.as_str())
                == Some("trusted")
    })
}

fn codex_user_config_path() -> Result<PathBuf, AppError> {
    if let Ok(home) = std::env::var("CODEX_HOME") {
        if !home.trim().is_empty() {
            return Ok(PathBuf::from(home).join("config.toml"));
        }
    }

    let home = std::env::var("USERPROFILE")
        .or_else(|_| std::env::var("HOME"))
        .map_err(|_| AppError::WorkspaceNotFound("Could not resolve home directory".into()))?;
    Ok(PathBuf::from(home).join(".codex").join("config.toml"))
}

fn codex_project_key(root: &Path) -> String {
    let clean = root
        .canonicalize()
        .map(|path| clean_path(&path))
        .unwrap_or_else(|_| clean_path(root));
    let key = clean.to_string_lossy().to_string();
    if cfg!(windows) {
        key.to_ascii_lowercase()
    } else {
        key
    }
}

fn path_keys_equal(left: &str, right: &str) -> bool {
    let normalize = |value: &str| {
        let value = value.replace('/', "\\");
        if cfg!(windows) {
            value.to_ascii_lowercase()
        } else {
            value
        }
    };
    normalize(left) == normalize(right)
}

fn write_if_changed(path: &Path, existing: Option<&str>, content: &str) -> Result<(), AppError> {
    if existing == Some(content) {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension(format!(
        "{}.tmp",
        path.extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or("tmp")
    ));
    std::fs::write(&temp, content)?;
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    std::fs::rename(&temp, path)?;
    Ok(())
}

fn ensure_trailing_newline(mut content: String) -> String {
    if !content.ends_with('\n') {
        content.push('\n');
    }
    content
}

#[cfg(test)]
mod tests {
    use super::{
        build_agents_md, build_codex_project_config, build_codex_user_config, codex_project_key,
        has_trusted_project, path_keys_equal,
    };
    use toml_edit::DocumentMut;

    #[test]
    fn creates_codex_project_config_when_absent() {
        let out = build_codex_project_config(None, "/bin/lmbrain-mcp", "/ws").unwrap();
        let doc = out.parse::<DocumentMut>().unwrap();
        assert_eq!(
            doc["mcp_servers"]["lmbrain"]["command"].as_str(),
            Some("/bin/lmbrain-mcp")
        );
        assert_eq!(
            doc["mcp_servers"]["lmbrain"]["args"]
                .as_array()
                .unwrap()
                .get(0)
                .unwrap()
                .as_str(),
            Some("--root")
        );
        assert_eq!(
            doc["mcp_servers"]["lmbrain"]["args"]
                .as_array()
                .unwrap()
                .get(1)
                .unwrap()
                .as_str(),
            Some("/ws")
        );
    }

    #[test]
    fn preserves_existing_codex_project_tables() {
        let existing = r#"
model = "gpt-5.4"

[mcp_servers.other]
command = "other-mcp"

[profiles.default]
approval_policy = "never"
"#;
        let out = build_codex_project_config(Some(existing), "lmbrain-mcp", "/ws").unwrap();
        let doc = out.parse::<DocumentMut>().unwrap();
        assert_eq!(doc["model"].as_str(), Some("gpt-5.4"));
        assert_eq!(
            doc["mcp_servers"]["other"]["command"].as_str(),
            Some("other-mcp")
        );
        assert_eq!(
            doc["profiles"]["default"]["approval_policy"].as_str(),
            Some("never")
        );
        assert_eq!(
            doc["mcp_servers"]["lmbrain"]["command"].as_str(),
            Some("lmbrain-mcp")
        );
    }

    #[test]
    fn codex_project_config_is_idempotent() {
        let first = build_codex_project_config(None, "lmbrain-mcp", "/ws").unwrap();
        let second = build_codex_project_config(Some(&first), "lmbrain-mcp", "/ws").unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn trust_lookup_matches_existing_windows_casing() {
        let doc = r#"
[projects.'E:\Git\LMBrain']
trust_level = "trusted"
"#
        .parse::<DocumentMut>()
        .unwrap();

        assert!(has_trusted_project(&doc, r"e:\git\lmbrain"));
        assert!(path_keys_equal(r"E:\Git\LMBrain", r"e:/git/lmbrain"));
    }

    #[test]
    fn user_config_trust_adds_entry_and_preserves_other_tables() {
        let existing = r#"
model = "gpt-5.4"

[mcp_servers.node_repl]
command = "node"
"#;
        let (out, changed) =
            build_codex_user_config(Some(existing), std::path::Path::new(r"E:\Fresh\Repo"))
                .unwrap();
        let doc = out.parse::<DocumentMut>().unwrap();
        assert!(changed);
        assert_eq!(doc["model"].as_str(), Some("gpt-5.4"));
        assert_eq!(
            doc["mcp_servers"]["node_repl"]["command"].as_str(),
            Some("node")
        );
        assert!(has_trusted_project(&doc, r"e:\fresh\repo"));
    }

    #[test]
    fn user_config_trust_is_noop_when_existing_trusted_entry_matches() {
        let existing = r#"
[projects.'e:\fresh\repo']
trust_level = "trusted"
"#;
        let (out, changed) =
            build_codex_user_config(Some(existing), std::path::Path::new(r"E:\Fresh\Repo"))
                .unwrap();
        assert!(!changed);
        assert_eq!(out, existing);
    }

    #[test]
    fn managed_agents_block_is_idempotent_and_preserves_user_text() {
        let first = build_agents_md(Some("# Team Notes\n\nKeep this."));
        let second = build_agents_md(Some(&first));
        assert_eq!(first, second);
        assert!(first.contains("# Team Notes"));
        assert!(first.contains(".lmbrain/AGENT.md"));
    }

    #[test]
    fn codex_project_key_lowercases_windows_paths() {
        let key = codex_project_key(std::path::Path::new(r"E:\Git\LMBrain"));
        if cfg!(windows) {
            assert_eq!(key, r"e:\git\lmbrain");
        }
    }
}
