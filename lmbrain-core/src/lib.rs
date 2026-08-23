//! Tauri-free, filesystem-backed controlled mutations for LMBrain artifacts.
pub mod attestation;
pub mod branching_strategy;
pub mod context;
pub mod debt;
pub mod debt_migration;
pub mod diagnostics;
pub mod dream;
pub mod error;
pub mod frontmatter;
pub mod harness_environment;
pub mod harness_manifest;
pub mod improvement;
pub mod invariants;
pub mod kit_feedback;
pub mod kit_migration;
pub mod markdown;
mod mutation_lock;
pub mod path;
pub mod review;
pub mod roadmap;
pub mod spec_dependencies;
pub mod spec_verification_gates;
pub mod taxonomy;
pub mod transitions;
pub mod verification;
pub mod verification_onboarding;
pub mod workspace_index;

pub use error::{CoreError, ErrorCode};
pub use mutation_lock::WorkspaceLock;
pub use workspace_index::{scan_workspace, ArtifactEntry, WorkspaceIndex};

pub use attestation::{
    attest_spec_requirement, attest_spec_requirement_delegated,
    build_verification_migration_preview, operator_gates, requirement_digest,
    unsupported_verification_requirements, verification_attestations, verification_blockers,
    verification_blockers_for_workspace, verification_requirements, AttestationDelegation,
    AttestationError, AttestationResult, OperatorGate, VerificationAttestation, VerificationBlocker,
    VerificationMigrationItem, VerificationMigrationPreview,
    VERIFICATION_ATTESTATION_SCHEMA_VERSION,
};
pub use branching_strategy::{
    load_branching_strategy, parse_branching_strategy, set_branching_strategy,
    validate_branching_strategy, BranchAuthorityConfig, BranchNamingConfig, BranchingStrategy,
    BranchingStrategyError, BranchingTopology, CommitTriggersConfig, BRANCHING_STRATEGY_PATH,
    BRANCHING_STRATEGY_SCHEMA_VERSION,
};
pub use context::{
    build_project_digest, build_review_context, build_spec_context, AgentProfileSummary,
    BoundedDebtList, BoundedDiagnosticList, BoundedOperatorGateList, BoundedSpecList, CompactAdr,
    CompactDebt, CompactReview, CompactSpec, Criterion, DebtDigest, DeclaredProjectState,
    DerivedProjectState, DiagnosticsSummary, ProjectDigest, ReviewContext, SpecContext,
    SpecDependencyDigest, SpecDependencyDigestItem, SpecLifecycleView, SpecParkingSummary,
};
pub use debt::{
    accept_debt_risk, create_debt, debt_candidates, debt_context, defer_debt, list_debts,
    plan_debt, reopen_debt, resolve_debt, supersede_debt, validate_debt_document, Debt,
    DebtCandidate, DebtCandidateInventory, DebtContext, DebtCreateInput, DebtError,
    RelationSummary,
};
pub use debt_migration::{
    debt_migrate, debt_migration_preview, DebtMigrationError, DebtMigrationItem,
    DebtMigrationPreview, DebtMigrationReference, DebtMigrationResult,
};
pub use diagnostics::{
    build_diagnostics, Diagnostic, DiagnosticFixability, DiagnosticSeverity,
    DIAGNOSTIC_SCHEMA_VERSION,
};
pub use dream::{
    capture_dream, list_dreams, Dream, DreamCreateInput, DreamError, DREAM_EVENT_SCHEMA_VERSION,
};
pub use harness_manifest::{
    canonical_manifest_digest, content_digest, load_harness_manifest, parse_harness_manifest,
    set_harness_manifest, validate_harness_manifest, workspace_identity, BrowserMcpCapability,
    BrowserMcpMode, BrowserMcpProvider, CapabilityState, HarnessHost, HarnessManifest,
    HarnessManifestError, HarnessManifestMutation, HarnessValidationIssue, HostConfiguration,
    LspRequirement, WorkspaceIdentity,
};
pub use improvement::{
    apply_improvement_proposal, build_agent_improvement_signals, create_improvement_proposal,
    AgentEffectivenessMetrics, AgentImprovementSignal, ImprovementApplyResult, ImprovementError,
    ImprovementProposalRequest,
};
pub use invariants::{classify_acceptance_criteria, AcceptanceCriterion, AcceptanceCriterionState};
pub use kit_feedback::{
    read_kit_feedback, record_kit_feedback, record_kit_feedback_resolution, KitFeedbackError,
    KitFeedbackInput, KitFeedbackMutation, KitFeedbackNote, KitFeedbackNoteStatus,
    KitFeedbackReport, KitFeedbackResolution, KitFeedbackResolutionInput,
    KitFeedbackResolutionMutation, KIT_FEEDBACK_REPORT_PATH, KIT_FEEDBACK_SCHEMA_VERSION,
};
pub use kit_migration::{
    kit_migrate, kit_migration_preview, record_kit_baseline, KitMigrationError,
    KitMigrationItem, KitMigrationPreview, KitMigrationResult,
};
pub use path::{read_artifact, ArtifactReadError};
pub use review::{
    analyze_review_lifecycle, build_review_migration_preview, parse_review_event_history,
    parse_review_event_value, ReviewEventHistory, ReviewEventInput, ReviewHistorySource,
    ReviewLifecycleAnalysis, ReviewLifecycleEvent, ReviewMigrationItem, ReviewMigrationPreview,
    REVIEW_EVENT_SCHEMA_VERSION,
};
pub use roadmap::{is_roadmap_status, parse_roadmap, Roadmap, RoadmapMilestone, ROADMAP_STATUSES};
pub use spec_dependencies::{
    set_spec_dependencies, spec_dependency_blockers, spec_dependency_candidates,
    spec_dependency_context, validate_spec_dependency_graph, MalformedSpec, SpecDependency,
    SpecDependencyBlocker, SpecDependencyCandidate, SpecDependencyCandidateInventory,
    SpecDependencyContext, SpecDependencyError, SpecDependencyMutation,
    SPEC_DEPENDENCY_EVENT_SCHEMA_VERSION,
};
pub use spec_verification_gates::{
    set_spec_verification_gates, SpecVerificationGatesError, SpecVerificationGatesMutation,
    SPEC_VERIFICATION_GATE_EVENT_SCHEMA_VERSION,
};
pub use taxonomy::{
    canonical_finding_categories, canonical_spec_tags, capability_tiers, default_thinking_level,
    normalize_capability_tier, normalize_finding_category, normalize_spec_tag,
    normalize_thinking_level, thinking_level_allowed, thinking_levels, validate_spec_tags,
    CategoryNormalization, SpecTagIssue, SpecTagNormalization, EFFORT_TAXONOMY_VERSION,
    FINDING_TAXONOMY_VERSION, SPEC_TAG_TAXONOMY_VERSION,
};
pub use transitions::{
    park_spec, record_effort_observation, record_review_event, repair_artifact_frontmatter,
    review_verdict, set_review_implementation_agent, set_spec_effort, set_spec_tags, supersede_adr,
    AdrLifecycle, AdrStatus, AgentLifecycle, AgentProposalLifecycle, AgentProposalStatus,
    AgentStatus, ArtifactKind, CreateRequest, DebtLifecycle, DebtStatus, HandoffLifecycle,
    HandoffStatus, Lifecycle, McpLifecycle, McpProposalLifecycle, McpProposalStatus, McpStatus,
    MutationOptions, MutationResult, RepairResult, ReviewLifecycle, ReviewStatus, SkillLifecycle,
    SkillStatus, SpecLifecycle, SpecParkingInput, SpecStatus, TransitionError,
};
pub use verification::parse_manifest as parse_verification_manifest;
pub use verification::{
    approve_verification_manifest, canonical_verification_manifest_digest,
    execute_spec_verification, load_verification_manifest, transcript_state,
    transcript_state_for_document, validate_verification_manifest, workspace_content_fingerprint,
    TranscriptState, VerificationApproval, VerificationError, VerificationGate,
    VerificationManifest, VerificationRunReport, VERIFICATION_MANIFEST_PATH,
};
pub use verification_onboarding::{
    default_verification_approval_path, discover_verification_manifest,
    rollback_verification_manifest, set_verification_manifest,
    validate_verification_manifest_source, verification_manifest_status, VerificationGateCandidate,
    VerificationManifestPreview, VerificationManifestPreviewValidation, VerificationManifestState,
    VerificationManifestStatus, VerificationManifestWriteResult, VerificationOnboardingError,
};
