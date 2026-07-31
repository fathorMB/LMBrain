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
    review::{
        next_review_event_id, parse_review_event_history, ReviewEventInput,
        REVIEW_EVENT_SCHEMA_VERSION,
    },
    taxonomy::{normalize_finding_category, FINDING_TAXONOMY_VERSION},
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
    Finding,
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
            Self::Finding => "FINDING",
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
            Self::Finding => "findings",
        }
    }

    fn status_dir(self, status: &str) -> Result<String, TransitionError> {
        match self {
            Self::Handoff => match status {
                "ready" => Ok("active".to_string()),
                "consumed" | "superseded" | "archived" => Ok("archive".to_string()),
                _ => Err(TransitionError::Invariant(format!(
                    "invalid handoff status: {status}"
                ))),
            },
            _ => Ok(status.to_string()),
        }
    }

    fn moves_for_status(self) -> bool {
        matches!(
            self,
            Self::Spec | Self::Review | Self::Skill | Self::Handoff | Self::Finding
        )
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
            Self::Finding => &["open"],
        }
    }
}

/// Frontmatter keys owned by the lifecycle engine; callers cannot set them
/// through `CreateRequest::fields`. The artifact kind itself is not listed
/// because it is carried by the allocated ID prefix, never by a field
/// ("kind" remains available as an ordinary domain field, e.g. skill kind).
const RESERVED_CREATION_FIELDS: &[&str] = &[
    "id",
    "title",
    "status",
    "created",
    "updated",
    "activity",
    "review_events",
    "mutation_overrides",
    "dependency_events",
    "parking_events",
    "finding_taxonomy_version",
];

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
pub struct SpecParkingInput {
    pub actor: String,
    pub reason: String,
    pub revisit_condition: Option<String>,
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
    transition_internal(root, artifact, target, options, None, None)
}

pub fn review_verdict(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    target: &str,
    event: ReviewEventInput,
    options: MutationOptions,
) -> Result<MutationResult, TransitionError> {
    if event.actor_role.trim().is_empty() {
        return Err(TransitionError::Invariant(
            "review verdict actor_role cannot be empty".into(),
        ));
    }
    if target != "accepted" && event.reason.trim().is_empty() {
        return Err(TransitionError::Invariant(format!(
            "review verdict '{target}' requires a non-empty reason"
        )));
    }
    let expected_actor = if target == "accepted" {
        "operator"
    } else {
        "project-lead"
    };
    if event.actor_role.trim() != expected_actor {
        return Err(TransitionError::Invariant(format!(
            "review verdict '{target}' requires actor_role '{expected_actor}'"
        )));
    }
    transition_internal(root, artifact, target, options, Some(event), None)
}

pub fn park_spec(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    input: SpecParkingInput,
) -> Result<MutationResult, TransitionError> {
    if input.actor.trim().is_empty() {
        return Err(TransitionError::Invariant(
            "spec parking requires a non-empty actor".into(),
        ));
    }
    if input.reason.trim().is_empty() {
        return Err(TransitionError::Invariant(
            "spec parking requires a non-empty reason".into(),
        ));
    }
    if input
        .revisit_condition
        .as_deref()
        .is_some_and(|value| value.trim().is_empty())
    {
        return Err(TransitionError::Invariant(
            "revisit_condition cannot be empty when provided".into(),
        ));
    }
    transition_internal(
        root,
        artifact,
        "backlog",
        MutationOptions::default(),
        None,
        Some(input),
    )
}

pub fn record_review_event(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    action: &str,
    event: ReviewEventInput,
    options: MutationOptions,
) -> Result<MutationResult, TransitionError> {
    require_force_reason(&options)?;
    if !matches!(
        action,
        "remediation" | "remediation-verification" | "escalation" | "takeover"
    ) {
        return Err(TransitionError::Invariant(format!(
            "unsupported review lifecycle event '{action}'"
        )));
    }
    if event.reason.trim().is_empty() {
        return Err(TransitionError::Invariant(format!(
            "review lifecycle event '{action}' requires a non-empty reason"
        )));
    }
    let expected_actor = match action {
        "remediation" => "implementation-specialist",
        "remediation-verification" => "project-lead",
        "escalation" => "operator",
        "takeover" => "project-lead",
        _ => unreachable!(),
    };
    if event.actor_role.trim() != expected_actor {
        return Err(TransitionError::Invariant(format!(
            "review lifecycle event '{action}' requires actor_role '{expected_actor}'"
        )));
    }
    if action == "remediation"
        && event
            .remediation_agent
            .as_deref()
            .map_or(true, |agent| agent.trim().is_empty())
    {
        return Err(TransitionError::Invariant(
            "review remediation requires remediation_agent".into(),
        ));
    }

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
    if id != initial_id || kind_for_id(&id) != Some(ArtifactKind::Review) {
        return Err(TransitionError::Invariant(
            "review lifecycle events require a stable REVIEW-* artifact".into(),
        ));
    }
    validate_existing_review_history(&document)?;
    let status = document
        .value("status")
        .ok_or_else(|| TransitionError::Missing("status".into()))?;
    if status == "superseded" {
        return Err(TransitionError::Invariant(
            "cannot append lifecycle events to a superseded review".into(),
        ));
    }
    if action == "remediation-verification" {
        if event.evidence_refs.is_empty()
            || event
                .evidence_refs
                .iter()
                .any(|reference| reference.trim().is_empty())
        {
            return Err(TransitionError::Invariant(
                "review remediation verification requires non-empty evidence_refs".into(),
            ));
        }
        let history = parse_review_event_history(&document);
        let previous = history.events.last().ok_or_else(|| {
            TransitionError::Invariant(
                "review remediation verification requires a preceding remediation event".into(),
            )
        })?;
        if previous.action != "remediation" {
            return Err(TransitionError::Invariant(
                "review remediation verification must immediately follow a remediation event"
                    .into(),
            ));
        }
    }
    document.set("updated", &today());
    document.append_activity(&format!("recorded review {action}"));
    append_review_event(&mut document, &id, action, &status, &status, &event)?;
    if let Some(reason) = options.reason.as_deref() {
        document.append_override_reason(reason);
    }
    if fs::read_to_string(&path)? != current_source {
        return Err(TransitionError::Invariant(
            "artifact changed while the lifecycle mutation was being prepared".into(),
        ));
    }
    atomic_write(&path, &document.render())?;
    Ok(MutationResult {
        id,
        status,
        path,
        forced: options.force,
    })
}

fn transition_internal(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    target: &str,
    options: MutationOptions,
    review_event: Option<ReviewEventInput>,
    parking_event: Option<SpecParkingInput>,
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

    if kind == ArtifactKind::Review && review_event.is_none() {
        return Err(TransitionError::Invariant(
            "review lifecycle changes require review_verdict event metadata".into(),
        ));
    }
    if kind == ArtifactKind::Finding {
        return Err(TransitionError::Invariant(
            "finding lifecycle changes require a semantic finding operation".into(),
        ));
    }
    if parking_event.is_some()
        && (kind != ArtifactKind::Spec || from != "ready" || target != "backlog")
    {
        return Err(TransitionError::Invariant(format!(
            "spec parking only permits ready -> backlog; found {kind:?} {from} -> {target}"
        )));
    }
    if kind == ArtifactKind::Review {
        validate_existing_review_history(&document)?;
    }

    if !allowed(kind, &from, target) && parking_event.is_none() {
        return Err(TransitionError::Illegal {
            kind,
            from,
            to: target.into(),
        });
    }

    let invariant_message = invariant_failure(guard.root(), &path, kind, target, &document);
    if let Some(message) = invariant_message.as_deref() {
        if !options.force {
            return Err(TransitionError::Invariant(message.into()));
        }
    }

    document.set("status", target);
    document.set("updated", &today());
    document.append_activity(&format!("transitioned {from} -> {target}"));
    if let Some(event) = parking_event.as_ref() {
        append_spec_parking_event(&mut document, &id, event)?;
    }
    if options.force {
        if let (Some(reason), Some(invariant)) =
            (options.reason.as_deref(), invariant_message.as_deref())
        {
            let actor_role = review_event
                .as_ref()
                .map(|event| event.actor_role.as_str())
                .unwrap_or_else(|| transition_authority(kind, target));
            append_mutation_override(
                &mut document,
                &id,
                &from,
                target,
                actor_role,
                reason,
                invariant,
            )?;
        }
    }
    if kind == ArtifactKind::Review {
        let event = review_event.unwrap_or_else(|| ReviewEventInput {
            actor_role: "controlled-mutation".into(),
            reason: options.reason.clone().unwrap_or_default(),
            evidence_refs: Vec::new(),
            remediation_agent: None,
        });
        append_review_event(&mut document, &id, "verdict", &from, target, &event)?;
    }
    if let Some(reason) = options.reason.as_deref() {
        let audit = invariant_message
            .as_deref()
            .map(|invariant| format!("{reason}\n\nUnmet invariant: {invariant}"))
            .unwrap_or_else(|| reason.to_string());
        document.append_override_reason(&audit);
    }

    let destination = destination_for(kind, &path, target)?;
    if destination != path && destination.exists() {
        return Err(TransitionError::Invariant(format!(
            "destination already contains an artifact named {}",
            destination
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("review artifact")
        )));
    }
    if fs::read_to_string(&path)? != current_source {
        return Err(TransitionError::Invariant(
            "artifact changed while the lifecycle mutation was being prepared".into(),
        ));
    }
    if destination == path {
        atomic_write(&path, &document.render())?;
    } else {
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        atomic_write(&path, &document.render())?;
        if let Err(move_error) = fs::rename(&path, &destination) {
            if let Err(rollback_error) = atomic_write(&path, &current_source) {
                return Err(TransitionError::Invariant(format!(
                    "artifact move failed ({move_error}); rollback also failed ({rollback_error}); one source artifact remains and diagnostics can reconcile its status/folder"
                )));
            }
            return Err(TransitionError::Io(move_error));
        }
    }

    Ok(MutationResult {
        id,
        status: target.into(),
        path: destination,
        forced: options.force,
    })
}

fn transition_authority(kind: ArtifactKind, target: &str) -> &'static str {
    match (kind, target) {
        (ArtifactKind::Spec, "ready") => "operator",
        (ArtifactKind::Spec, "working" | "review") => "implementation-specialist",
        (ArtifactKind::Spec, "done" | "discarded") => "project-lead",
        (ArtifactKind::Review, "accepted") => "operator",
        (ArtifactKind::Review, _) => "project-lead",
        _ => "controlled-mutation",
    }
}

fn append_spec_parking_event(
    document: &mut Document,
    spec_id: &str,
    input: &SpecParkingInput,
) -> Result<(), TransitionError> {
    let sequence = document.object_array("parking_events").len() + 1;
    let mut fields = vec![
        ("schema_version".into(), serde_json::json!("1")),
        (
            "id".into(),
            serde_json::json!(format!("{spec_id}-PARKING-{sequence:03}")),
        ),
        (
            "timestamp".into(),
            serde_json::json!(Local::now().to_rfc3339()),
        ),
        ("actor".into(), serde_json::json!(input.actor.trim())),
        ("from_status".into(), serde_json::json!("ready")),
        ("to_status".into(), serde_json::json!("backlog")),
        ("reason".into(), serde_json::json!(input.reason.trim())),
        ("readiness_invalidated".into(), serde_json::json!(true)),
    ];
    if let Some(revisit) = input.revisit_condition.as_deref() {
        fields.push((
            "revisit_condition".into(),
            serde_json::json!(revisit.trim()),
        ));
    }
    document.append_object("parking_events", &fields)?;
    Ok(())
}

fn append_mutation_override(
    document: &mut Document,
    artifact_id: &str,
    from: &str,
    to: &str,
    actor_role: &str,
    reason: &str,
    invariant: &str,
) -> Result<(), TransitionError> {
    let sequence = document.object_array("mutation_overrides").len() + 1;
    document.append_object(
        "mutation_overrides",
        &[
            ("schema_version".into(), serde_json::json!("1")),
            (
                "id".into(),
                serde_json::json!(format!("{artifact_id}-OVERRIDE-{sequence:03}")),
            ),
            ("actor_role".into(), serde_json::json!(actor_role)),
            (
                "timestamp".into(),
                serde_json::json!(Local::now().to_rfc3339()),
            ),
            ("from".into(), serde_json::json!(from)),
            ("to".into(), serde_json::json!(to)),
            ("reason".into(), serde_json::json!(reason.trim())),
            ("unmet_invariant".into(), serde_json::json!(invariant)),
        ],
    )?;
    Ok(())
}

fn validate_existing_review_history(document: &Document) -> Result<(), TransitionError> {
    let fields = document.fields();
    if fields
        .get("review_events")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|events| !events.is_empty())
    {
        let history = parse_review_event_history(document);
        if !history.warnings.is_empty() {
            return Err(TransitionError::Invariant(format!(
                "review lifecycle history is invalid: {}",
                history.warnings.join(" ")
            )));
        }
    }
    Ok(())
}

fn append_review_event(
    document: &mut Document,
    review_id: &str,
    action: &str,
    from: &str,
    target: &str,
    input: &ReviewEventInput,
) -> Result<(), TransitionError> {
    let event_id = next_review_event_id(document, review_id);
    let mut fields = vec![
        (
            "schema_version".into(),
            serde_json::Value::String(REVIEW_EVENT_SCHEMA_VERSION.into()),
        ),
        ("id".into(), serde_json::Value::String(event_id)),
        (
            "timestamp".into(),
            serde_json::Value::String(Local::now().to_rfc3339()),
        ),
        (
            "action".into(),
            serde_json::Value::String(action.to_owned()),
        ),
        ("from_status".into(), serde_json::Value::String(from.into())),
        ("to_status".into(), serde_json::Value::String(target.into())),
        (
            "actor_role".into(),
            serde_json::Value::String(input.actor_role.trim().into()),
        ),
        (
            "reason".into(),
            serde_json::Value::String(input.reason.trim().into()),
        ),
    ];
    if !input.evidence_refs.is_empty() {
        fields.push((
            "evidence_refs".into(),
            serde_json::Value::Array(
                input
                    .evidence_refs
                    .iter()
                    .map(|reference| serde_json::Value::String(reference.trim().into()))
                    .collect(),
            ),
        ));
    }
    if let Some(agent) = document
        .value("implementation_agent")
        .filter(|agent| !agent.trim().is_empty())
    {
        fields.push((
            "implementation_agent".into(),
            serde_json::Value::String(agent),
        ));
    }
    if let Some(agent) = input
        .remediation_agent
        .as_deref()
        .filter(|agent| !agent.trim().is_empty())
    {
        fields.push((
            "remediation_agent".into(),
            serde_json::Value::String(agent.trim().into()),
        ));
    }
    document.append_object("review_events", &fields)?;
    Ok(())
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
    if request.kind == ArtifactKind::Finding {
        return Err(TransitionError::Invariant(
            "findings require the semantic finding_create operation".into(),
        ));
    }
    let guard = PathGuard::new(root)?;
    let status = request
        .status
        .clone()
        .unwrap_or_else(|| default_status(request.kind).into());

    // Fail closed before any filesystem mutation: an invalid request must not
    // leave directories, files, activity entries, or lock residue behind.
    if !request.kind.creation_statuses().contains(&status.as_str()) {
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
    if request.kind == ArtifactKind::Spec {
        crate::spec_dependencies::validate_candidate_dependencies(
            root,
            &id,
            &document.string_array("depends_on"),
        )
        .map_err(|error| TransitionError::InvalidField(error.to_string()))?;
    }
    if request.kind == ArtifactKind::Review {
        for category in document.string_array("finding_categories") {
            let normalized = normalize_finding_category(&category);
            match normalized.canonical {
                None => {
                    return Err(TransitionError::InvalidField(format!(
                        "unknown finding category '{category}'; use a canonical taxonomy value"
                    )));
                }
                Some(canonical) if normalized.is_alias => {
                    return Err(TransitionError::InvalidField(format!(
                        "finding category '{category}' is a legacy alias; new reviews must use '{canonical}'"
                    )));
                }
                Some(_) => {}
            }
        }
        document.set("finding_taxonomy_version", FINDING_TAXONOMY_VERSION);
    }
    document.append_activity("created");
    if request.kind == ArtifactKind::Review {
        append_review_event(
            &mut document,
            &id,
            "submitted",
            "none",
            "pending",
            &ReviewEventInput {
                actor_role: "project-lead".into(),
                reason: "review artifact created".into(),
                evidence_refs: Vec::new(),
                remediation_agent: None,
            },
        )?;
    }
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
    } else if id.starts_with("FINDING-") {
        Some(ArtifactKind::Finding)
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
                | ("changes-requested", "changes-requested")
                | ("changes-requested", "accepted")
                | ("changes-requested", "blocked")
                | ("blocked", "blocked")
                | ("blocked", "changes-requested")
                | ("blocked", "accepted")
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
        ArtifactKind::Finding => matches!(
            (from, to),
            ("open", "planned")
                | ("open", "deferred")
                | ("open", "resolved")
                | ("open", "accepted-risk")
                | ("open", "superseded")
                | ("planned", "open")
                | ("planned", "deferred")
                | ("planned", "resolved")
                | ("planned", "accepted-risk")
                | ("planned", "superseded")
                | ("deferred", "open")
                | ("deferred", "planned")
                | ("deferred", "resolved")
                | ("deferred", "accepted-risk")
                | ("deferred", "superseded")
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
    if kind == ArtifactKind::Spec && matches!(target, "ready" | "working") {
        let blockers = crate::spec_dependency_blockers(root, document);
        if !blockers.is_empty() {
            return Some(format!(
                "hard spec prerequisites are not complete: {}",
                blockers
                    .iter()
                    .map(|blocker| format!(
                        "{} [{}] via {}: {}",
                        blocker.id,
                        blocker.status,
                        blocker.chain.join(" -> "),
                        blocker.cause
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
    }
    if kind == ArtifactKind::Spec && matches!(target, "review" | "done") {
        let phase = if target == "review" {
            "before-submit"
        } else {
            "before-done"
        };
        let blockers = crate::verification_blockers_for_workspace(root, document, phase);
        if !blockers.is_empty() {
            return Some(format!(
                "{phase} verification blocked: {}",
                blockers
                    .iter()
                    .map(|blocker| format!(
                        "{} (owner={}): {}",
                        blocker.requirement_id, blocker.owner, blocker.cause
                    ))
                    .collect::<Vec<_>>()
                    .join("; ")
            ));
        }
    }
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
            if invariants::waived_findings_are_valid(root, &document.body).is_err() =>
        {
            Some(invariants::waived_findings_are_valid(root, &document.body).unwrap_err())
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
        ArtifactKind::Finding => "open",
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
        ArtifactKind::Finding => "finding.md",
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
