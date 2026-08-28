//! Experimental, opt-in decision maps. These artifacts deliberately sit beside
//! the delivery board: they shape a route to a spec and never authorize work.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use chrono::Local;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    frontmatter::{atomic_write, Document, FrontmatterError},
    mutation_lock::WorkspaceLock,
    path::PathGuard,
    transitions::MutationResult,
};

pub const WAYFINDER_EVENT_SCHEMA_VERSION: &str = "1";
const MAP_STATUSES: &[&str] = &[
    "draft",
    "active",
    "cleared",
    "superseded",
    "abandoned",
    "archived",
];
const TICKET_STATUSES: &[&str] = &["open", "claimed", "resolved", "out_of_scope", "superseded"];
const TICKET_TYPES: &[&str] = &["conversation", "prototype", "research", "prerequisite"];

#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
pub struct WayfinderFog {
    pub id: String,
    pub summary: String,
    pub provenance: String,
    pub state: String,
    pub graduated_to: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
pub struct WayfinderMap {
    pub id: String,
    pub title: String,
    pub status: String,
    pub destination: String,
    pub fog: Vec<WayfinderFog>,
    pub path: String,
    pub updated: String,
    pub malformed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
pub struct WayfinderTicket {
    pub id: String,
    pub map: String,
    pub title: String,
    pub status: String,
    pub ticket_type: String,
    pub question: String,
    pub blockers: Vec<String>,
    pub claimed_by: Option<String>,
    pub claimed_at: Option<String>,
    pub resolution_summary: Option<String>,
    pub path: String,
    pub malformed: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
pub struct WayfinderOverview {
    pub maps: Vec<WayfinderMapSummary>,
}
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
pub struct WayfinderMapSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub destination: String,
    pub frontier_count: usize,
    pub claimed_count: usize,
    pub blocked_count: usize,
    pub fog_count: usize,
    pub resolved_count: usize,
    pub updated: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, ts_rs::TS)]
pub struct WayfinderMapContext {
    pub map: WayfinderMap,
    pub frontier: Vec<WayfinderTicket>,
    pub blocked: Vec<WayfinderTicket>,
    pub claimed: Vec<WayfinderTicket>,
    pub resolved: Vec<WayfinderTicket>,
    pub diagnostics: Vec<String>,
    pub omitted_ticket_count: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WayfinderMapCreate {
    pub title: String,
    pub destination: String,
    pub notes: String,
    pub out_of_scope: String,
    pub actor: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WayfinderTicketCreate {
    pub map: String,
    pub title: String,
    pub ticket_type: String,
    pub question: String,
    #[serde(default)]
    pub blockers: Vec<String>,
    pub bounded_context: String,
    pub actor: String,
}

#[derive(Debug, Error)]
pub enum WayfinderError {
    #[error(transparent)]
    Path(#[from] crate::path::PathError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Frontmatter(#[from] FrontmatterError),
    #[error("invalid Wayfinder artifact: {0}")]
    Invalid(String),
    #[error("artifact not found: {0}")]
    NotFound(String),
}

pub fn overview(root: impl AsRef<Path>) -> WayfinderOverview {
    let root = root.as_ref();
    let maps = list_maps(root);
    let tickets = list_tickets(root);
    WayfinderOverview {
        maps: maps
            .into_iter()
            .map(|map| {
                let (frontier, blocked, claimed) = classified(&tickets, &map.id);
                let resolved_count = tickets
                    .iter()
                    .filter(|ticket| ticket.map == map.id && ticket.status == "resolved")
                    .count();
                WayfinderMapSummary {
                    id: map.id,
                    title: map.title,
                    status: map.status,
                    destination: map.destination,
                    fog_count: map.fog.iter().filter(|fog| fog.state == "active").count(),
                    resolved_count,
                    frontier_count: frontier.len(),
                    blocked_count: blocked.len(),
                    claimed_count: claimed.len(),
                    updated: map.updated,
                }
            })
            .collect(),
    }
}

pub fn map_context(
    root: impl AsRef<Path>,
    map_id: &str,
) -> Result<WayfinderMapContext, WayfinderError> {
    let root = root.as_ref();
    let map = list_maps(root)
        .into_iter()
        .find(|map| map.id == map_id)
        .ok_or_else(|| WayfinderError::NotFound(map_id.into()))?;
    let tickets = list_tickets(root);
    let (frontier, blocked, claimed) = classified(&tickets, map_id);
    let resolved = tickets
        .iter()
        .filter(|ticket| {
            ticket.map == map_id
                && matches!(
                    ticket.status.as_str(),
                    "resolved" | "out_of_scope" | "superseded"
                )
        })
        .cloned()
        .collect();
    let diagnostics = validate_map(&map, &tickets);
    Ok(WayfinderMapContext {
        map,
        frontier,
        blocked,
        claimed,
        resolved,
        diagnostics,
        omitted_ticket_count: 0,
    })
}

pub fn list_maps(root: impl AsRef<Path>) -> Vec<WayfinderMap> {
    let mut items = scan(
        root.as_ref().join(".lmbrain/wayfinder/maps"),
        "MAP",
        parse_map,
        malformed_map,
    );
    items.sort_by(|a, b| a.id.cmp(&b.id));
    items
}
pub fn list_tickets(root: impl AsRef<Path>) -> Vec<WayfinderTicket> {
    let mut items = scan(
        root.as_ref().join(".lmbrain/wayfinder/tickets"),
        "WAY",
        parse_ticket,
        malformed_ticket,
    );
    items.sort_by(|a, b| a.id.cmp(&b.id));
    items
}

pub fn create_map(
    root: impl AsRef<Path>,
    input: WayfinderMapCreate,
) -> Result<MutationResult, WayfinderError> {
    required(&[
        ("title", &input.title),
        ("destination", &input.destination),
        ("actor", &input.actor),
    ])?;
    let guard = PathGuard::new(root)?;
    let _lock = WorkspaceLock::acquire(guard.root())?;
    let id = next_id(guard.root(), "MAP");
    let date = today();
    let path = guard
        .root()
        .join(".lmbrain/wayfinder/maps")
        .join(format!("{id}.md"));
    if path.exists() {
        return Err(WayfinderError::Invalid(format!(
            "{id} already exists; refusing to overwrite"
        )));
    }
    let content = format!("---\nid: {id}\ntitle: {}\nstatus: draft\ndestination: {}\nnotes: {}\nout_of_scope: {}\nfog: []\ncreated: {date}\nupdated: {date}\nwayfinder_events: [{}]\n---\n# {}\n", q(&input.title), q(&input.destination), q(&input.notes), q(&input.out_of_scope), event("created", &input.actor), input.title.trim());
    atomic_write(&path, &content)?;
    Ok(result(id, "draft", guard.root(), &path))
}

pub fn create_ticket(
    root: impl AsRef<Path>,
    input: WayfinderTicketCreate,
) -> Result<MutationResult, WayfinderError> {
    required(&[
        ("map", &input.map),
        ("title", &input.title),
        ("question", &input.question),
        ("actor", &input.actor),
    ])?;
    if !TICKET_TYPES.contains(&input.ticket_type.as_str()) {
        return Err(WayfinderError::Invalid(
            "ticket_type must be conversation, prototype, research, or prerequisite".into(),
        ));
    }
    let guard = PathGuard::new(root)?;
    let _lock = WorkspaceLock::acquire(guard.root())?;
    let maps = list_maps(guard.root());
    if !maps.iter().any(|map| map.id == input.map) {
        return Err(WayfinderError::NotFound(input.map));
    }
    let tickets = list_tickets(guard.root());
    validate_blockers(&input.map, &input.blockers, &tickets)?;
    let id = next_id(guard.root(), "WAY");
    let date = today();
    let path = guard
        .root()
        .join(".lmbrain/wayfinder/tickets")
        .join(format!("{id}.md"));
    if path.exists() {
        return Err(WayfinderError::Invalid(format!(
            "{id} already exists; refusing to overwrite"
        )));
    }
    let content = format!("---\nid: {id}\ntitle: {}\nstatus: open\nmap: {}\nticket_type: {}\nquestion: {}\nblockers: {}\nclaimed_by: null\nclaimed_at: null\nresolution_summary: null\ncreated: {date}\nupdated: {date}\nwayfinder_events: [{}]\n---\n# {}\n\n## Bounded context\n\n{}\n", q(&input.title), q(&input.map), q(&input.ticket_type), q(&input.question), array(&input.blockers), event("created", &input.actor), input.title.trim(), input.bounded_context.trim());
    atomic_write(&path, &content)?;
    Ok(result(id, "open", guard.root(), &path))
}

pub fn claim_ticket(
    root: impl AsRef<Path>,
    ticket_id: &str,
    claimant: &str,
) -> Result<MutationResult, WayfinderError> {
    mutate_ticket(root, ticket_id, claimant, |doc| {
        if doc.value("status").as_deref() != Some("open") {
            return Err(WayfinderError::Invalid(
                "only an open frontier ticket can be claimed".into(),
            ));
        }
        doc.set("status", "claimed");
        doc.set("claimed_by", &q(claimant));
        doc.set("claimed_at", &q(&Local::now().to_rfc3339()));
        Ok("claimed")
    })
}
pub fn release_ticket(
    root: impl AsRef<Path>,
    ticket_id: &str,
    actor: &str,
) -> Result<MutationResult, WayfinderError> {
    mutate_ticket(root, ticket_id, actor, |doc| {
        if doc.value("status").as_deref() != Some("claimed") {
            return Err(WayfinderError::Invalid(
                "only a claimed ticket can be released".into(),
            ));
        }
        doc.set("status", "open");
        doc.set("claimed_by", "null");
        doc.set("claimed_at", "null");
        Ok("open")
    })
}
pub fn resolve_ticket(
    root: impl AsRef<Path>,
    ticket_id: &str,
    actor: &str,
    summary: &str,
    evidence: &str,
    operator_evidence: Option<&str>,
) -> Result<MutationResult, WayfinderError> {
    required(&[
        ("actor", actor),
        ("summary", summary),
        ("evidence", evidence),
    ])?;
    mutate_ticket(root, ticket_id, actor, |doc| {
        let ticket_type = doc.value("ticket_type").unwrap_or_default();
        if matches!(ticket_type.as_str(), "conversation" | "prototype")
            && operator_evidence.unwrap_or_default().trim().is_empty()
        {
            return Err(WayfinderError::Invalid(
                "conversation and prototype tickets require explicit operator evidence".into(),
            ));
        }
        if !matches!(
            doc.value("status").as_deref(),
            Some("open") | Some("claimed")
        ) {
            return Err(WayfinderError::Invalid(
                "only an open or claimed ticket can be resolved".into(),
            ));
        }
        doc.set("status", "resolved");
        doc.set("resolution_summary", &q(summary));
        doc.set("resolution_evidence", &q(evidence));
        doc.set(
            "operator_evidence",
            &operator_evidence.map(q).unwrap_or_else(|| "null".into()),
        );
        Ok("resolved")
    })
}
pub fn clear_map(
    root: impl AsRef<Path>,
    map_id: &str,
    actor: &str,
) -> Result<MutationResult, WayfinderError> {
    let guard = PathGuard::new(root)?;
    let _lock = WorkspaceLock::acquire(guard.root())?;
    let context = map_context(guard.root(), map_id)?;
    if !context.diagnostics.is_empty()
        || !context.map.fog.iter().all(|fog| fog.state != "active")
        || !context.frontier.is_empty()
        || !context.blocked.is_empty()
        || !context.claimed.is_empty()
    {
        return Err(WayfinderError::Invalid(
            "map cannot clear while diagnostics, active fog, or unresolved tickets remain".into(),
        ));
    }
    let path = guard
        .root()
        .join(".lmbrain/wayfinder/maps")
        .join(format!("{map_id}.md"));
    let mut doc = Document::parse(&fs::read_to_string(&path)?)?;
    doc.set("status", "cleared");
    doc.set("updated", &today());
    append_event(&mut doc, "cleared", actor);
    atomic_write(&path, &doc.render())?;
    Ok(result(map_id.into(), "cleared", guard.root(), &path))
}

fn mutate_ticket(
    root: impl AsRef<Path>,
    ticket_id: &str,
    actor: &str,
    change: impl FnOnce(&mut Document) -> Result<&'static str, WayfinderError>,
) -> Result<MutationResult, WayfinderError> {
    let guard = PathGuard::new(root)?;
    let _lock = WorkspaceLock::acquire(guard.root())?;
    let path = guard
        .root()
        .join(".lmbrain/wayfinder/tickets")
        .join(format!("{ticket_id}.md"));
    if !path.exists() {
        return Err(WayfinderError::NotFound(ticket_id.into()));
    }
    let source = fs::read_to_string(&path)?;
    let mut doc = Document::parse(&source)?;
    let status = change(&mut doc)?;
    doc.set("updated", &today());
    append_event(&mut doc, status, actor);
    atomic_write(&path, &doc.render())?;
    Ok(result(ticket_id.into(), status, guard.root(), &path))
}

fn classified(
    tickets: &[WayfinderTicket],
    map: &str,
) -> (
    Vec<WayfinderTicket>,
    Vec<WayfinderTicket>,
    Vec<WayfinderTicket>,
) {
    let resolved: BTreeSet<&str> = tickets
        .iter()
        .filter(|ticket| {
            matches!(
                ticket.status.as_str(),
                "resolved" | "out_of_scope" | "superseded"
            )
        })
        .map(|ticket| ticket.id.as_str())
        .collect();
    let mut frontier = vec![];
    let mut blocked = vec![];
    let mut claimed = vec![];
    for ticket in tickets.iter().filter(|ticket| ticket.map == map) {
        if ticket.status == "claimed" {
            claimed.push(ticket.clone());
        } else if ticket.status == "open" {
            if ticket
                .blockers
                .iter()
                .all(|blocker| resolved.contains(blocker.as_str()))
            {
                frontier.push(ticket.clone());
            } else {
                blocked.push(ticket.clone());
            }
        }
    }
    (frontier, blocked, claimed)
}
fn validate_map(map: &WayfinderMap, tickets: &[WayfinderTicket]) -> Vec<String> {
    let mut diagnostics = vec![];
    if map.malformed {
        diagnostics.push(format!(
            "{} could not be parsed and is treated as malformed",
            map.id
        ));
    }
    if map.destination.trim().is_empty() {
        diagnostics.push("map destination is required".into());
    }
    if !MAP_STATUSES.contains(&map.status.as_str()) {
        diagnostics.push(format!("invalid map status: {}", map.status));
    }
    let mut fog = BTreeSet::new();
    for item in &map.fog {
        if !fog.insert(&item.id) {
            diagnostics.push(format!("duplicate fog id: {}", item.id));
        }
    }
    // A malformed ticket's `map` field could not be read, so it cannot be
    // attributed to a specific map. Rather than silently ignore it (and risk
    // clear_map discarding a ticket that actually belongs here), surface it
    // as a diagnostic on every map until it is repaired.
    for ticket in tickets.iter().filter(|ticket| ticket.malformed) {
        diagnostics.push(format!(
            "{} could not be parsed and may belong to this map; resolve manually before clearing",
            ticket.id
        ));
    }
    for ticket in tickets
        .iter()
        .filter(|ticket| !ticket.malformed && ticket.map == map.id)
    {
        if ticket.question.trim().is_empty() {
            diagnostics.push(format!("{} has no precise question", ticket.id));
        }
        if !TICKET_TYPES.contains(&ticket.ticket_type.as_str())
            || !TICKET_STATUSES.contains(&ticket.status.as_str())
        {
            diagnostics.push(format!("{} has invalid type or status", ticket.id));
        }
    }
    diagnostics
}
fn validate_blockers(
    map: &str,
    blockers: &[String],
    tickets: &[WayfinderTicket],
) -> Result<(), WayfinderError> {
    let ids: BTreeMap<&str, &WayfinderTicket> = tickets
        .iter()
        .map(|ticket| (ticket.id.as_str(), ticket))
        .collect();
    for blocker in blockers {
        let ticket = ids
            .get(blocker.as_str())
            .ok_or_else(|| WayfinderError::Invalid(format!("blocker {blocker} does not exist")))?;
        if ticket.map != map {
            return Err(WayfinderError::Invalid(format!(
                "blocker {blocker} belongs to another map"
            )));
        }
    }
    Ok(())
}
fn parse_map(path: &Path, doc: &Document) -> WayfinderMap {
    WayfinderMap {
        id: doc.value("id").unwrap_or_else(|| "MALFORMED".into()),
        title: doc.value("title").unwrap_or_else(|| "Malformed map".into()),
        status: doc.value("status").unwrap_or_default(),
        destination: doc.value("destination").unwrap_or_default(),
        fog: doc
            .object_array("fog")
            .into_iter()
            .map(|item| WayfinderFog {
                id: item
                    .get("id")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .into(),
                summary: item
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .into(),
                provenance: item
                    .get("provenance")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .into(),
                state: item
                    .get("state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("active")
                    .into(),
                graduated_to: item
                    .get("graduated_to")
                    .and_then(|v| v.as_str())
                    .map(str::to_owned),
            })
            .collect(),
        path: path.to_string_lossy().replace('\\', "/"),
        updated: doc.value("updated").unwrap_or_default(),
        malformed: false,
    }
}
fn parse_ticket(path: &Path, doc: &Document) -> WayfinderTicket {
    WayfinderTicket {
        id: doc.value("id").unwrap_or_else(|| "MALFORMED".into()),
        map: doc.value("map").unwrap_or_default(),
        title: doc
            .value("title")
            .unwrap_or_else(|| "Malformed ticket".into()),
        status: doc.value("status").unwrap_or_default(),
        ticket_type: doc.value("ticket_type").unwrap_or_default(),
        question: doc.value("question").unwrap_or_default(),
        blockers: doc.string_array("blockers"),
        claimed_by: doc.value("claimed_by"),
        claimed_at: doc.value("claimed_at"),
        resolution_summary: doc.value("resolution_summary"),
        path: path.to_string_lossy().replace('\\', "/"),
        malformed: false,
    }
}
/// Scans `dir` for `{prefix}-*.md` artifacts. A file whose name matches the
/// prefix but fails to read or parse is surfaced via `placeholder` (id taken
/// from the filename) instead of being silently dropped, so a malformed
/// high-numbered file cannot cause its id to be reused by the next create.
fn scan<T>(
    dir: PathBuf,
    prefix: &str,
    parse: fn(&Path, &Document) -> T,
    placeholder: fn(String, &Path) -> T,
) -> Vec<T> {
    let mut items = vec![];
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|v| v.to_str()) != Some("md") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|v| v.to_str()) else {
                continue;
            };
            if !stem.starts_with(&format!("{prefix}-")) {
                continue;
            }
            match fs::read_to_string(&path)
                .ok()
                .and_then(|source| Document::parse(&source).ok())
            {
                Some(doc) => items.push(parse(&path, &doc)),
                None => items.push(placeholder(stem.to_string(), &path)),
            }
        }
    }
    items
}
fn malformed_map(id: String, path: &Path) -> WayfinderMap {
    WayfinderMap {
        id,
        title: "Malformed map".into(),
        status: String::new(),
        destination: String::new(),
        fog: vec![],
        path: path.to_string_lossy().replace('\\', "/"),
        updated: String::new(),
        malformed: true,
    }
}
fn malformed_ticket(id: String, path: &Path) -> WayfinderTicket {
    WayfinderTicket {
        id,
        map: String::new(),
        title: "Malformed ticket".into(),
        status: String::new(),
        ticket_type: String::new(),
        question: String::new(),
        blockers: vec![],
        claimed_by: None,
        claimed_at: None,
        resolution_summary: None,
        path: path.to_string_lossy().replace('\\', "/"),
        malformed: true,
    }
}
/// Derived from filenames on disk (not parsed documents), so an unparsable
/// file still reserves its id and cannot have its slot silently reused by the
/// next create.
fn next_id(root: &Path, prefix: &str) -> String {
    let dir = if prefix == "MAP" {
        root.join(".lmbrain/wayfinder/maps")
    } else {
        root.join(".lmbrain/wayfinder/tickets")
    };
    let max = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|v| v.to_str()) != Some("md") {
                return None;
            }
            path.file_stem()
                .and_then(|v| v.to_str())?
                .strip_prefix(&format!("{prefix}-"))?
                .parse::<u32>()
                .ok()
        })
        .max()
        .unwrap_or(0);
    format!("{prefix}-{:03}", max + 1)
}
fn required(values: &[(&str, &str)]) -> Result<(), WayfinderError> {
    for (name, value) in values {
        if value.trim().is_empty() {
            return Err(WayfinderError::Invalid(format!("{name} is required")));
        }
    }
    Ok(())
}
/// Appends a new event to `wayfinder_events` instead of replacing the array,
/// so `created` and every prior claim/release/resolve event survive each
/// mutation. Existing events are re-rendered from the parsed document (rather
/// than reusing their raw text) since `Document::set` only tracks the
/// rendered frontmatter text, not a separate per-key source snippet.
fn append_event(doc: &mut Document, action: &str, actor: &str) {
    let mut rendered: Vec<String> = doc
        .object_array("wayfinder_events")
        .iter()
        .map(render_event_object)
        .collect();
    rendered.push(event(action, actor));
    doc.set("wayfinder_events", &format!("[{}]", rendered.join(", ")));
}
fn render_event_object(fields: &serde_json::Map<String, serde_json::Value>) -> String {
    let field = |key: &str| -> String {
        fields
            .get(key)
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_string()
    };
    format!(
        "{{schema_version: {}, timestamp: {}, action: {}, actor: {}}}",
        q(&field("schema_version")),
        q(&field("timestamp")),
        q(&field("action")),
        q(&field("actor")),
    )
}
fn event(action: &str, actor: &str) -> String {
    format!(
        "{{schema_version: {}, timestamp: {}, action: {}, actor: {}}}",
        q(WAYFINDER_EVENT_SCHEMA_VERSION),
        q(&Local::now().to_rfc3339()),
        q(action),
        q(actor)
    )
}
fn q(value: &str) -> String {
    serde_json::to_string(value.trim()).unwrap()
}
fn array(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| q(value))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
fn today() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}
fn result(id: String, status: &str, root: &Path, path: &Path) -> MutationResult {
    MutationResult {
        id,
        status: status.into(),
        path: path.strip_prefix(root).unwrap_or(path).to_path_buf(),
        forced: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn workspace() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/wayfinder/maps")).unwrap();
        fs::create_dir_all(dir.path().join(".lmbrain/wayfinder/tickets")).unwrap();
        dir
    }

    fn create_map(root: &Path) -> String {
        super::create_map(
            root,
            WayfinderMapCreate {
                title: "Test map".into(),
                destination: "Somewhere".into(),
                notes: String::new(),
                out_of_scope: String::new(),
                actor: "tester".into(),
            },
        )
        .unwrap()
        .id
    }

    #[test]
    fn ticket_lifecycle_appends_events_instead_of_overwriting() {
        let dir = workspace();
        let root = dir.path();
        let map_id = create_map(root);
        let ticket = super::create_ticket(
            root,
            WayfinderTicketCreate {
                map: map_id,
                title: "Decide something".into(),
                ticket_type: "conversation".into(),
                question: "What should we do?".into(),
                blockers: vec![],
                bounded_context: "context".into(),
                actor: "tester".into(),
            },
        )
        .unwrap();
        super::claim_ticket(root, &ticket.id, "claimant").unwrap();
        super::resolve_ticket(
            root,
            &ticket.id,
            "resolver",
            "summary",
            "evidence",
            Some("operator said so"),
        )
        .unwrap();

        let path = root
            .join(".lmbrain/wayfinder/tickets")
            .join(format!("{}.md", ticket.id));
        let source = fs::read_to_string(&path).unwrap();
        let doc = Document::parse(&source).unwrap();
        let events = doc.object_array("wayfinder_events");
        assert_eq!(events.len(), 3, "events found: {events:?}");
        let actions: Vec<&str> = events
            .iter()
            .map(|event| event.get("action").and_then(Value::as_str).unwrap())
            .collect();
        assert_eq!(actions, vec!["created", "claimed", "resolved"]);
    }

    #[test]
    fn next_id_reserves_ids_from_malformed_high_numbered_files() {
        let dir = workspace();
        let root = dir.path();
        fs::write(
            root.join(".lmbrain/wayfinder/maps/MAP-003.md"),
            "not valid frontmatter at all",
        )
        .unwrap();
        let id = super::next_id(root, "MAP");
        assert_eq!(id, "MAP-004");
    }

    // `next_id` derives the next id from filenames on disk, so under normal,
    // single-process operation it can never propose an id that already has a
    // file — the overwrite guard is unreachable through that path by design.
    // It exists purely as defense against a filesystem that resolves two
    // distinct names to the same file, such as Windows' case-insensitivity:
    // a file that differs only in case is invisible to `next_id`'s
    // case-sensitive prefix match, yet still occupies the exact path a
    // subsequent create would target.
    #[cfg(windows)]
    #[test]
    fn create_map_refuses_to_overwrite_an_existing_file() {
        let dir = workspace();
        let root = dir.path();
        fs::write(
            root.join(".lmbrain/wayfinder/maps/map-001.md"),
            "not valid frontmatter",
        )
        .unwrap();
        let error = super::create_map(
            root,
            WayfinderMapCreate {
                title: "Test map".into(),
                destination: "Somewhere".into(),
                notes: String::new(),
                out_of_scope: String::new(),
                actor: "tester".into(),
            },
        )
        .unwrap_err();
        assert!(matches!(error, WayfinderError::Invalid(_)));
        assert!(error.to_string().contains("already exists"));
    }

    #[test]
    fn scan_surfaces_a_malformed_placeholder_instead_of_dropping_the_file() {
        let dir = workspace();
        let root = dir.path();
        fs::write(
            root.join(".lmbrain/wayfinder/maps/MAP-002.md"),
            "not valid frontmatter",
        )
        .unwrap();
        let maps = list_maps(root);
        assert_eq!(maps.len(), 1);
        assert!(maps[0].malformed);
        assert_eq!(maps[0].id, "MAP-002");

        // A malformed map's own context surfaces the corruption in diagnostics,
        // which blocks clearing it until it is repaired.
        let context = map_context(root, "MAP-002").unwrap();
        assert!(context.diagnostics.iter().any(|d| d.contains("MAP-002")));
        assert!(clear_map(root, "MAP-002", "tester").is_err());
    }

    #[test]
    fn a_malformed_ticket_blocks_clearing_every_map_since_its_owner_is_unknown() {
        let dir = workspace();
        let root = dir.path();
        let map_id = create_map(root);
        fs::write(
            root.join(".lmbrain/wayfinder/tickets/WAY-001.md"),
            "not valid frontmatter",
        )
        .unwrap();
        let context = map_context(root, &map_id).unwrap();
        assert!(context.diagnostics.iter().any(|d| d.contains("WAY-001")));
        assert!(clear_map(root, &map_id, "tester").is_err());
    }

    #[test]
    fn scan_results_are_sorted_by_id() {
        let dir = workspace();
        let root = dir.path();
        let _second = create_map(root);
        let _third = create_map(root);
        let ids: Vec<String> = list_maps(root).into_iter().map(|map| map.id).collect();
        let mut sorted = ids.clone();
        sorted.sort();
        assert_eq!(ids, sorted);
    }
}
