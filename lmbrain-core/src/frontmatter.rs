use std::{fs, path::Path};
use chrono::Local;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FrontmatterError { #[error("missing or malformed YAML frontmatter")] Malformed, #[error(transparent)] Io(#[from] std::io::Error) }

#[derive(Debug, Clone)]
pub struct Document { pub frontmatter: String, pub body: String, pub newline: &'static str }
impl Document {
    pub fn parse(source: &str) -> Result<Self, FrontmatterError> {
        let newline = if source.contains("\r\n") { "\r\n" } else { "\n" };
        if !source.starts_with("---") { return Err(FrontmatterError::Malformed); }
        let marker = format!("{newline}---");
        let end = source[3..].find(&marker).ok_or(FrontmatterError::Malformed)? + 3;
        let after = end + marker.len();
        Ok(Self { frontmatter: source[3..end].trim_start_matches(['\r','\n']).to_string(), body: source[after..].trim_start_matches(['\r','\n']).to_string(), newline })
    }
    pub fn value(&self, key: &str) -> Option<String> {
        self.frontmatter.lines().find_map(|line| {
            let trimmed = line.trim_start(); let (found, value) = trimmed.split_once(':')?;
            (found == key).then(|| value.trim().trim_matches('"').trim_matches('\'').to_string())
        })
    }
    /// Replaces only top-level scalar fields, retaining line order, comments, quoting of unrelated fields and newline style.
    pub fn set(&mut self, key: &str, value: &str) {
        let mut found = false;
        let lines: Vec<String> = self.frontmatter.lines().map(|line| {
            let trimmed = line.trim_start();
            if !found && !line.starts_with([' ', '\t']) && trimmed.split_once(':').map(|(k,_)| k == key).unwrap_or(false) {
                found = true; format!("{key}: {value}")
            } else { line.to_string() }
        }).collect();
        self.frontmatter = lines.join(self.newline);
        if !found { if !self.frontmatter.is_empty() { self.frontmatter.push_str(self.newline); } self.frontmatter.push_str(&format!("{key}: {value}")); }
    }
    pub fn append_activity(&mut self, action: &str) {
        let today = Local::now().format("%Y-%m-%d");
        // YAML block syntax is deliberately simple and parser-friendly; retain existing fields untouched.
        if self.value("activity").is_some() { self.frontmatter.push_str(&format!("{}  - date: {}{}    action: {}", self.newline, today, self.newline, yaml_scalar(action))); }
        else { self.frontmatter.push_str(&format!("{}activity:{}  - date: {}{}    action: {}", self.newline, self.newline, today, self.newline, yaml_scalar(action))); }
    }
    pub fn append_override_reason(&mut self, reason: &str) {
        self.body.push_str(&format!("{}{}## Mutation override{}{}", self.newline, self.newline, self.newline, reason.trim()));
    }
    pub fn render(&self) -> String { format!("---{}{}{}---{}{}", self.newline, self.frontmatter, self.newline, self.newline, self.body) }
}
fn yaml_scalar(value: &str) -> String { format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\"")) }
pub fn atomic_write(path: &Path, content: &str) -> Result<(), FrontmatterError> {
    let parent = path.parent().ok_or(FrontmatterError::Malformed)?;
    fs::create_dir_all(parent)?;
    let temp = parent.join(format!(".{}.{}.tmp", path.file_name().and_then(|v| v.to_str()).unwrap_or("artifact"), std::process::id()));
    fs::write(&temp, content)?;
    #[cfg(windows)] if path.exists() { fs::remove_file(path)?; }
    fs::rename(temp, path)?;
    Ok(())
}
