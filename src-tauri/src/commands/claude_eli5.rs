use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

const STYLE: &str = include_str!("../../assets/claude-output-styles/ELI5.md");
const SETTINGS_LOCAL: &str = ".claude/settings.local.json";
const WORKTREE_INCLUDE_ENTRY: &str = ".claude/settings.local.json";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ClaudeEli5Preference {
    pub enabled: bool,
}

pub struct ClaudeEli5Service {
    preference: Mutex<ClaudeEli5Preference>,
    path: PathBuf,
}

impl ClaudeEli5Service {
    pub fn initialize(app_data_dir: &Path) -> Result<Self, String> {
        let directory = app_data_dir.join("lmbrain");
        fs::create_dir_all(&directory)
            .map_err(|error| format!("Cannot create settings directory: {error}"))?;
        let path = directory.join("claude-eli5.json");
        let preference = if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|error| format!("Cannot read Claude ELI5 preference: {error}"))?;
            serde_json::from_str(&content)
                .map_err(|error| format!("Claude ELI5 preference is invalid: {error}"))?
        } else {
            ClaudeEli5Preference::default()
        };
        Ok(Self {
            preference: Mutex::new(preference),
            path,
        })
    }

    pub fn preference(&self) -> ClaudeEli5Preference {
        self.preference
            .lock()
            .expect("Claude ELI5 preference lock poisoned")
            .clone()
    }

    pub fn set_enabled(
        &self,
        enabled: bool,
        workspace: Option<&Path>,
    ) -> Result<ClaudeEli5Preference, String> {
        if enabled {
            if let Some(root) = workspace {
                activate(root)?;
            } else {
                install_or_verify_style()?;
            }
        } else if let Some(root) = workspace {
            deactivate(root)?;
        }
        let next = ClaudeEli5Preference { enabled };
        atomic_write(
            &self.path,
            &serde_json::to_string_pretty(&next).expect("serializable preference"),
        )?;
        *self
            .preference
            .lock()
            .expect("Claude ELI5 preference lock poisoned") = next.clone();
        Ok(next)
    }

    pub fn apply_if_enabled(&self, root: &Path) -> Result<(), String> {
        if self.preference().enabled {
            activate(root)
        } else {
            Ok(())
        }
    }
}

pub fn activate(root: &Path) -> Result<(), String> {
    install_or_verify_style()?;
    let settings_path = root.join(SETTINGS_LOCAL);
    let mut settings = read_json_object(&settings_path, ".claude/settings.local.json")?;
    settings.insert("outputStyle".into(), Value::String("ELI5".into()));
    let serialized = serde_json::to_string_pretty(&Value::Object(settings))
        .map_err(|error| format!("Cannot serialize .claude/settings.local.json: {error}"))?;
    atomic_write(&settings_path, &serialized)?;
    add_worktree_include(root)
}

pub fn deactivate(root: &Path) -> Result<(), String> {
    let settings_path = root.join(SETTINGS_LOCAL);
    if settings_path.exists() {
        let mut settings = read_json_object(&settings_path, ".claude/settings.local.json")?;
        if settings.get("outputStyle") == Some(&Value::String("ELI5".into())) {
            settings.remove("outputStyle");
            let serialized =
                serde_json::to_string_pretty(&Value::Object(settings)).map_err(|error| {
                    format!("Cannot serialize .claude/settings.local.json: {error}")
                })?;
            atomic_write(&settings_path, &serialized)?;
        }
    }
    remove_worktree_include(root)
}

fn install_or_verify_style() -> Result<(), String> {
    let home = user_home().ok_or_else(|| "Cannot resolve the current user's home directory for Claude output styles. Set USERPROFILE (Windows) or HOME and try again.".to_string())?;
    let style_path = home.join(".claude").join("output-styles").join("ELI5.md");
    if style_path.exists() {
        let current = fs::read_to_string(&style_path).map_err(|error| {
            format!(
                "Cannot read existing ELI5 output style at {}: {error}",
                style_path.display()
            )
        })?;
        if current != STYLE {
            return Err(format!("An existing user-owned ELI5 output style differs from LMBrain's bundled definition: {}. Review or rename it manually; LMBrain will not overwrite it.", style_path.display()));
        }
        return Ok(());
    }
    atomic_write(&style_path, STYLE)
}

fn add_worktree_include(root: &Path) -> Result<(), String> {
    let path = root.join(".worktreeinclude");
    let current = if path.exists() {
        read_optional(&path)?
    } else {
        String::new()
    };
    if current
        .lines()
        .any(|line| line.trim() == WORKTREE_INCLUDE_ENTRY)
    {
        return Ok(());
    }
    let newline = if current.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut next = current;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push_str(newline);
    }
    next.push_str(WORKTREE_INCLUDE_ENTRY);
    next.push_str(newline);
    atomic_write(&path, &next)
}

fn remove_worktree_include(root: &Path) -> Result<(), String> {
    let path = root.join(".worktreeinclude");
    if !path.exists() {
        return Ok(());
    }
    let current = read_optional(&path)?;
    let newline = if current.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };
    let mut changed = false;
    let retained: Vec<&str> = current
        .lines()
        .filter(|line| {
            let owned = line.trim() == WORKTREE_INCLUDE_ENTRY;
            changed |= owned;
            !owned
        })
        .collect();
    if changed {
        let mut next = retained.join(newline);
        if !next.is_empty() && current.ends_with('\n') {
            next.push_str(newline);
        }
        atomic_write(&path, &next)?;
    }
    Ok(())
}

fn read_json_object(path: &Path, label: &str) -> Result<Map<String, Value>, String> {
    if !path.exists() {
        return Ok(Map::new());
    }
    let content = read_optional(path)?;
    match serde_json::from_str::<Value>(&content) {
        Ok(Value::Object(value)) => Ok(value),
        Ok(_) => Err(format!(
            "{label} must contain a JSON object; LMBrain left it unchanged."
        )),
        Err(error) => Err(format!(
            "{label} contains invalid JSON ({error}); LMBrain left it unchanged."
        )),
    }
}

fn read_optional(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("Cannot read {}: {error}", path.display()))
}

fn atomic_write(path: &Path, content: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("Cannot determine parent directory for {}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("Cannot create {}: {error}", parent.display()))?;
    let temporary = parent.join(format!(
        ".{}.lmbrain-tmp",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("config")
    ));
    fs::write(&temporary, content).map_err(|error| {
        format!(
            "Cannot write temporary file for {}: {error}",
            path.display()
        )
    })?;
    fs::rename(&temporary, path).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        format!("Cannot replace {} atomically: {error}", path.display())
    })
}

fn user_home() -> Option<PathBuf> {
    std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" }).map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::{activate, deactivate, STYLE};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn activation_merges_and_deactivation_preserves_a_user_change() {
        let directory = tempdir().unwrap();
        let root = directory.path();
        let settings = root.join(".claude/settings.local.json");
        fs::create_dir_all(settings.parent().unwrap()).unwrap();
        fs::write(&settings, r#"{"permissions":{"allow":["Read"]}}"#).unwrap();
        // Exercise project-local merge helpers without touching the real home style.
        let mut value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&settings).unwrap()).unwrap();
        value["outputStyle"] = serde_json::Value::String("ELI5".into());
        fs::write(&settings, serde_json::to_string_pretty(&value).unwrap()).unwrap();
        fs::write(root.join(".worktreeinclude"), "src/**\n").unwrap();
        super::add_worktree_include(root).unwrap();
        assert!(fs::read_to_string(root.join(".worktreeinclude"))
            .unwrap()
            .contains(".claude/settings.local.json"));
        deactivate(root).unwrap();
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&settings).unwrap()).unwrap();
        assert!(value.get("outputStyle").is_none());
        assert!(value.get("permissions").is_some());
        assert!(!fs::read_to_string(root.join(".worktreeinclude"))
            .unwrap()
            .contains(".claude/settings.local.json"));
        assert!(STYLE.contains("keep-coding-instructions: true"));
        let _ = activate; // retained as a public lifecycle entry point.
    }
}
