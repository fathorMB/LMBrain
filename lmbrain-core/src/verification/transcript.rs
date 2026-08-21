use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use serde::Serialize;

use super::{
    execution::VerificationGateResult,
    fingerprint::workspace_content_fingerprint_with,
    manifest::{
        canonical_verification_manifest_digest, gate_contract_digest, hex_digest,
        load_verification_manifest, manifest_exclusions,
    },
    VerificationError,
};
use crate::frontmatter::Document;
use crate::markdown::fence_mask;

pub const GENERATED_TRANSCRIPT_START: &str = "<!-- lmbrain-generated-verification:start -->";
pub const GENERATED_TRANSCRIPT_END: &str = "<!-- lmbrain-generated-verification:end -->";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TranscriptState {
    Missing,
    Empty,
    HandAuthored,
    GeneratedFresh,
    GeneratedStale,
}

pub fn render_transcript(
    manifest_digest: &str,
    pre_fingerprint: &str,
    fingerprint: &str,
    gate_contract_digest: &str,
    results: &[VerificationGateResult],
    invalidation_reason: Option<&str>,
    hash: Option<&str>,
) -> String {
    let mut text = format!(
        "<!-- generated-by: lmbrain-verify@{} -->\n<!-- manifest-digest: {manifest_digest} -->\n<!-- gate-contract-digest: {gate_contract_digest} -->\n<!-- workspace-fingerprint-before: {pre_fingerprint} -->\n<!-- workspace-fingerprint: {fingerprint} -->\n",
        env!("CARGO_PKG_VERSION")
    );
    if let Some(reason) = invalidation_reason {
        text.push_str(&format!("<!-- invalidated: {reason} -->\n"));
    }
    if let Some(hash) = hash {
        text.push_str(&format!("<!-- transcript-hash: {hash} -->\n"));
    }
    for result in results {
        text.push_str(&format!(
            "\n#### Gate `{}`\n\n```text\n$ {}\nstarted: {}\nfinished: {}\nduration_ms: {}\nexit_code: {}\ntimed_out: {}\nexpectation_met: {}\nenvironment_policy: minimal-inherited-allowlist\nremoved_environment_variables: {}\n--- stdout ---\n{}\n--- stderr ---\n{}\n```\n",
            result.id,
            result.command,
            result.started_at,
            result.finished_at,
            result.duration_ms,
            result.exit_code.map_or_else(|| "none".into(), |code| code.to_string()),
            result.timed_out,
            result.expectation_met,
            result.removed_environment_variables.join(", "),
            result.stdout,
            result.stderr
        ));
    }
    text
}

pub fn replace_transcript(body: &str, transcript: &str) -> Result<String, VerificationError> {
    let lines: Vec<&str> = body.lines().collect();
    let fenced = fence_mask(&lines);
    let implementation = lines
        .iter()
        .enumerate()
        .position(|(index, line)| !fenced[index] && line.trim() == "## Implementation evidence")
        .ok_or_else(|| VerificationError::Artifact("missing ## Implementation evidence".into()))?;
    let existing = lines
        .iter()
        .enumerate()
        .skip(implementation + 1)
        .find(|(index, line)| !fenced[*index] && line.trim() == "### Verification transcript")
        .map(|(index, _)| index);
    let mut output: Vec<String> = Vec::new();
    if let Some(start) = existing {
        let end = lines
            .iter()
            .enumerate()
            .skip(start + 1)
            .find(|(index, line)| !fenced[*index] && line.trim_start().starts_with("### "))
            .map(|(index, _)| index)
            .unwrap_or(lines.len());
        let section = lines[start + 1..end].join("\n");
        let start_count = section.matches(GENERATED_TRANSCRIPT_START).count();
        let end_count = section.matches(GENERATED_TRANSCRIPT_END).count();
        if start_count != end_count || start_count > 1 {
            return Err(VerificationError::Artifact(
                "verification transcript has an incomplete or duplicate LMBrain-managed region"
                    .into(),
            ));
        }
        let managed = format!(
            "{GENERATED_TRANSCRIPT_START}\n{}\n{GENERATED_TRANSCRIPT_END}",
            transcript.trim_matches('\n')
        );
        let updated = if let Some((range_start, range_end)) = generated_transcript_range(&section) {
            format!(
                "{}{}{}",
                &section[..range_start],
                managed,
                &section[range_end..]
            )
        } else if section.trim().is_empty() {
            format!("\n{managed}\n")
        } else {
            format!("{}\n\n{managed}\n", section.trim_end_matches('\n'))
        };
        output.extend(lines[..=start].iter().map(|line| (*line).to_string()));
        output.extend(updated.lines().map(str::to_string));
        output.extend(lines[end..].iter().map(|line| (*line).to_string()));
    } else {
        let end = lines
            .iter()
            .enumerate()
            .skip(implementation + 1)
            .find(|(index, line)| !fenced[*index] && line.trim_start().starts_with("## "))
            .map(|(index, _)| index)
            .unwrap_or(lines.len());
        output.extend(lines[..end].iter().map(|line| (*line).to_string()));
        output.push(String::new());
        output.push("### Verification transcript".into());
        output.push(String::new());
        output.push(GENERATED_TRANSCRIPT_START.into());
        output.extend(transcript.trim_matches('\n').lines().map(str::to_string));
        output.push(GENERATED_TRANSCRIPT_END.into());
        output.extend(lines[end..].iter().map(|line| (*line).to_string()));
    }
    Ok(format!("{}\n", output.join("\n")))
}

pub fn generated_transcript(section: &str) -> Option<&str> {
    if let Some(start) = section.find(GENERATED_TRANSCRIPT_START) {
        let content_start = start + GENERATED_TRANSCRIPT_START.len();
        let end = section[content_start..].find(GENERATED_TRANSCRIPT_END)? + content_start;
        return Some(section[content_start..end].trim_matches('\n'));
    }
    generated_transcript_range(section).map(|(start, end)| section[start..end].trim_matches('\n'))
}

pub fn generated_transcript_range(section: &str) -> Option<(usize, usize)> {
    if let Some(start) = section.find(GENERATED_TRANSCRIPT_START) {
        let after_start = start + GENERATED_TRANSCRIPT_START.len();
        let end = section[after_start..].find(GENERATED_TRANSCRIPT_END)?
            + after_start
            + GENERATED_TRANSCRIPT_END.len();
        return Some((start, end));
    }

    let start = section.find("<!-- generated-by: lmbrain-verify@")?;
    let legacy = &section[start..];
    let recorded_hash = metadata(legacy, "transcript-hash")?;
    let mut candidate_ends = legacy
        .match_indices('\n')
        .map(|(offset, _)| offset + 1)
        .collect::<Vec<_>>();
    if candidate_ends.last().copied() != Some(legacy.len()) {
        candidate_ends.push(legacy.len());
    }
    candidate_ends.into_iter().find_map(|end| {
        transcript_hash_matches(&legacy[..end], &recorded_hash).then_some((start, start + end))
    })
}

pub fn transcript_hash_matches(transcript: &str, recorded_hash: &str) -> bool {
    let without_hash = transcript
        .lines()
        .filter(|line| !line.trim().starts_with("<!-- transcript-hash:"))
        .collect::<Vec<_>>()
        .join("\n");
    let canonical = format!("{}\n", without_hash.trim_matches('\n'));
    hex_digest(canonical.as_bytes()) == recorded_hash
}

pub fn section_at_level<'a>(body: &'a str, heading: &str, level: usize) -> Option<&'a str> {
    crate::markdown::find_section(body, heading, level)
}

pub fn has_nonempty_fence(section: &str) -> bool {
    let mut in_fence = false;
    let mut content = false;
    for line in section.lines() {
        if line.trim_start().starts_with("```") {
            if in_fence && content {
                return true;
            }
            in_fence = !in_fence;
            content = false;
        } else if in_fence && !line.trim().is_empty() {
            content = true;
        }
    }
    false
}

pub fn metadata(section: &str, key: &str) -> Option<String> {
    let prefix = format!("<!-- {key}:");
    section.lines().find_map(|line| {
        let line = line.trim();
        line.strip_prefix(&prefix)
            .and_then(|value| value.strip_suffix("-->"))
            .map(|value| value.trim().to_string())
    })
}

pub fn transcript_state(root: &Path, body: &str) -> TranscriptState {
    transcript_state_with_exclusions(root, body, &BTreeSet::new())
}

pub fn transcript_state_with_exclusions(
    root: &Path,
    body: &str,
    exclusions: &BTreeSet<PathBuf>,
) -> TranscriptState {
    let Some(implementation) = section_at_level(body, "Implementation evidence", 2) else {
        return TranscriptState::Missing;
    };
    let Some(section) = section_at_level(implementation, "Verification transcript", 3) else {
        return TranscriptState::Missing;
    };
    if !has_nonempty_fence(section) {
        return TranscriptState::Empty;
    }
    let Some(generated) = generated_transcript(section) else {
        if section.contains("generated-by: lmbrain-verify")
            || section.contains(GENERATED_TRANSCRIPT_START)
            || section.contains(GENERATED_TRANSCRIPT_END)
        {
            return TranscriptState::GeneratedStale;
        }
        return TranscriptState::HandAuthored;
    };
    let Some(recorded) = metadata(generated, "workspace-fingerprint") else {
        return TranscriptState::HandAuthored;
    };
    if metadata(generated, "invalidated").is_some() {
        return TranscriptState::GeneratedStale;
    }
    if metadata(generated, "workspace-fingerprint-before").is_some_and(|before| before != recorded)
    {
        return TranscriptState::GeneratedStale;
    }
    let Some(recorded_manifest) = metadata(generated, "manifest-digest") else {
        return TranscriptState::GeneratedStale;
    };
    let Some(recorded_hash) = metadata(generated, "transcript-hash") else {
        return TranscriptState::GeneratedStale;
    };
    if !transcript_hash_matches(generated, &recorded_hash) {
        return TranscriptState::GeneratedStale;
    }
    let current_manifest = load_verification_manifest(root)
        .and_then(|manifest| canonical_verification_manifest_digest(&manifest));
    match (
        workspace_content_fingerprint_with(root, exclusions),
        current_manifest,
    ) {
        (Ok(current), Ok(manifest)) if current == recorded && manifest == recorded_manifest => {
            TranscriptState::GeneratedFresh
        }
        _ => TranscriptState::GeneratedStale,
    }
}

pub fn transcript_state_for_document(root: &Path, document: &Document) -> TranscriptState {
    let exclusions = load_verification_manifest(root)
        .map(|manifest| {
            manifest_exclusions(&manifest, &document.string_array("verification_gates"))
        })
        .unwrap_or_default();
    let state = transcript_state_with_exclusions(root, &document.body, &exclusions);
    if state != TranscriptState::GeneratedFresh {
        return state;
    }
    let Some(implementation) = section_at_level(&document.body, "Implementation evidence", 2)
    else {
        return TranscriptState::GeneratedStale;
    };
    let Some(section) = section_at_level(implementation, "Verification transcript", 3) else {
        return TranscriptState::GeneratedStale;
    };
    let Some(generated) = generated_transcript(section) else {
        return TranscriptState::GeneratedStale;
    };
    let expected = gate_contract_digest(&document.string_array("verification_gates"));
    match metadata(generated, "gate-contract-digest") {
        Some(recorded) if recorded == expected => TranscriptState::GeneratedFresh,
        _ => TranscriptState::GeneratedStale,
    }
}
