use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Local;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    frontmatter::{atomic_write, Document, FrontmatterError},
    invariants,
    path::{PathError, PathGuard},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArtifactKind {
    Spec,
    Review,
    Adr,
    Agent,
    AgentProposal,
    Mcp,
    McpProposal,
    Handoff,
}

impl ArtifactKind {
    pub fn prefix(self) -> &'static str {
        match self {
            Self::Spec => "SPEC",
            Self::Review => "REVIEW",
            Self::Adr => "ADR",
            Self::Agent => "AGENT",
            Self::AgentProposal => "AGENT-PROP",
            Self::Mcp => "MCP",
            Self::McpProposal => "MCP-PROP",
            Self::Handoff => "HANDOFF",
        }
    }

    fn base(self) -> &'static str {
        match self {
            Self::Spec => "specs",
            Self::Review => "reviews",
            Self::Adr => "decisions",
            Self::Agent => "agents/profiles",
            Self::AgentProposal => "agents/proposals",
            Self::Mcp => "mcp/specs",
            Self::McpProposal => "mcp/proposals",
            Self::Handoff => "handoffs/active",
        }
    }

    fn moves_for_status(self) -> bool {
        matches!(self, Self::Spec | Self::Review)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MutationOptions {
    #[serde(default)]
    pub force: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationResult {
    pub id: String,
    pub status: String,
    pub path: PathBuf,
    pub forced: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequest {
    pub kind: ArtifactKind,
    pub title: String,
    pub status: Option<String>,
    pub fields: Vec<(String, String)>,
}

#[derive(Debug, Error)]
pub enum TransitionError {
    #[error(transparent)]
    Path(#[from] PathError),
    #[error(transparent)]
    Frontmatter(#[from] FrontmatterError),
    #[error("artifact is missing required field '{0}'")]
    Missing(String),
    #[error("illegal {kind:?} transition from '{from}' to '{to}'")]
    Illegal {
        kind: ArtifactKind,
        from: String,
        to: String,
    },
    #[error("invariant failed: {0}")]
    Invariant(String),
    #[error("force requires a non-empty reason")]
    MissingForceReason,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn transition(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    target: &str,
    options: MutationOptions,
) -> Result<MutationResult, TransitionError> {
    require_force_reason(&options)?;

    let guard = PathGuard::new(root)?;
    let path = guard.resolve_existing(artifact)?;
    let mut document = Document::parse(&fs::read_to_string(&path)?)?;
    let id = document
        .value("id")
        .ok_or_else(|| TransitionError::Missing("id".into()))?;
    let from = document
        .value("status")
        .ok_or_else(|| TransitionError::Missing("status".into()))?;
    let kind = kind_for_id(&id)
        .ok_or_else(|| TransitionError::Missing("recognized artifact ID".into()))?;

    if !allowed(kind, &from, target) {
        return Err(TransitionError::Illegal {
            kind,
            from,
            to: target.into(),
        });
    }

    if let Some(message) = invariant_failure(guard.root(), &path, kind, target, &document) {
        if !options.force {
            return Err(TransitionError::Invariant(message));
        }
    }

    document.set("status", target);
    document.set("updated", &today());
    document.append_activity(&format!("transitioned {from} -> {target}"));
    if let Some(reason) = options.reason.as_deref() {
        document.append_override_reason(reason);
    }

    let destination = destination_for(kind, &path, target)?;
    atomic_write(&destination, &document.render())?;
    if destination != path {
        fs::remove_file(&path)?;
    }

    Ok(MutationResult {
        id,
        status: target.into(),
        path: destination,
        forced: options.force,
    })
}

pub fn set_recommended_agent(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    agent: &str,
    options: MutationOptions,
) -> Result<MutationResult, TransitionError> {
    set_field(
        root,
        artifact,
        "recommended_agent",
        agent,
        options,
        |root| invariants::recommended_agent_resolves(root, Some(agent)),
    )
}

fn set_field(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    key: &str,
    value: &str,
    options: MutationOptions,
    valid: impl Fn(&Path) -> bool,
) -> Result<MutationResult, TransitionError> {
    require_force_reason(&options)?;

    let guard = PathGuard::new(root)?;
    let path = guard.resolve_existing(artifact)?;
    let mut document = Document::parse(&fs::read_to_string(&path)?)?;
    let id = document
        .value("id")
        .ok_or_else(|| TransitionError::Missing("id".into()))?;

    if !valid(guard.root()) && !options.force {
        return Err(TransitionError::Invariant(format!("invalid {key}")));
    }

    document.set(key, value);
    document.set("updated", &today());
    document.append_activity(&format!("set {key}"));
    if let Some(reason) = options.reason.as_deref() {
        document.append_override_reason(reason);
    }

    atomic_write(&path, &document.render())?;
    Ok(MutationResult {
        id,
        status: document.value("status").unwrap_or_default(),
        path,
        forced: options.force,
    })
}

pub fn create(
    root: impl AsRef<Path>,
    request: CreateRequest,
) -> Result<MutationResult, TransitionError> {
    let guard = PathGuard::new(root)?;
    let status = request
        .status
        .clone()
        .unwrap_or_else(|| default_status(request.kind).into());

    let mut dir = guard.root().join(".lmbrain").join(request.kind.base());
    if request.kind.moves_for_status() {
        dir = dir.join(&status);
    }
    fs::create_dir_all(&dir)?;

    let lock = guard.root().join(".lmbrain/.mutation-allocation.lock");
    let mut attempts = 0;
    while fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&lock)
        .is_err()
    {
        attempts += 1;
        if attempts > 100 {
            return Err(TransitionError::Invariant(
                "could not acquire ID allocation lock".into(),
            ));
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let result = create_locked(guard.root(), &dir, request, status);
    let _ = fs::remove_file(lock);
    result
}

fn create_locked(
    root: &Path,
    dir: &Path,
    request: CreateRequest,
    status: String,
) -> Result<MutationResult, TransitionError> {
    let id = format!(
        "{}-{:03}",
        request.kind.prefix(),
        next_id(root, request.kind)
    );
    let path = dir.join(format!("{}-{}.md", id, slug(&request.title)));

    let template = root
        .join(".lmbrain/templates")
        .join(template_name(request.kind));
    let mut source = fs::read_to_string(template).unwrap_or_else(|_| default_template());
    let date = today();

    source = source
        .replace(&format!("{}-XXX", request.kind.prefix()), &id)
        .replace("Concise task title", &request.title)
        .replace("Feature or work item title", &request.title)
        .replace("YYYY-MM-DD", &date);

    let mut document = Document::parse(&source)?;
    document.set("id", &id);
    document.set(
        "title",
        &format!("\"{}\"", request.title.replace('"', "\\\"")),
    );
    document.set("status", &status);
    document.set("created", &date);
    document.set("updated", &date);
    for (key, value) in request.fields {
        document.set(&key, &value);
    }
    document.append_activity("created");
    atomic_write(&path, &document.render())?;

    Ok(MutationResult {
        id,
        status,
        path,
        forced: false,
    })
}

fn next_id(root: &Path, kind: ArtifactKind) -> u32 {
    let mut max = 0;
    scan_ids(&root.join(".lmbrain"), kind.prefix(), &mut max);
    max + 1
}

fn scan_ids(dir: &Path, prefix: &str, max: &mut u32) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                scan_ids(&path, prefix, max);
            } else if let Ok(source) = fs::read_to_string(path) {
                if let Ok(document) = Document::parse(&source) {
                    if let Some(id) = document.value("id") {
                        if let Some(number) = numeric_suffix(&id, prefix) {
                            *max = (*max).max(number);
                        }
                    }
                }
            }
        }
    }
}

fn numeric_suffix(id: &str, prefix: &str) -> Option<u32> {
    let suffix = id.strip_prefix(prefix)?.strip_prefix('-')?;
    suffix.parse().ok()
}

fn destination_for(
    kind: ArtifactKind,
    path: &Path,
    target: &str,
) -> Result<PathBuf, TransitionError> {
    if !kind.moves_for_status() {
        return Ok(path.to_path_buf());
    }

    let base = path
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| TransitionError::Missing("status directory".into()))?;

    Ok(base.join(target).join(
        path.file_name()
            .ok_or_else(|| TransitionError::Missing("file name".into()))?,
    ))
}

fn require_force_reason(options: &MutationOptions) -> Result<(), TransitionError> {
    if options.force
        && options
            .reason
            .as_deref()
            .map_or(true, |reason| reason.trim().is_empty())
    {
        Err(TransitionError::MissingForceReason)
    } else {
        Ok(())
    }
}

pub fn kind_for_id(id: &str) -> Option<ArtifactKind> {
    if id.starts_with("MCP-PROP-") {
        Some(ArtifactKind::McpProposal)
    } else if id.starts_with("AGENT-PROP-") {
        Some(ArtifactKind::AgentProposal)
    } else if id.starts_with("SPEC-") {
        Some(ArtifactKind::Spec)
    } else if id.starts_with("REVIEW-") {
        Some(ArtifactKind::Review)
    } else if id.starts_with("ADR-") {
        Some(ArtifactKind::Adr)
    } else if id.starts_with("AGENT-") {
        Some(ArtifactKind::Agent)
    } else if id.starts_with("MCP-") {
        Some(ArtifactKind::Mcp)
    } else if id.starts_with("HANDOFF-") {
        Some(ArtifactKind::Handoff)
    } else {
        None
    }
}

pub fn allowed(kind: ArtifactKind, from: &str, to: &str) -> bool {
    match kind {
        ArtifactKind::Spec => matches!(
            (from, to),
            ("backlog", "ready")
                | ("ready", "working")
                | ("working", "review")
                | ("review", "done")
                | (_, "discarded")
        ),
        ArtifactKind::Review => matches!(
            (from, to),
            ("pending", "accepted")
                | ("pending", "changes-requested")
                | ("pending", "blocked")
                | (_, "superseded")
        ),
        ArtifactKind::Adr => matches!(
            (from, to),
            ("proposed", "accepted")
                | ("proposed", "rejected")
                | ("accepted", "superseded")
                | ("accepted", "deprecated")
        ),
        ArtifactKind::Agent => matches!(
            (from, to),
            ("proposed", "active")
                | ("proposed", "inactive")
                | ("active", "inactive")
                | ("inactive", "active")
                | (_, "retired")
        ),
        ArtifactKind::AgentProposal => matches!(
            (from, to),
            ("proposed", "approved") | ("proposed", "rejected")
        ),
        ArtifactKind::Mcp => matches!(
            (from, to),
            ("specified", "active")
                | ("active", "inactive")
                | ("inactive", "active")
                | (_, "deprecated")
        ),
        ArtifactKind::McpProposal => matches!(
            (from, to),
            ("proposed", "approved")
                | ("proposed", "rejected")
                | ("approved", "implemented")
                | (_, "blocked")
        ),
        ArtifactKind::Handoff => matches!(
            (from, to),
            ("ready", "consumed") | ("ready", "superseded") | (_, "archived")
        ),
    }
}

fn invariant_failure(
    root: &Path,
    path: &Path,
    kind: ArtifactKind,
    target: &str,
    document: &Document,
) -> Option<String> {
    match (kind, target) {
        (ArtifactKind::Spec, "ready")
            if !invariants::recommended_agent_resolves(
                root,
                document.value("recommended_agent").as_deref(),
            ) =>
        {
            Some("recommended_agent does not resolve".into())
        }
        (ArtifactKind::Spec, "done")
            if !invariants::criteria_complete_with_evidence(&document.body) =>
        {
            Some(
                "a done spec requires its acceptance criteria checked and evidence recorded".into(),
            )
        }
        (ArtifactKind::Spec, "done")
            if !invariants::spec_has_accepted_review(
                root,
                &document.value("id").unwrap_or_default(),
            ) =>
        {
            Some("a done spec requires an accepted review".into())
        }
        (ArtifactKind::Handoff, "ready") if !invariants::single_ready_handoff(root, Some(path)) => {
            Some("only one ready handoff is allowed".into())
        }
        _ => None,
    }
}

fn default_status(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Spec => "backlog",
        ArtifactKind::Review => "pending",
        ArtifactKind::Adr
        | ArtifactKind::Agent
        | ArtifactKind::AgentProposal
        | ArtifactKind::McpProposal => "proposed",
        ArtifactKind::Mcp => "specified",
        ArtifactKind::Handoff => "ready",
    }
}

fn template_name(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Spec => "spec.md",
        ArtifactKind::Review => "review.md",
        ArtifactKind::Adr => "adr.md",
        ArtifactKind::Agent => "agent-profile.md",
        ArtifactKind::AgentProposal => "agent-proposal.md",
        ArtifactKind::Mcp => "mcp-spec.md",
        ArtifactKind::McpProposal => "mcp-proposal.md",
        ArtifactKind::Handoff => "session-handoff.md",
    }
}

fn default_template() -> String {
    "---\nid: ID\ntitle: Title\nstatus: STATUS\ncreated: DATE\nupdated: DATE\ntags: []\nlinks: []\n---\n\n# Title\n"
        .into()
}

fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn slug(title: &str) -> String {
    title
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
