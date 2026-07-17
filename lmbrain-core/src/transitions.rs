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
    mutation_lock::ArtifactMutationLock,
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
    Skill,
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
            Self::Skill => "SKILL",
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
            Self::Handoff => "handoffs",
            Self::Skill => "skills",
        }
    }

    fn status_dir(self, status: &str) -> Result<String, TransitionError> {
        match self {
            Self::Handoff => match status {
                "ready" => Ok("active".to_string()),
                "consumed" | "superseded" | "archived" => Ok("archive".to_string()),
                _ => Err(TransitionError::Invariant(format!("invalid handoff status: {status}"))),
            },
            _ => Ok(status.to_string()),
        }
    }

    fn moves_for_status(self) -> bool {
        matches!(self, Self::Spec | Self::Review | Self::Skill | Self::Handoff)
    }

    /// Statuses an artifact may be created with. Only initial lifecycle states
    /// are allowed; anything further must go through governed transitions.
    pub fn creation_statuses(self) -> &'static [&'static str] {
        match self {
            Self::Spec => &["backlog"],
            Self::Review => &["pending"],
            Self::Adr | Self::Agent | Self::AgentProposal | Self::McpProposal | Self::Skill => {
                &["proposed"]
            }
            Self::Mcp => &["specified"],
            Self::Handoff => &["ready"],
        }
    }


}

/// Frontmatter keys owned by the lifecycle engine; callers cannot set them
/// through `CreateRequest::fields`. The artifact kind itself is not listed
/// because it is carried by the allocated ID prefix, never by a field
/// ("kind" remains available as an ordinary domain field, e.g. skill kind).
const RESERVED_CREATION_FIELDS: &[&str] = &["id", "title", "status", "created", "updated", "activity"];

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
    #[error("invalid creation status '{status}' for {kind:?} artifacts; allowed: {allowed}")]
    InvalidCreationStatus {
        kind: ArtifactKind,
        status: String,
        allowed: String,
    },
    #[error("field '{0}' is core-owned lifecycle metadata and cannot be set at creation")]
    ReservedField(String),
    #[error("invalid creation field: {0}")]
    InvalidField(String),
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
    let artifact = artifact.as_ref();
    let path = guard.resolve_existing(artifact)?;
    let initial = Document::parse(&fs::read_to_string(&path)?)?;
    let initial_id = initial
        .value("id")
        .ok_or_else(|| TransitionError::Missing("id".into()))?;
    let _lock = ArtifactMutationLock::acquire(guard.root(), &initial_id)?;
    let path = guard.resolve_existing(artifact)?;
    let current_source = fs::read_to_string(&path)?;
    let mut document = Document::parse(&current_source)?;
    let id = document
        .value("id")
        .ok_or_else(|| TransitionError::Missing("id".into()))?;
    if id != initial_id {
        return Err(TransitionError::Invariant(format!(
            "artifact changed identity while waiting for its mutation lock: expected {initial_id}, found {id}"
        )));
    }
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
    if fs::read_to_string(&path)? != current_source {
        return Err(TransitionError::Invariant(
            "artifact changed while the lifecycle mutation was being prepared".into(),
        ));
    }
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
        None,
        |root| invariants::recommended_agent_resolves(root, Some(agent)),
    )
}

pub fn set_agent_mnemonic_name(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    name: &str,
    options: MutationOptions,
) -> Result<MutationResult, TransitionError> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(TransitionError::Invariant(
            "mnemonic_name cannot be empty".into(),
        ));
    }
    set_field(
        root,
        artifact,
        "mnemonic_name",
        &format!("\"{}\"", trimmed.replace('"', "\\\"")),
        options,
        Some(ArtifactKind::Agent),
        |_| true,
    )
}

fn set_field(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    key: &str,
    value: &str,
    options: MutationOptions,
    expected_kind: Option<ArtifactKind>,
    valid: impl Fn(&Path) -> bool,
) -> Result<MutationResult, TransitionError> {
    require_force_reason(&options)?;

    let guard = PathGuard::new(root)?;
    let artifact = artifact.as_ref();
    let path = guard.resolve_existing(artifact)?;
    let initial = Document::parse(&fs::read_to_string(&path)?)?;
    let initial_id = initial
        .value("id")
        .ok_or_else(|| TransitionError::Missing("id".into()))?;
    let _lock = ArtifactMutationLock::acquire(guard.root(), &initial_id)?;
    let path = guard.resolve_existing(artifact)?;
    let current_source = fs::read_to_string(&path)?;
    let mut document = Document::parse(&current_source)?;
    let id = document
        .value("id")
        .ok_or_else(|| TransitionError::Missing("id".into()))?;
    if id != initial_id {
        return Err(TransitionError::Invariant(format!(
            "artifact changed identity while waiting for its mutation lock: expected {initial_id}, found {id}"
        )));
    }
    if let Some(expected_kind) = expected_kind {
        let actual_kind = kind_for_id(&id)
            .ok_or_else(|| TransitionError::Missing("recognized artifact ID".into()))?;
        if actual_kind != expected_kind {
            return Err(TransitionError::Invariant(format!(
                "expected {expected_kind:?} artifact"
            )));
        }
    }

    if !valid(guard.root()) && !options.force {
        return Err(TransitionError::Invariant(format!("invalid {key}")));
    }

    document.set(key, value);
    document.set("updated", &today());
    document.append_activity(&format!("set {key}"));
    if let Some(reason) = options.reason.as_deref() {
        document.append_override_reason(reason);
    }

    if fs::read_to_string(&path)? != current_source {
        return Err(TransitionError::Invariant(
            "artifact changed while the field mutation was being prepared".into(),
        ));
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

    // Fail closed before any filesystem mutation: an invalid request must not
    // leave directories, files, activity entries, or lock residue behind.
    if !request
        .kind
        .creation_statuses()
        .contains(&status.as_str())
    {
        return Err(TransitionError::InvalidCreationStatus {
            kind: request.kind,
            status,
            allowed: request.kind.creation_statuses().join(", "),
        });
    }
    for (key, value) in &request.fields {
        if RESERVED_CREATION_FIELDS.contains(&key.trim().to_ascii_lowercase().as_str()) {
            return Err(TransitionError::ReservedField(key.clone()));
        }
        if !valid_field_key(key) {
            return Err(TransitionError::InvalidField(format!(
                "field key '{key}' must start with a letter and use only letters, digits, '_' or '-'"
            )));
        }
        if value.contains('\n') || value.contains('\r') {
            return Err(TransitionError::InvalidField(format!(
                "field '{key}' value must not contain line breaks"
            )));
        }
    }

    let mut dir = guard.root().join(".lmbrain").join(request.kind.base());
    if request.kind.moves_for_status() {
        dir = dir.join(request.kind.status_dir(&status)?);
    }

    let _lock = ArtifactMutationLock::acquire(guard.root(), "creation-allocation")?;

    if request.kind == ArtifactKind::Handoff
        && status == "ready"
        && !invariants::single_ready_handoff(guard.root(), None)
    {
        return Err(TransitionError::Invariant(
            "only one ready handoff is allowed".into(),
        ));
    }

    create_locked(guard.root(), &dir, request, status)
}

fn valid_field_key(key: &str) -> bool {
    let mut chars = key.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
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
    if id_exists(root, &id) {
        return Err(TransitionError::Invariant(format!(
            "allocated ID {id} already exists in the workspace"
        )));
    }
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
    // The status directory is only materialized once the request has fully
    // validated, so a rejected create leaves no filesystem residue.
    fs::create_dir_all(dir)?;
    atomic_write(&path, &document.render())?;

    Ok(MutationResult {
        id,
        status,
        path,
        forced: false,
    })
}

fn id_exists(root: &Path, id: &str) -> bool {
    fn scan(dir: &Path, id: &str) -> bool {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if scan(&path, id) {
                        return true;
                    }
                } else if fs::read_to_string(&path)
                    .ok()
                    .and_then(|source| Document::parse(&source).ok())
                    .and_then(|document| document.value("id"))
                    .as_deref()
                    == Some(id)
                {
                    return true;
                }
            }
        }
        false
    }
    scan(&root.join(".lmbrain"), id)
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

    let sub_dir = kind.status_dir(target)?;

    Ok(base.join(sub_dir).join(
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
    } else if id.starts_with("SKILL-") {
        Some(ArtifactKind::Skill)
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
        ArtifactKind::Skill => matches!(
            (from, to),
            ("proposed", "active") | ("proposed", "retired") | ("active", "retired")
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
        (ArtifactKind::Spec, "review") => {
            match crate::verification::transcript_state_for_document(root, document) {
                crate::verification::TranscriptState::Missing => Some(
                    "spec_submit requires ### Verification transcript inside ## Implementation evidence with the exact command and pasted output in a non-empty fenced block".into(),
                ),
                crate::verification::TranscriptState::Empty => Some(
                    "spec_submit requires a non-empty fenced command/result block in ### Verification transcript".into(),
                ),
                crate::verification::TranscriptState::GeneratedStale => Some(
                    "kit-generated verification evidence is stale for the current workspace; run spec_verify again or use an explicitly reasoned force override".into(),
                ),
                crate::verification::TranscriptState::HandAuthored
                | crate::verification::TranscriptState::GeneratedFresh => None,
            }
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
        ArtifactKind::Skill => "proposed",
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
        ArtifactKind::Skill => "skill.md",
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
