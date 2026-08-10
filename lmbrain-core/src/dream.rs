use std::{fs, path::{Path, PathBuf}};

use chrono::Local;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{frontmatter::{atomic_write, Document, FrontmatterError}, mutation_lock::ArtifactMutationLock, path::{PathError, PathGuard}, transitions::MutationResult};

pub const DREAM_EVENT_SCHEMA_VERSION: &str = "1";
const CLASSIFICATIONS: &[&str] = &["technical-debt", "design-debt"];
const CONFIDENCES: &[&str] = &["low", "medium", "high"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DreamCreateInput {
    pub title: String, pub classification: String, pub confidence: String,
    pub area: Option<String>, #[serde(default)] pub related_artifacts: Vec<String>,
    pub context_digest: String, pub rationale: String, pub suggested_disposition: String,
    pub actor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Dream {
    pub id: String, pub title: String, pub status: String, pub classification: String,
    pub confidence: String, pub area: Option<String>, pub related_artifacts: Vec<String>,
    pub context_digest: String, pub created: String, pub updated: String, pub body: String,
    pub path: String, pub malformed: bool,
}

#[derive(Debug, Error)]
pub enum DreamError { #[error(transparent)] Path(#[from] PathError), #[error(transparent)] Frontmatter(#[from] FrontmatterError), #[error(transparent)] Io(#[from] std::io::Error), #[error("invalid dream: {0}")] Invalid(String), #[error("dream changed concurrently: {0}")] Concurrent(String) }

pub fn list_dreams(root: &Path) -> Vec<Dream> {
    let mut dreams = Vec::new();
    for path in markdown_files(&root.join(".lmbrain/dreams")) {
        let id = path.file_stem().and_then(|v| v.to_str()).unwrap_or("MALFORMED").to_string();
        if !id.starts_with("DREAM-") { continue; }
        let relative = relative_path(root, &path);
        match fs::read_to_string(&path).ok().and_then(|s| Document::parse(&s).ok()) {
            Some(doc) => dreams.push(Dream { id: doc.value("id").unwrap_or(id), title: doc.value("title").unwrap_or_else(|| "Malformed dream".into()), status: doc.value("status").unwrap_or_else(|| "unknown".into()), classification: doc.value("classification").unwrap_or_else(|| "unknown".into()), confidence: doc.value("confidence").unwrap_or_else(|| "unknown".into()), area: doc.value("area"), related_artifacts: doc.string_array("related_artifacts"), context_digest: doc.value("context_digest").unwrap_or_default(), created: doc.value("created").unwrap_or_default(), updated: doc.value("updated").unwrap_or_default(), body: doc.body.to_string(), path: relative, malformed: !valid(&doc, &path) }),
            None => dreams.push(Dream { id, title: "Malformed dream".into(), status: "unknown".into(), classification: "unknown".into(), confidence: "unknown".into(), area: None, related_artifacts: vec![], context_digest: String::new(), created: String::new(), updated: String::new(), body: String::new(), path: relative, malformed: true }),
        }
    }
    dreams.sort_by(|a,b| b.created.cmp(&a.created).then_with(|| a.id.cmp(&b.id))); dreams
}

pub fn capture_dream(root: impl AsRef<Path>, input: DreamCreateInput) -> Result<MutationResult, DreamError> {
    validate_input(&input)?; let guard = PathGuard::new(root)?; let _lock = ArtifactMutationLock::acquire(guard.root(), "dream-allocation")?;
    let id = next_id(guard.root()); let dir = guard.root().join(".lmbrain/dreams/captured"); fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{id}-{}.md", slug(&input.title))); let date = Local::now().format("%Y-%m-%d").to_string();
    let content = format!("---\nid: {id}\ntitle: {}\nstatus: captured\nclassification: {}\nconfidence: {}\narea: {}\nrelated_artifacts: {}\ncontext_digest: {}\ncreated: {date}\nupdated: {date}\ndream_events:\n  - schema_version: {}\n    id: {id}-EVENT-001\n    timestamp: {}\n    action: captured\n    actor: {}\n---\n# {}\n\n## Grounded rationale\n\n{}\n\n## Suggested disposition\n\n{}\n", quote(&input.title), quote(&input.classification), quote(&input.confidence), optional(input.area.as_deref()), yaml_array(&input.related_artifacts), quote(&input.context_digest), DREAM_EVENT_SCHEMA_VERSION, Local::now().to_rfc3339(), quote(&input.actor), input.title.trim(), input.rationale.trim(), input.suggested_disposition.trim());
    atomic_write(&path, &content)?;
    Ok(MutationResult { id, status: "captured".into(), path: relative_path(guard.root(), &path).into(), forced: false })
}

fn validate_input(input: &DreamCreateInput) -> Result<(), DreamError> { for (name, value) in [("title", &input.title), ("context_digest", &input.context_digest), ("rationale", &input.rationale), ("suggested_disposition", &input.suggested_disposition), ("actor", &input.actor)] { if value.trim().is_empty() { return Err(DreamError::Invalid(format!("{name} is required"))); } } if !CLASSIFICATIONS.contains(&input.classification.as_str()) { return Err(DreamError::Invalid("classification must be technical-debt or design-debt".into())); } if !CONFIDENCES.contains(&input.confidence.as_str()) { return Err(DreamError::Invalid("confidence must be low, medium, or high".into())); } if input.related_artifacts.is_empty() { return Err(DreamError::Invalid("at least one related_artifact is required for provenance".into())); } Ok(()) }
fn valid(doc: &Document, path: &Path) -> bool { let status = doc.value("status").unwrap_or_default(); matches!(status.as_str(), "captured"|"triaged"|"promoted"|"discarded") && path.parent().and_then(|p|p.file_name()).and_then(|v|v.to_str()) == Some(status.as_str()) && CLASSIFICATIONS.contains(&doc.value("classification").unwrap_or_default().as_str()) && CONFIDENCES.contains(&doc.value("confidence").unwrap_or_default().as_str()) && !doc.string_array("related_artifacts").is_empty() && !doc.value("context_digest").unwrap_or_default().is_empty() }
fn next_id(root: &Path) -> String { let max = list_dreams(root).iter().filter_map(|d| d.id.strip_prefix("DREAM-")?.parse::<u32>().ok()).max().unwrap_or(0); format!("DREAM-{:03}", max + 1) }
fn markdown_files(dir: &Path) -> Vec<PathBuf> { let mut out = vec![]; if let Ok(entries)=fs::read_dir(dir) { for entry in entries.flatten() { let p=entry.path(); if p.is_dir() { out.extend(markdown_files(&p)); } else if p.extension().and_then(|v|v.to_str())==Some("md") { out.push(p); } } } out }
fn relative_path(root:&Path,path:&Path)->String { path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/") }
fn quote(value:&str)->String { serde_json::to_string(value.trim()).unwrap_or_else(|_| "\"\"".into()) }
fn optional(value:Option<&str>)->String { value.filter(|v|!v.trim().is_empty()).map(quote).unwrap_or_else(||"null".into()) }
fn yaml_array(values:&[String])->String { format!("[{}]", values.iter().map(|v|quote(v)).collect::<Vec<_>>().join(", ")) }
fn slug(value:&str)->String { let s:String=value.chars().map(|c|if c.is_ascii_alphanumeric(){c.to_ascii_lowercase()}else{'-'}).collect(); s.trim_matches('-').chars().take(72).collect::<String>() }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_requires_grounding_and_lists_the_new_record() {
        let dir = tempfile::tempdir().unwrap();
        let input = DreamCreateInput { title: "Tighten review provenance".into(), classification: "technical-debt".into(), confidence: "medium".into(), area: Some("backend".into()), related_artifacts: vec!["SPEC-001".into()], context_digest: "digest-2026-08-10".into(), rationale: "A tentative observation grounded in the current spec.".into(), suggested_disposition: "Triage before planning.".into(), actor: "AGENT-LEAD".into() };
        let result = capture_dream(dir.path(), input).unwrap();
        assert_eq!(result.id, "DREAM-001");
        let dreams = list_dreams(dir.path());
        assert_eq!(dreams.len(), 1); assert!(!dreams[0].malformed); assert_eq!(dreams[0].related_artifacts, ["SPEC-001"]);
    }
}
