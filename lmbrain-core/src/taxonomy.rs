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
}
