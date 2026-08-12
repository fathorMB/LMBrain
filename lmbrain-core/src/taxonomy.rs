use serde::{Deserialize, Serialize};

pub const FINDING_TAXONOMY_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CategoryNormalization {
    pub raw: String,
    pub canonical: Option<String>,
    pub is_alias: bool,
}

pub fn normalize_finding_category(raw: &str) -> CategoryNormalization {
    let normalized = raw.trim().to_lowercase().replace([' ', '_'], "-");
    let canonical = match normalized.as_str() {
        "correctness" | "runtime-regression" | "invariant-preservation" | "root-cause" => {
            Some("correctness")
        }
        "verification-integrity"
        | "verification-transcript"
        | "evidence-integrity"
        | "evidence-accuracy"
        | "evidence-completeness"
        | "unreproducible-verification"
        | "unverified-deliverable"
        | "false-deviation" => Some("verification-integrity"),
        "test-quality" | "test-coverage" | "test-cannot-fail" | "tautological-gate"
        | "test-infra" => Some("test-quality"),
        "documentation"
        | "docs"
        | "documentation-accuracy"
        | "contract-hygiene"
        | "repo-hygiene"
        | "hygiene"
        | "readability" => Some("documentation"),
        "metrics-integrity"
        | "metric-cannot-fail"
        | "unsound-metric"
        | "metrics-validity"
        | "metric-validity"
        | "metrics-accounting"
        | "latency-accounting"
        | "counterfactual-causality" => Some("metrics-integrity"),
        "schema-conformance" | "schema-compatibility" | "schema-mismatch" | "canonical-binding" => {
            Some("schema-conformance")
        }
        "robustness" | "repair-fragility" | "timeout" | "error-taxonomy" => Some("robustness"),
        "usability" => Some("usability"),
        "accessibility" | "a11y" | "wcag" | "accessibility-fix" => Some("accessibility"),
        "localization" | "bilingualism" | "bilingual-divergence" => Some("localization"),
        "maintainability" => Some("maintainability"),
        "security-boundary" | "information-leak" => Some("security-boundary"),
        "compatibility" | "compatibility-validation" | "model-matrix" => Some("compatibility"),
        "performance" | "latency" => Some("performance"),
        "provenance" | "provenance-accuracy" | "undeclared-provenance" => Some("provenance"),
        "requirements-completeness"
        | "dropped-requirement"
        | "incomplete-criterion"
        | "completeness"
        | "missing-deliverable" => Some("requirements-completeness"),
        _ => None,
    }
    .map(str::to_owned);
    let is_alias = canonical
        .as_deref()
        .is_some_and(|canonical| canonical != normalized);
    CategoryNormalization {
        raw: raw.to_owned(),
        canonical,
        is_alias,
    }
}

pub fn canonical_finding_categories() -> &'static [&'static str] {
    &[
        "accessibility",
        "compatibility",
        "correctness",
        "documentation",
        "localization",
        "maintainability",
        "metrics-integrity",
        "performance",
        "provenance",
        "requirements-completeness",
        "robustness",
        "schema-conformance",
        "security-boundary",
        "test-quality",
        "usability",
        "verification-integrity",
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_converge_but_unknown_values_remain_inspectable() {
        for alias in [
            "evidence-integrity",
            "verification-transcript",
            "evidence_accuracy",
        ] {
            let normalized = normalize_finding_category(alias);
            assert_eq!(
                normalized.canonical.as_deref(),
                Some("verification-integrity")
            );
            assert_eq!(normalized.raw, alias);
        }
        let unknown = normalize_finding_category("project-specific-surprise");
        assert_eq!(unknown.canonical, None);
        assert_eq!(unknown.raw, "project-specific-surprise");
    }

    #[test]
    fn accessibility_is_canonical_and_its_aliases_converge() {
        assert!(canonical_finding_categories().contains(&"accessibility"));
        let canonical = normalize_finding_category("accessibility");
        assert_eq!(canonical.canonical.as_deref(), Some("accessibility"));
        assert!(!canonical.is_alias);
        for alias in ["a11y", "wcag", "accessibility-fix", "A11Y"] {
            let normalized = normalize_finding_category(alias);
            assert_eq!(normalized.canonical.as_deref(), Some("accessibility"));
            assert!(normalized.is_alias);
        }
    }
}

// ─── Spec tags (issue #49) ────────────────────────────────────────

pub const SPEC_TAG_TAXONOMY_VERSION: &str = "1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecTagNormalization {
    pub raw: String,
    /// Present when the value is syntactically valid, whether or not it is canonical.
    pub value: Option<String>,
    pub is_alias: bool,
    pub is_canonical: bool,
    pub invalid_reason: Option<String>,
}

/// Lowercase, trim, unify separators, drop a leading `#`. Values must match
/// `^[a-z0-9][a-z0-9-]*$` and be 2..=32 characters.
pub fn normalize_spec_tag(raw: &str) -> SpecTagNormalization {
    let trimmed = raw.trim().trim_start_matches('#').trim();
    let normalized = trimmed.to_lowercase().replace([' ', '_'], "-");
    let invalid_reason = if normalized.is_empty() {
        Some("tag is empty".to_string())
    } else if normalized.chars().count() < 2 {
        Some(format!("tag `{normalized}` is shorter than 2 characters"))
    } else if normalized.chars().count() > 32 {
        Some(format!("tag `{normalized}` is longer than 32 characters"))
    } else if !normalized
        .chars()
        .next()
        .is_some_and(|first| first.is_ascii_lowercase() || first.is_ascii_digit())
    {
        Some(format!(
            "tag `{normalized}` must start with a letter or digit"
        ))
    } else if !normalized.chars().all(|character| {
        character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
    }) {
        Some(format!(
            "tag `{normalized}` may only contain lowercase letters, digits, and `-`"
        ))
    } else {
        None
    };

    if invalid_reason.is_some() {
        return SpecTagNormalization {
            raw: raw.to_owned(),
            value: None,
            is_alias: false,
            is_canonical: false,
            invalid_reason,
        };
    }

    let canonical = canonical_spec_tag_alias(&normalized).unwrap_or_else(|| normalized.clone());
    SpecTagNormalization {
        raw: raw.to_owned(),
        is_alias: canonical != normalized,
        is_canonical: canonical_spec_tags().contains(&canonical.as_str()),
        value: Some(canonical),
        invalid_reason: None,
    }
}

fn canonical_spec_tag_alias(normalized: &str) -> Option<String> {
    match normalized {
        "docs" | "documentation-update" => Some("documentation"),
        "a11y" | "accessibility-fix" => Some("accessibility"),
        "perf" => Some("performance"),
        "user-interface" => Some("ui"),
        "user-experience" => Some("ux"),
        "test" | "tests" => Some("testing"),
        "refactoring" => Some("refactor"),
        _ => None,
    }
    .map(str::to_owned)
}

/// Starter vocabulary seeded from values already in use that do not restate a
/// structured field. It is a starting point, not a closed set: unknown values
/// stay usable and are reported as informational diagnostics.
pub fn canonical_spec_tags() -> &'static [&'static str] {
    &[
        "accessibility",
        "agents",
        "diagnostics",
        "documentation",
        "kit",
        "markdown",
        "mcp",
        "migration",
        "performance",
        "refactor",
        "regression",
        "remediation",
        "reviews",
        "roadmap",
        "security",
        "sessions",
        "testing",
        "ui",
        "ux",
        "verification",
        "wiki",
        "workflow",
    ]
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpecTagIssue {
    Invalid {
        raw: String,
        reason: String,
    },
    /// The value duplicates a structured field on the same spec.
    RestatesField {
        raw: String,
        field: String,
    },
    Duplicate {
        value: String,
    },
}

impl SpecTagIssue {
    pub fn message(&self) -> String {
        match self {
            Self::Invalid { reason, .. } => reason.clone(),
            Self::RestatesField { raw, field } => format!(
                "tag `{raw}` restates the `{field}` field; set `{field}` instead of tagging it"
            ),
            Self::Duplicate { value } => {
                format!("tag `{value}` appears more than once after normalization")
            }
        }
    }
}

fn looks_like_a_release_marker(value: &str) -> bool {
    let candidate = value.strip_prefix('v').unwrap_or(value);
    let parts: Vec<&str> = candidate.split('.').collect();
    parts.len() >= 2
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()))
}

/// Validates a tag list against the spec's own structured fields. Returns the
/// normalized list plus every issue found; callers decide whether issues block.
pub fn validate_spec_tags(
    raw_tags: &[String],
    milestone: Option<&str>,
    area: Option<&str>,
    priority: Option<&str>,
) -> (Vec<String>, Vec<SpecTagIssue>) {
    let mut normalized_tags: Vec<String> = Vec::new();
    let mut issues: Vec<SpecTagIssue> = Vec::new();

    let field_values: Vec<(&str, String)> = [
        ("milestone", milestone),
        ("area", area),
        ("priority", priority),
    ]
    .into_iter()
    .filter_map(|(field, value)| {
        let value = value?.trim().to_lowercase().replace([' ', '_'], "-");
        (!value.is_empty()).then_some((field, value))
    })
    .collect();

    for raw in raw_tags {
        // Field-restating values are checked before syntax so that `3.1.0`
        // reports "use the milestone field" rather than "dots are not allowed".
        let simplified = raw
            .trim()
            .trim_start_matches('#')
            .trim()
            .to_lowercase()
            .replace([' ', '_'], "-");

        if let Some((field, _)) = field_values
            .iter()
            .find(|(_, field_value)| field_value == &simplified)
        {
            issues.push(SpecTagIssue::RestatesField {
                raw: raw.clone(),
                field: (*field).to_string(),
            });
            continue;
        }

        // Release markers belong in `milestone` even when this spec has none.
        if looks_like_a_release_marker(&simplified) || simplified.starts_with("milestone-") {
            issues.push(SpecTagIssue::RestatesField {
                raw: raw.clone(),
                field: "milestone".to_string(),
            });
            continue;
        }

        let normalization = normalize_spec_tag(raw);
        let Some(value) = normalization.value else {
            issues.push(SpecTagIssue::Invalid {
                raw: raw.clone(),
                reason: normalization
                    .invalid_reason
                    .unwrap_or_else(|| "invalid tag".into()),
            });
            continue;
        };

        if normalized_tags.contains(&value) {
            issues.push(SpecTagIssue::Duplicate {
                value: value.clone(),
            });
            continue;
        }
        normalized_tags.push(value);
    }

    (normalized_tags, issues)
}

// ─── Effort and capability tiers (issue #64) ──────────────────────

pub const EFFORT_TAXONOMY_VERSION: &str = "1";

pub fn capability_tiers() -> &'static [&'static str] {
    &["luna", "terra", "sol"]
}

pub fn thinking_levels() -> &'static [&'static str] {
    &["minimal", "standard", "extended", "maximum"]
}

pub fn normalize_capability_tier(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_lowercase();
    capability_tiers()
        .contains(&normalized.as_str())
        .then_some(normalized)
}

pub fn normalize_thinking_level(raw: &str) -> Option<String> {
    let normalized = raw.trim().to_lowercase();
    thinking_levels()
        .contains(&normalized.as_str())
        .then_some(normalized)
}

/// Each tier carries a default level so a Lead states one decision, not two.
pub fn default_thinking_level(tier: &str) -> &'static str {
    match tier {
        "luna" => "minimal",
        "sol" => "extended",
        _ => "standard",
    }
}

/// Constrained combinations: footprint and delicacy are independent, but the
/// extremes of one bound the other.
pub fn thinking_level_allowed(tier: &str, level: &str) -> Result<(), String> {
    match (tier, level) {
        ("sol", "minimal") => Err(
            "a Sol spec cannot use `minimal` reasoning: cross-layer work needs deliberation".into(),
        ),
        ("luna", "maximum") => Err(
            "a Luna spec cannot use `maximum` reasoning without a recorded reason; record one or raise the tier".into(),
        ),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod spec_metadata_tests {
    use super::*;

    #[test]
    fn tags_normalize_case_separators_and_aliases() {
        assert_eq!(
            normalize_spec_tag("  Kit_Feedback ").value.as_deref(),
            Some("kit-feedback")
        );
        let alias = normalize_spec_tag("#Docs");
        assert_eq!(alias.value.as_deref(), Some("documentation"));
        assert!(alias.is_alias);
        assert!(alias.is_canonical);
    }

    #[test]
    fn syntactically_invalid_tags_are_reported_not_silently_dropped() {
        for raw in ["", "x", "spaced value!", &"a".repeat(33)] {
            let normalization = normalize_spec_tag(raw);
            assert!(
                normalization.value.is_none(),
                "expected {raw} to be invalid"
            );
            assert!(normalization.invalid_reason.is_some());
        }
    }

    #[test]
    fn tags_restating_structured_fields_are_rejected() {
        let (tags, issues) = validate_spec_tags(
            &[
                "3.1.0".into(),
                "milestone-m02".into(),
                "Rust".into(),
                "high".into(),
                "wiki".into(),
            ],
            Some("3.1.0"),
            Some("rust"),
            Some("high"),
        );
        assert_eq!(tags, vec!["wiki".to_string()]);
        assert_eq!(issues.len(), 4);
        assert!(issues
            .iter()
            .all(|issue| matches!(issue, SpecTagIssue::RestatesField { .. })));
        assert!(issues[0].message().contains("milestone"));
    }

    #[test]
    fn release_markers_are_rejected_even_without_a_milestone_field() {
        let (tags, issues) = validate_spec_tags(&["v3.1".into(), "2.8.0".into()], None, None, None);
        assert!(tags.is_empty());
        assert_eq!(issues.len(), 2);
    }

    #[test]
    fn unknown_tags_stay_usable() {
        let (tags, issues) = validate_spec_tags(&["project-specific".into()], None, None, None);
        assert_eq!(tags, vec!["project-specific".to_string()]);
        assert!(issues.is_empty());
        assert!(!normalize_spec_tag("project-specific").is_canonical);
    }

    #[test]
    fn duplicates_collapse_after_normalization() {
        let (tags, issues) = validate_spec_tags(&["UI".into(), "ui".into()], None, None, None);
        assert_eq!(tags, vec!["ui".to_string()]);
        assert!(matches!(
            issues.as_slice(),
            [SpecTagIssue::Duplicate { .. }]
        ));
    }

    #[test]
    fn effort_tiers_default_and_constrain_thinking_levels() {
        assert_eq!(normalize_capability_tier(" Sol ").as_deref(), Some("sol"));
        assert_eq!(normalize_capability_tier("jupiter"), None);
        assert_eq!(default_thinking_level("luna"), "minimal");
        assert_eq!(default_thinking_level("terra"), "standard");
        assert_eq!(default_thinking_level("sol"), "extended");
        assert!(thinking_level_allowed("sol", "minimal").is_err());
        assert!(thinking_level_allowed("luna", "maximum").is_err());
        assert!(thinking_level_allowed("terra", "maximum").is_ok());
        assert!(thinking_level_allowed("sol", "maximum").is_ok());
    }
}
