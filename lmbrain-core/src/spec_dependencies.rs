use std::{
    collections::{BTreeMap, BTreeSet, HashSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};

use chrono::Local;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use thiserror::Error;

use crate::{
    frontmatter::{atomic_write, Document, FrontmatterError},
    mutation_lock::ArtifactMutationLock,
    path::{PathError, PathGuard},
    transitions::{kind_for_id, ArtifactKind},
};

pub const SPEC_DEPENDENCY_EVENT_SCHEMA_VERSION: &str = "1";
const MAX_TRANSITIVE_DEPENDENCIES: usize = 100;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecDependency {
    pub id: String,
    pub title: String,
    pub status: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecDependencyBlocker {
    pub id: String,
    pub status: String,
    pub chain: Vec<String>,
    pub cause: String,
}

/// A spec file the graph scan could not parse: excluded from the graph but
/// reported explicitly so one corrupted artifact degrades the context read
/// instead of silently shrinking the dependency graph (#85).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MalformedSpec {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecDependencyContext {
    pub source_digest: String,
    pub direct_prerequisites: Vec<SpecDependency>,
    pub direct_dependents: Vec<SpecDependency>,
    pub transitive_prerequisites: Vec<SpecDependency>,
    pub blockers: Vec<SpecDependencyBlocker>,
    pub truncated: bool,
    #[serde(default)]
    pub malformed_specs: Vec<MalformedSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecDependencyMutation {
    pub id: String,
    pub path: PathBuf,
    pub depends_on: Vec<String>,
    pub updated: String,
    pub source_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecDependencyCandidate {
    pub spec_id: String,
    pub prerequisite_id: String,
    pub evidence: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecDependencyCandidateInventory {
    pub candidates: Vec<SpecDependencyCandidate>,
    pub mutated: bool,
}

#[derive(Debug, Clone)]
struct SpecNode {
    id: String,
    title: String,
    status: String,
    path: PathBuf,
    depends_on: Vec<String>,
    dependency_shape_error: Option<String>,
}

#[derive(Debug, Error)]
pub enum SpecDependencyError {
    #[error(transparent)]
    Path(#[from] PathError),
    #[error(transparent)]
    Frontmatter(#[from] FrontmatterError),
    #[error("invalid spec dependencies: {0}")]
    Invalid(String),
    #[error("spec dependency mutation conflict: {0}")]
    Conflict(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub fn spec_dependency_context(
    root: &Path,
    spec_id_or_path: &str,
) -> Result<SpecDependencyContext, SpecDependencyError> {
    let (nodes, malformed_specs) = scan_specs_with_malformed(root)?;
    let id = resolve_spec_id(root, spec_id_or_path)?;
    let node = nodes.get(&id).ok_or_else(|| {
        // If the requested spec exists but cannot parse, name it and the exact
        // parse failure instead of a generic "does not resolve".
        let stem = format!("{id}-");
        match malformed_specs.iter().find(|spec| {
            Path::new(&spec.path)
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(&stem) || name == format!("{id}.md"))
        }) {
            Some(broken) => SpecDependencyError::Invalid(format!(
                "spec '{id}' exists at {} but its frontmatter cannot be parsed ({}); repair it with lmbrain_repair_frontmatter before reading dependency context",
                broken.path, broken.error
            )),
            None => SpecDependencyError::Invalid(format!("spec '{id}' does not resolve")),
        }
    })?;
    let source_digest = crate::content_digest(&fs::read(&node.path)?);

    let direct_prerequisites = node
        .depends_on
        .iter()
        .filter_map(|dependency| nodes.get(dependency))
        .map(|dependency| compact(root, dependency))
        .collect();
    let mut direct_dependents = nodes
        .values()
        .filter(|candidate| candidate.depends_on.contains(&id))
        .map(|dependency| compact(root, dependency))
        .collect::<Vec<_>>();
    direct_dependents.sort_by(|left, right| left.id.cmp(&right.id));

    let (transitive_prerequisites, truncated) = transitive(root, &nodes, &id);
    let blockers = dependency_blockers_from_nodes(&nodes, &id);
    Ok(SpecDependencyContext {
        source_digest,
        direct_prerequisites,
        direct_dependents,
        transitive_prerequisites,
        blockers,
        truncated,
        malformed_specs,
    })
}

pub fn spec_dependency_blockers(root: &Path, document: &Document) -> Vec<SpecDependencyBlocker> {
    let id = document.value("id").unwrap_or_default();
    let mut nodes = match scan_specs(root) {
        Ok(nodes) => nodes,
        Err(error) => {
            return vec![SpecDependencyBlocker {
                id,
                status: "invalid".into(),
                chain: Vec::new(),
                cause: error.to_string(),
            }]
        }
    };
    if let Some(node) = nodes.get_mut(&id) {
        node.depends_on = document.string_array("depends_on");
    }
    dependency_blockers_from_nodes(&nodes, &id)
}

pub fn validate_spec_dependency_graph(root: &Path) -> Vec<(String, String)> {
    let nodes = match scan_specs(root) {
        Ok(nodes) => nodes,
        Err(error) => return vec![("workspace".into(), error.to_string())],
    };
    let mut issues = Vec::new();
    for node in nodes.values() {
        if let Some(error) = node.dependency_shape_error.as_ref() {
            issues.push((node.id.clone(), error.clone()));
        }
        let mut seen = HashSet::new();
        for dependency in &node.depends_on {
            if dependency == &node.id {
                issues.push((node.id.clone(), "depends_on cannot reference itself".into()));
            } else if !seen.insert(dependency) {
                issues.push((
                    node.id.clone(),
                    format!("depends_on contains duplicate '{dependency}'"),
                ));
            } else if !nodes.contains_key(dependency) {
                issues.push((
                    node.id.clone(),
                    format!("depends_on references missing spec '{dependency}'"),
                ));
            }
        }
    }
    for cycle in cycles(&nodes) {
        let message = format!("spec dependency cycle: {}", cycle.join(" -> "));
        for id in cycle.iter().take(cycle.len().saturating_sub(1)) {
            issues.push((id.clone(), message.clone()));
        }
    }
    issues.sort();
    issues.dedup();
    issues
}

pub fn spec_dependency_candidates(
    root: &Path,
) -> Result<SpecDependencyCandidateInventory, SpecDependencyError> {
    let nodes = scan_specs(root)?;
    let phrase = Regex::new(
        r"(?i)(dipendenza\s+dura|hard\s+(dependency|prerequisite)|non\s+iniziare\s+prima|do\s+not\s+start\s+before)",
    )
    .expect("static dependency phrase regex");
    let id_pattern = Regex::new(r"SPEC-\d+").expect("static spec id regex");
    let mut candidates = Vec::new();
    for node in nodes.values() {
        let source = fs::read_to_string(&node.path)?;
        let document = Document::parse(&source)?;
        for line in document.body.lines().filter(|line| phrase.is_match(line)) {
            for matched in id_pattern.find_iter(line) {
                let dependency = matched.as_str();
                if dependency == node.id || node.depends_on.iter().any(|id| id == dependency) {
                    continue;
                }
                candidates.push(SpecDependencyCandidate {
                    spec_id: node.id.clone(),
                    prerequisite_id: dependency.to_string(),
                    evidence: line.trim().chars().take(300).collect(),
                    path: relative_path(root, &node.path),
                });
            }
        }
    }
    candidates.sort_by(|left, right| {
        left.spec_id
            .cmp(&right.spec_id)
            .then_with(|| left.prerequisite_id.cmp(&right.prerequisite_id))
            .then_with(|| left.evidence.cmp(&right.evidence))
    });
    candidates.dedup_by(|left, right| {
        left.spec_id == right.spec_id && left.prerequisite_id == right.prerequisite_id
    });
    Ok(SpecDependencyCandidateInventory {
        candidates,
        mutated: false,
    })
}

pub fn set_spec_dependencies(
    root: &Path,
    artifact: &Path,
    mut depends_on: Vec<String>,
    actor: &str,
    reason: &str,
    expected_digest: &str,
) -> Result<SpecDependencyMutation, SpecDependencyError> {
    require_text("actor", actor)?;
    require_text("reason", reason)?;
    depends_on
        .iter_mut()
        .for_each(|id| *id = id.trim().to_string());

    let guard = PathGuard::new(root)?;
    let path = guard.resolve_existing(artifact)?;
    let initial = Document::parse(&fs::read_to_string(&path)?)?;
    let initial_id = initial
        .value("id")
        .ok_or_else(|| SpecDependencyError::Invalid("artifact is missing id".into()))?;
    let _lock = ArtifactMutationLock::acquire(guard.root(), &initial_id)?;
    let source = fs::read_to_string(&path)?;
    let mut document = Document::parse(&source)?;
    let id = document.value("id").unwrap_or_default();
    if id != initial_id || kind_for_id(&id) != Some(ArtifactKind::Spec) {
        return Err(SpecDependencyError::Invalid(
            "dependency mutation requires a stable SPEC-* artifact".into(),
        ));
    }
    let status = document.value("status").unwrap_or_default();
    if status != "backlog" {
        return Err(SpecDependencyError::Invalid(format!(
            "hard dependencies can only be changed while a spec is in backlog; '{id}' is '{status}'"
        )));
    }
    require_text("expected_digest", expected_digest)?;
    let actual_digest = crate::content_digest(source.as_bytes());
    if actual_digest != expected_digest {
        return Err(SpecDependencyError::Conflict(format!(
            "expected source digest '{expected_digest}', found '{actual_digest}'"
        )));
    }

    let old = document.string_array("depends_on");
    validate_candidate_dependencies(root, &id, &depends_on)?;
    let updated = Local::now().format("%Y-%m-%d").to_string();
    document.set(
        "depends_on",
        &serde_json::to_string(&depends_on).unwrap_or_else(|_| "[]".into()),
    );
    document.set("updated", &updated);
    document.append_activity("updated hard spec dependencies");
    let sequence = document.object_array("dependency_events").len() + 1;
    document.append_object(
        "dependency_events",
        &[
            (
                "schema_version".into(),
                json!(SPEC_DEPENDENCY_EVENT_SCHEMA_VERSION),
            ),
            ("id".into(), json!(format!("{id}-DEPENDENCY-{sequence:03}"))),
            ("timestamp".into(), json!(Local::now().to_rfc3339())),
            ("actor".into(), json!(actor.trim())),
            ("reason".into(), json!(reason.trim())),
            ("previous".into(), json!(old)),
            ("current".into(), json!(depends_on)),
        ],
    )?;
    if fs::read_to_string(&path)? != source {
        return Err(SpecDependencyError::Conflict(
            "artifact changed while dependency mutation was prepared".into(),
        ));
    }
    let rendered = document.render();
    atomic_write(&path, &rendered)?;
    Ok(SpecDependencyMutation {
        id,
        path,
        depends_on,
        updated,
        source_digest: crate::content_digest(rendered.as_bytes()),
    })
}

pub(crate) fn validate_candidate_dependencies(
    root: &Path,
    id: &str,
    depends_on: &[String],
) -> Result<(), SpecDependencyError> {
    let mut nodes = scan_specs(root)?;
    let mut seen = HashSet::new();
    for dependency in depends_on {
        if dependency.is_empty() {
            return Err(SpecDependencyError::Invalid(
                "depends_on entries cannot be empty".into(),
            ));
        }
        if dependency == id {
            return Err(SpecDependencyError::Invalid(
                "depends_on cannot reference itself".into(),
            ));
        }
        if !seen.insert(dependency) {
            return Err(SpecDependencyError::Invalid(format!(
                "depends_on contains duplicate '{dependency}'"
            )));
        }
        if !nodes.contains_key(dependency) {
            return Err(SpecDependencyError::Invalid(format!(
                "depends_on references missing spec '{dependency}'"
            )));
        }
    }
    if let Some(node) = nodes.get_mut(id) {
        node.depends_on = depends_on.to_vec();
    } else {
        nodes.insert(
            id.to_string(),
            SpecNode {
                id: id.to_string(),
                title: String::new(),
                status: "backlog".into(),
                path: PathBuf::new(),
                depends_on: depends_on.to_vec(),
                dependency_shape_error: None,
            },
        );
    }
    if let Some(cycle) = cycles(&nodes)
        .into_iter()
        .find(|cycle| cycle.contains(&id.to_string()))
    {
        return Err(SpecDependencyError::Invalid(format!(
            "dependency update would create cycle: {}",
            cycle.join(" -> ")
        )));
    }
    Ok(())
}

fn dependency_blockers_from_nodes(
    nodes: &BTreeMap<String, SpecNode>,
    id: &str,
) -> Vec<SpecDependencyBlocker> {
    let Some(node) = nodes.get(id) else {
        return Vec::new();
    };
    let mut blockers = Vec::new();
    if let Some(error) = node.dependency_shape_error.as_ref() {
        blockers.push(SpecDependencyBlocker {
            id: id.to_string(),
            status: "malformed".into(),
            chain: vec![id.to_string()],
            cause: error.clone(),
        });
    }
    for dependency in &node.depends_on {
        let mut chain = vec![id.to_string(), dependency.clone()];
        match nodes.get(dependency) {
            None => blockers.push(SpecDependencyBlocker {
                id: dependency.clone(),
                status: "missing".into(),
                chain,
                cause: "hard prerequisite does not resolve".into(),
            }),
            Some(prerequisite) if prerequisite.status != "done" => {
                let cause = if prerequisite.status == "discarded" {
                    "hard prerequisite was discarded; planning must replace or remove it"
                } else {
                    "hard prerequisite is not done"
                };
                blockers.push(SpecDependencyBlocker {
                    id: dependency.clone(),
                    status: prerequisite.status.clone(),
                    chain: chain.clone(),
                    cause: cause.into(),
                });
                append_transitive_blockers(nodes, prerequisite, &mut chain, &mut blockers);
            }
            Some(_) => {}
        }
    }
    blockers.sort_by(|left, right| {
        left.chain
            .cmp(&right.chain)
            .then_with(|| left.id.cmp(&right.id))
    });
    blockers.dedup_by(|left, right| left.id == right.id && left.chain == right.chain);
    blockers
}

fn append_transitive_blockers(
    nodes: &BTreeMap<String, SpecNode>,
    node: &SpecNode,
    chain: &mut Vec<String>,
    blockers: &mut Vec<SpecDependencyBlocker>,
) {
    if chain.len() > MAX_TRANSITIVE_DEPENDENCIES {
        return;
    }
    for dependency in &node.depends_on {
        if chain.contains(dependency) {
            continue;
        }
        chain.push(dependency.clone());
        match nodes.get(dependency) {
            None => blockers.push(SpecDependencyBlocker {
                id: dependency.clone(),
                status: "missing".into(),
                chain: chain.clone(),
                cause: "transitive hard prerequisite does not resolve".into(),
            }),
            Some(prerequisite) if prerequisite.status != "done" => {
                blockers.push(SpecDependencyBlocker {
                    id: dependency.clone(),
                    status: prerequisite.status.clone(),
                    chain: chain.clone(),
                    cause: "transitive hard prerequisite is not done".into(),
                });
                append_transitive_blockers(nodes, prerequisite, chain, blockers);
            }
            Some(_) => {}
        }
        chain.pop();
    }
}

fn transitive(
    root: &Path,
    nodes: &BTreeMap<String, SpecNode>,
    id: &str,
) -> (Vec<SpecDependency>, bool) {
    let mut output = Vec::new();
    let mut seen = HashSet::new();
    let mut queue = VecDeque::new();
    if let Some(node) = nodes.get(id) {
        queue.extend(node.depends_on.iter().cloned());
    }
    let mut truncated = false;
    while let Some(candidate) = queue.pop_front() {
        if !seen.insert(candidate.clone()) {
            continue;
        }
        if output.len() == MAX_TRANSITIVE_DEPENDENCIES {
            truncated = true;
            break;
        }
        if let Some(node) = nodes.get(&candidate) {
            output.push(compact(root, node));
            queue.extend(node.depends_on.iter().cloned());
        }
    }
    (output, truncated)
}

fn cycles(nodes: &BTreeMap<String, SpecNode>) -> Vec<Vec<String>> {
    fn visit(
        id: &str,
        nodes: &BTreeMap<String, SpecNode>,
        visited: &mut HashSet<String>,
        stack: &mut Vec<String>,
        active: &mut HashSet<String>,
        cycles: &mut BTreeSet<Vec<String>>,
    ) {
        if active.contains(id) {
            if let Some(start) = stack.iter().position(|entry| entry == id) {
                let mut cycle = stack[start..].to_vec();
                cycle.push(id.to_string());
                cycles.insert(canonical_cycle(cycle));
            }
            return;
        }
        if !visited.insert(id.to_string()) {
            return;
        }
        active.insert(id.to_string());
        stack.push(id.to_string());
        if let Some(node) = nodes.get(id) {
            for dependency in &node.depends_on {
                if nodes.contains_key(dependency) {
                    visit(dependency, nodes, visited, stack, active, cycles);
                }
            }
        }
        stack.pop();
        active.remove(id);
    }

    let mut visited = HashSet::new();
    let mut active = HashSet::new();
    let mut stack = Vec::new();
    let mut output = BTreeSet::new();
    for id in nodes.keys() {
        visit(
            id,
            nodes,
            &mut visited,
            &mut stack,
            &mut active,
            &mut output,
        );
    }
    output.into_iter().collect()
}

fn canonical_cycle(mut cycle: Vec<String>) -> Vec<String> {
    cycle.pop();
    if cycle.is_empty() {
        return cycle;
    }
    let start = cycle
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| left.cmp(right))
        .map(|(index, _)| index)
        .unwrap_or(0);
    cycle.rotate_left(start);
    cycle.push(cycle[0].clone());
    cycle
}

fn scan_specs(root: &Path) -> Result<BTreeMap<String, SpecNode>, SpecDependencyError> {
    scan_specs_with_malformed(root).map(|(nodes, _)| nodes)
}

fn scan_specs_with_malformed(
    root: &Path,
) -> Result<(BTreeMap<String, SpecNode>, Vec<MalformedSpec>), SpecDependencyError> {
    let base = root.join(".lmbrain/specs");
    let mut nodes = BTreeMap::new();
    let mut malformed = Vec::new();
    if !base.exists() {
        return Ok((nodes, malformed));
    }
    let mut paths = Vec::new();
    for status in fs::read_dir(&base)? {
        let status = status?;
        if !status.path().is_dir() {
            continue;
        }
        for file in fs::read_dir(status.path())? {
            let path = file?.path();
            if path.extension().and_then(|value| value.to_str()) == Some("md") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    for path in paths {
        let source = fs::read_to_string(&path)?;
        let document = match Document::parse(&source) {
            Ok(document) => document,
            Err(error) => {
                malformed.push(MalformedSpec {
                    path: relative_path(root, &path),
                    error: error.to_string(),
                });
                continue;
            }
        };
        let Some(id) = document.value("id") else {
            continue;
        };
        if kind_for_id(&id) != Some(ArtifactKind::Spec) {
            continue;
        }
        nodes.insert(
            id.clone(),
            SpecNode {
                id,
                title: document.value("title").unwrap_or_default(),
                status: document.value("status").unwrap_or_default(),
                path,
                depends_on: document.string_array("depends_on"),
                dependency_shape_error: document
                    .fields()
                    .get("depends_on")
                    .filter(|value| {
                        value
                            .as_array()
                            .map_or(true, |items| items.iter().any(|item| !item.is_string()))
                    })
                    .map(|_| "depends_on must be an array of SPEC-* strings".into()),
            },
        );
    }
    Ok((nodes, malformed))
}

fn resolve_spec_id(root: &Path, value: &str) -> Result<String, SpecDependencyError> {
    if value.starts_with("SPEC-") && !value.contains(['/', '\\']) {
        return Ok(value.to_string());
    }
    let guard = PathGuard::new(root)?;
    let path = guard.resolve_existing(Path::new(value))?;
    let document = Document::parse(&fs::read_to_string(path)?)?;
    document
        .value("id")
        .filter(|id| kind_for_id(id) == Some(ArtifactKind::Spec))
        .ok_or_else(|| SpecDependencyError::Invalid("artifact is not a SPEC-*".into()))
}

fn compact(root: &Path, node: &SpecNode) -> SpecDependency {
    SpecDependency {
        id: node.id.clone(),
        title: node.title.clone(),
        status: node.status.clone(),
        path: relative_path(root, &node.path),
    }
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn require_text(label: &str, value: &str) -> Result<(), SpecDependencyError> {
    if value.trim().is_empty() {
        Err(SpecDependencyError::Invalid(format!(
            "{label} cannot be empty"
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn workspace() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        for status in ["backlog", "ready", "working", "review", "done", "discarded"] {
            fs::create_dir_all(dir.path().join(format!(".lmbrain/specs/{status}"))).unwrap();
        }
        dir
    }

    fn spec(root: &Path, id: &str, status: &str, dependencies: &[&str]) -> PathBuf {
        let path = root.join(format!(
            ".lmbrain/specs/{status}/{id}-{}.md",
            id.to_lowercase()
        ));
        fs::write(
            &path,
            format!(
                "---\nid: {id}\ntitle: {id}\nstatus: {status}\ndepends_on: {}\nupdated: 2026-07-29\nactivity: []\ndependency_events: []\n---\n# {id}\n",
                serde_json::to_string(dependencies).unwrap()
            ),
        )
        .unwrap();
        path
    }

    #[test]
    fn one_malformed_spec_degrades_gracefully_and_is_named_when_requested() {
        let dir = workspace();
        spec(dir.path(), "SPEC-001", "backlog", &[]);
        spec(dir.path(), "SPEC-002", "backlog", &["SPEC-001"]);
        // The 4.0.1 field corruption: duplicate top-level activity keys.
        fs::write(
            dir.path()
                .join(".lmbrain/specs/backlog/SPEC-003-spec-003.md"),
            "---\nid: SPEC-003\ntitle: Corrupted\nstatus: backlog\ndepends_on: []\nactivity:\n  - date: 2026-08-06\n    action: \"created\"\nactivity:\n  - date: 2026-08-06\n    action: \"set effort\"\n---\n# Corrupted\n",
        )
        .unwrap();

        // Reading a healthy spec keeps working and reports the corrupted file.
        let context = spec_dependency_context(dir.path(), "SPEC-002").unwrap();
        assert_eq!(context.direct_prerequisites[0].id, "SPEC-001");
        assert_eq!(context.malformed_specs.len(), 1);
        assert!(context.malformed_specs[0].path.contains("SPEC-003"));
        assert!(context.malformed_specs[0].error.contains("duplicate"));

        // Requesting the corrupted spec names the file, the parse failure,
        // and the repair verb instead of a generic resolution error.
        let error = spec_dependency_context(dir.path(), "SPEC-003").unwrap_err();
        let message = error.to_string();
        assert!(message.contains("SPEC-003"), "{message}");
        assert!(message.contains("lmbrain_repair_frontmatter"), "{message}");
    }

    #[test]
    fn xenomark_shaped_chain_blocks_in_order_and_unblocks_when_done() {
        let dir = workspace();
        spec(dir.path(), "SPEC-055", "done", &[]);
        spec(dir.path(), "SPEC-057", "done", &["SPEC-055"]);
        spec(dir.path(), "SPEC-058", "working", &["SPEC-057"]);
        spec(dir.path(), "SPEC-054", "backlog", &["SPEC-058"]);
        let context = spec_dependency_context(dir.path(), "SPEC-054").unwrap();
        assert_eq!(context.direct_prerequisites[0].id, "SPEC-058");
        assert_eq!(context.blockers[0].id, "SPEC-058");
        assert_eq!(context.transitive_prerequisites.len(), 3);
    }

    #[test]
    fn managed_update_rejects_missing_duplicate_self_cycle_and_stale_write() {
        let dir = workspace();
        let first = spec(dir.path(), "SPEC-001", "backlog", &[]);
        spec(dir.path(), "SPEC-002", "backlog", &["SPEC-001"]);
        let initial_digest = crate::content_digest(&fs::read(&first).unwrap());
        for invalid in [
            vec!["SPEC-404".into()],
            vec!["SPEC-001".into()],
            vec!["SPEC-002".into(), "SPEC-002".into()],
            vec!["SPEC-002".into()],
        ] {
            assert!(set_spec_dependencies(
                dir.path(),
                &first,
                invalid,
                "AGENT-LEAD",
                "Planning update",
                &initial_digest
            )
            .is_err());
        }
        let result = set_spec_dependencies(
            dir.path(),
            &first,
            Vec::new(),
            "AGENT-LEAD",
            "Explicitly independent",
            &initial_digest,
        )
        .unwrap();
        assert!(set_spec_dependencies(
            dir.path(),
            &first,
            Vec::new(),
            "AGENT-LEAD",
            "Stale write",
            &initial_digest
        )
        .is_err());
        let document = Document::parse(&fs::read_to_string(result.path).unwrap()).unwrap();
        assert_eq!(document.object_array("dependency_events").len(), 1);
    }

    #[test]
    fn explicit_legacy_prose_is_suggested_without_becoming_a_constraint() {
        let dir = workspace();
        spec(dir.path(), "SPEC-055", "done", &[]);
        let path = spec(dir.path(), "SPEC-057", "backlog", &[]);
        let mut source = fs::read_to_string(&path).unwrap();
        source.push_str("\n**Dipendenza dura da [[SPEC-055]].** Non iniziare prima.\n");
        fs::write(&path, source).unwrap();
        let candidates = spec_dependency_candidates(dir.path()).unwrap();
        assert!(!candidates.mutated);
        assert_eq!(candidates.candidates.len(), 1);
        assert_eq!(candidates.candidates[0].prerequisite_id, "SPEC-055");
        assert!(spec_dependency_context(dir.path(), "SPEC-057")
            .unwrap()
            .direct_prerequisites
            .is_empty());
    }

    #[test]
    fn malformed_dependency_field_fails_closed_with_artifact_identity() {
        let dir = workspace();
        let path = spec(dir.path(), "SPEC-060", "backlog", &[]);
        fs::write(
            path,
            "---\nid: SPEC-060\ntitle: Broken\nstatus: backlog\ndepends_on: SPEC-055\n---\n",
        )
        .unwrap();
        let issues = validate_spec_dependency_graph(dir.path());
        assert_eq!(issues[0].0, "SPEC-060");
        assert!(issues[0].1.contains("must be an array"));
        let context = spec_dependency_context(dir.path(), "SPEC-060").unwrap();
        assert_eq!(context.blockers[0].status, "malformed");
    }
}
