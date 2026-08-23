use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use crate::{
    frontmatter::Document,
    transitions::{kind_for_id, ArtifactKind},
};

#[derive(Debug, Clone)]
pub struct ArtifactEntry {
    pub id: String,
    pub kind: ArtifactKind,
    pub status: String,
    pub path: PathBuf,
    pub rel_path: PathBuf,
    pub document: Document,
    pub raw: String,
}

#[derive(Debug, Clone, Default)]
pub struct WorkspaceIndex {
    pub root: PathBuf,
    pub artifacts: HashMap<String, ArtifactEntry>,
    pub by_kind: HashMap<ArtifactKind, Vec<String>>,
    pub by_path: HashMap<PathBuf, String>,
}

impl WorkspaceIndex {
    pub fn scan(root: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let root = root.as_ref().to_path_buf();
        let lmbrain_dir = root.join(".lmbrain");
        let mut index = Self {
            root: root.clone(),
            artifacts: HashMap::new(),
            by_kind: HashMap::new(),
            by_path: HashMap::new(),
        };

        if !lmbrain_dir.exists() {
            return Ok(index);
        }

        let mut files = Vec::new();
        scan_dir(&lmbrain_dir, &lmbrain_dir, &mut files)?;
        files.sort_unstable();

        for file_path in files {
            let Ok(source) = fs::read_to_string(&file_path) else {
                continue;
            };
            let Ok(doc) = Document::parse(&source) else {
                continue;
            };
            let Some(id) = doc.value("id") else {
                continue;
            };
            let id = id.trim().to_string();
            if id.is_empty() {
                continue;
            }
            let Some(kind) = kind_for_id(&id) else {
                continue;
            };
            let status = doc.value("status").unwrap_or_default();
            let rel_path = file_path
                .strip_prefix(&root)
                .unwrap_or(&file_path)
                .to_path_buf();

            let entry = ArtifactEntry {
                id: id.clone(),
                kind,
                status,
                path: file_path.clone(),
                rel_path,
                document: doc,
                raw: source,
            };

            if let Some(previous) = index.artifacts.insert(id.clone(), entry) {
                index.by_path.remove(&previous.path);
            } else {
                index.by_kind.entry(kind).or_default().push(id.clone());
            }
            index.by_path.insert(file_path, id);
        }

        Ok(index)
    }

    pub fn get(&self, id: &str) -> Option<&ArtifactEntry> {
        self.artifacts.get(id)
    }

    pub fn get_by_path(&self, path: &Path) -> Option<&ArtifactEntry> {
        let lookup_path = if path.is_relative() {
            self.root.join(path)
        } else {
            path.to_path_buf()
        };
        self.by_path
            .get(&lookup_path)
            .and_then(|id| self.artifacts.get(id))
    }

    pub fn by_kind(&self, kind: ArtifactKind) -> impl Iterator<Item = &ArtifactEntry> {
        self.by_kind
            .get(&kind)
            .into_iter()
            .flatten()
            .filter_map(|id| self.artifacts.get(id))
    }

    pub fn id_exists(&self, id: &str) -> bool {
        self.artifacts.contains_key(id)
    }

    pub fn next_id(&self, kind: ArtifactKind) -> u32 {
        let prefix = kind.prefix();
        let mut max = 0;
        for id in self.artifacts.keys() {
            if let Some(num_str) = id.strip_prefix(prefix).and_then(|s| s.strip_prefix('-')) {
                if let Ok(num) = num_str.parse::<u32>() {
                    max = max.max(num);
                }
            }
        }
        max + 1
    }
}

pub(crate) fn is_artifact_scaffolding_path(lmbrain_dir: &Path, path: &Path) -> bool {
    let relative = path.strip_prefix(lmbrain_dir).unwrap_or(path);
    relative.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|value| value.eq_ignore_ascii_case("templates"))
    }) || path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.eq_ignore_ascii_case("README.md"))
}

pub(crate) fn is_artifact_markdown_path(lmbrain_dir: &Path, path: &Path) -> bool {
    path.extension().and_then(|ext| ext.to_str()) == Some("md")
        && !is_artifact_scaffolding_path(lmbrain_dir, path)
}

fn scan_dir(
    lmbrain_dir: &Path,
    dir: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), std::io::Error> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') || is_artifact_scaffolding_path(lmbrain_dir, &path) {
                    continue;
                }
                scan_dir(lmbrain_dir, &path, files)?;
            } else if is_artifact_markdown_path(lmbrain_dir, &path) {
                files.push(path);
            }
        }
    }
    Ok(())
}

pub fn scan_workspace(root: impl AsRef<Path>) -> Result<WorkspaceIndex, std::io::Error> {
    WorkspaceIndex::scan(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn index_scans_artifacts_and_resolves_lookups() {
        let dir = tempdir().unwrap();
        let lmbrain = dir.path().join(".lmbrain");
        let specs_backlog = lmbrain.join("specs/backlog");
        let reviews_pending = lmbrain.join("reviews/pending");
        fs::create_dir_all(&specs_backlog).unwrap();
        fs::create_dir_all(&reviews_pending).unwrap();

        fs::write(
            specs_backlog.join("SPEC-001-test.md"),
            "---\nid: SPEC-001\ntitle: Test Spec\nstatus: backlog\n---\n\n# Body",
        )
        .unwrap();

        fs::write(
            reviews_pending.join("REVIEW-001-test.md"),
            "---\nid: REVIEW-001\ntitle: Test Review\nspec_id: SPEC-001\nstatus: pending\n---\n\n# Body",
        )
        .unwrap();

        let index = scan_workspace(dir.path()).unwrap();
        assert!(index.id_exists("SPEC-001"));
        assert!(index.id_exists("REVIEW-001"));
        assert!(!index.id_exists("SPEC-002"));

        assert_eq!(index.next_id(ArtifactKind::Spec), 2);
        assert_eq!(index.next_id(ArtifactKind::Review), 2);
        assert_eq!(index.next_id(ArtifactKind::Adr), 1);

        let spec = index.get("SPEC-001").unwrap();
        assert_eq!(spec.kind, ArtifactKind::Spec);
        assert_eq!(spec.status, "backlog");

        let specs: Vec<_> = index.by_kind(ArtifactKind::Spec).collect();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].id, "SPEC-001");
    }

    #[test]
    fn duplicate_ids_are_exposed_once_to_index_consumers() {
        let dir = tempdir().unwrap();
        let backlog = dir.path().join(".lmbrain/specs/backlog");
        let ready = dir.path().join(".lmbrain/specs/ready");
        fs::create_dir_all(&backlog).unwrap();
        fs::create_dir_all(&ready).unwrap();
        for (directory, status) in [(&backlog, "backlog"), (&ready, "ready")] {
            fs::write(
                directory.join("SPEC-001.md"),
                format!("---\nid: SPEC-001\nstatus: {status}\n---\n"),
            )
            .unwrap();
        }

        let index = scan_workspace(dir.path()).unwrap();
        assert_eq!(index.by_kind(ArtifactKind::Spec).count(), 1);
        assert_eq!(index.get("SPEC-001").unwrap().status, "ready");
    }

    #[test]
    fn artifact_discovery_excludes_scaffolding_but_keeps_malformed_candidates() {
        let dir = tempdir().unwrap();
        let lmbrain = dir.path().join(".lmbrain");
        let open = lmbrain.join("findings/open");
        let templates = lmbrain.join("templates");
        fs::create_dir_all(&open).unwrap();
        fs::create_dir_all(&templates).unwrap();
        fs::write(lmbrain.join("findings/README.md"), "# Findings\n").unwrap();
        fs::write(open.join("README.md"), "# Open\n").unwrap();
        fs::write(templates.join("finding.md"), "id: FINDING-XXX\n").unwrap();
        let candidate = open.join("FINDING-001-broken.md");
        fs::write(&candidate, "broken\n").unwrap();

        assert!(is_artifact_scaffolding_path(
            &lmbrain,
            &lmbrain.join("findings/README.md")
        ));
        assert!(is_artifact_scaffolding_path(
            &lmbrain,
            &templates.join("finding.md")
        ));
        assert!(is_artifact_markdown_path(&lmbrain, &candidate));
        assert!(!is_artifact_markdown_path(
            &lmbrain,
            &open.join("README.md")
        ));
    }
}
