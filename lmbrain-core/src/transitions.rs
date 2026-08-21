use std::{
    fs,
    path::{Path, PathBuf},
};

use chrono::Local;
use serde::{Deserialize, Serialize};

use crate::{
    frontmatter::{atomic_write, repair_duplicate_top_level_keys, Document},
    invariants,
    mutation_lock::ArtifactMutationLock,
    path::PathGuard,
    review::{
        next_review_event_id, parse_review_event_history, ReviewEventInput,
        REVIEW_EVENT_SCHEMA_VERSION,
    },
    taxonomy::{normalize_finding_category, FINDING_TAXONOMY_VERSION},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SpecStatus {
    Backlog,
    Ready,
    Working,
    Review,
    Done,
    Discarded,
}

impl SpecStatus {
    pub fn all() -> &'static [Self] {
        &[
            Self::Backlog,
            Self::Ready,
            Self::Working,
            Self::Review,
            Self::Done,
            Self::Discarded,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Backlog => "backlog",
            Self::Ready => "ready",
            Self::Working => "working",
            Self::Review => "review",
            Self::Done => "done",
            Self::Discarded => "discarded",
        }
    }
}

impl Default for SpecStatus {
    fn default() -> Self {
        Self::Backlog
    }
}

impl std::fmt::Display for SpecStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for SpecStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "backlog" => Ok(Self::Backlog),
            "ready" => Ok(Self::Ready),
            "working" => Ok(Self::Working),
            "review" => Ok(Self::Review),
            "done" => Ok(Self::Done),
            "discarded" => Ok(Self::Discarded),
            _ => Err(format!("invalid spec status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewStatus {
    Pending,
    Accepted,
    ChangesRequested,
    Blocked,
    Superseded,
}

impl ReviewStatus {
    pub fn all() -> &'static [Self] {
        &[
            Self::Pending,
            Self::Accepted,
            Self::ChangesRequested,
            Self::Blocked,
            Self::Superseded,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::ChangesRequested => "changes-requested",
            Self::Blocked => "blocked",
            Self::Superseded => "superseded",
        }
    }
}

impl Default for ReviewStatus {
    fn default() -> Self {
        Self::Pending
    }
}

impl std::fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ReviewStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "changes-requested" => Ok(Self::ChangesRequested),
            "blocked" => Ok(Self::Blocked),
            "superseded" => Ok(Self::Superseded),
            _ => Err(format!("invalid review status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AdrStatus {
    Proposed,
    Accepted,
    Rejected,
    Superseded,
    Deprecated,
}

impl AdrStatus {
    pub fn all() -> &'static [Self] {
        &[
            Self::Proposed,
            Self::Accepted,
            Self::Rejected,
            Self::Superseded,
            Self::Deprecated,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Deprecated => "deprecated",
        }
    }
}

impl Default for AdrStatus {
    fn default() -> Self {
        Self::Proposed
    }
}

impl std::fmt::Display for AdrStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AdrStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "proposed" => Ok(Self::Proposed),
            "accepted" => Ok(Self::Accepted),
            "rejected" => Ok(Self::Rejected),
            "superseded" => Ok(Self::Superseded),
            "deprecated" => Ok(Self::Deprecated),
            _ => Err(format!("invalid adr status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentStatus {
    Proposed,
    Active,
    Inactive,
    Retired,
}

impl AgentStatus {
    pub fn all() -> &'static [Self] {
        &[
            Self::Proposed,
            Self::Active,
            Self::Inactive,
            Self::Retired,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Retired => "retired",
        }
    }
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self::Proposed
    }
}

impl std::fmt::Display for AgentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AgentStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "proposed" => Ok(Self::Proposed),
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "retired" => Ok(Self::Retired),
            _ => Err(format!("invalid agent status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentProposalStatus {
    Proposed,
    Approved,
    Rejected,
}

impl AgentProposalStatus {
    pub fn all() -> &'static [Self] {
        &[Self::Proposed, Self::Approved, Self::Rejected]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
}

impl Default for AgentProposalStatus {
    fn default() -> Self {
        Self::Proposed
    }
}

impl std::fmt::Display for AgentProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AgentProposalStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "proposed" => Ok(Self::Proposed),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            _ => Err(format!("invalid agent proposal status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpStatus {
    Specified,
    Active,
    Inactive,
    Deprecated,
}

impl McpStatus {
    pub fn all() -> &'static [Self] {
        &[
            Self::Specified,
            Self::Active,
            Self::Inactive,
            Self::Deprecated,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Specified => "specified",
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Deprecated => "deprecated",
        }
    }
}

impl Default for McpStatus {
    fn default() -> Self {
        Self::Specified
    }
}

impl std::fmt::Display for McpStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for McpStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "specified" => Ok(Self::Specified),
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "deprecated" => Ok(Self::Deprecated),
            _ => Err(format!("invalid mcp status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum McpProposalStatus {
    Proposed,
    Approved,
    Rejected,
    Implemented,
    Blocked,
}

impl McpProposalStatus {
    pub fn all() -> &'static [Self] {
        &[
            Self::Proposed,
            Self::Approved,
            Self::Rejected,
            Self::Implemented,
            Self::Blocked,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Implemented => "implemented",
            Self::Blocked => "blocked",
        }
    }
}

impl Default for McpProposalStatus {
    fn default() -> Self {
        Self::Proposed
    }
}

impl std::fmt::Display for McpProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for McpProposalStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "proposed" => Ok(Self::Proposed),
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "implemented" => Ok(Self::Implemented),
            "blocked" => Ok(Self::Blocked),
            _ => Err(format!("invalid mcp proposal status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HandoffStatus {
    Ready,
    Consumed,
    Superseded,
    Archived,
}

impl HandoffStatus {
    pub fn all() -> &'static [Self] {
        &[
            Self::Ready,
            Self::Consumed,
            Self::Superseded,
            Self::Archived,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Consumed => "consumed",
            Self::Superseded => "superseded",
            Self::Archived => "archived",
        }
    }
}

impl Default for HandoffStatus {
    fn default() -> Self {
        Self::Ready
    }
}

impl std::fmt::Display for HandoffStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for HandoffStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "ready" => Ok(Self::Ready),
            "consumed" => Ok(Self::Consumed),
            "superseded" => Ok(Self::Superseded),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("invalid handoff status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SkillStatus {
    Proposed,
    Active,
    Retired,
}

impl SkillStatus {
    pub fn all() -> &'static [Self] {
        &[Self::Proposed, Self::Active, Self::Retired]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Active => "active",
            Self::Retired => "retired",
        }
    }
}

impl Default for SkillStatus {
    fn default() -> Self {
        Self::Proposed
    }
}

impl std::fmt::Display for SkillStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for SkillStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "proposed" => Ok(Self::Proposed),
            "active" => Ok(Self::Active),
            "retired" => Ok(Self::Retired),
            _ => Err(format!("invalid skill status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DebtStatus {
    Open,
    Planned,
    Deferred,
    Resolved,
    AcceptedRisk,
    Superseded,
}

impl DebtStatus {
    pub fn all() -> &'static [Self] {
        &[
            Self::Open,
            Self::Planned,
            Self::Deferred,
            Self::Resolved,
            Self::AcceptedRisk,
            Self::Superseded,
        ]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Planned => "planned",
            Self::Deferred => "deferred",
            Self::Resolved => "resolved",
            Self::AcceptedRisk => "accepted-risk",
            Self::Superseded => "superseded",
        }
    }
}

impl Default for DebtStatus {
    fn default() -> Self {
        Self::Open
    }
}

impl std::fmt::Display for DebtStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for DebtStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "open" => Ok(Self::Open),
            "planned" => Ok(Self::Planned),
            "deferred" => Ok(Self::Deferred),
            "resolved" => Ok(Self::Resolved),
            "accepted-risk" => Ok(Self::AcceptedRisk),
            "superseded" => Ok(Self::Superseded),
            _ => Err(format!("invalid debt status: {s}")),
        }
    }
}

pub trait Lifecycle: 'static + Sized {
    type Status: Copy
        + Eq
        + std::hash::Hash
        + std::fmt::Debug
        + std::fmt::Display
        + Default
        + std::str::FromStr
        + Serialize
        + for<'de> Deserialize<'de>
        + 'static;

    fn kind() -> ArtifactKind;
    fn transitions() -> &'static [(Self::Status, Self::Status)];
    fn is_transition_allowed(from: Self::Status, to: Self::Status) -> bool;
    fn status_dir(status: Self::Status) -> Result<String, TransitionError>;
    fn default_status() -> Self::Status;
    fn authority(target: Self::Status) -> &'static str;
    fn template_name() -> &'static str;
    fn creation_statuses() -> &'static [Self::Status];
    fn moves_for_status() -> bool {
        true
    }
}

pub struct SpecLifecycle;
impl Lifecycle for SpecLifecycle {
    type Status = SpecStatus;

    fn kind() -> ArtifactKind {
        ArtifactKind::Spec
    }

    fn transitions() -> &'static [(SpecStatus, SpecStatus)] {
        &[
            (SpecStatus::Backlog, SpecStatus::Ready),
            (SpecStatus::Ready, SpecStatus::Working),
            (SpecStatus::Working, SpecStatus::Review),
            (SpecStatus::Review, SpecStatus::Done),
            (SpecStatus::Backlog, SpecStatus::Discarded),
            (SpecStatus::Ready, SpecStatus::Discarded),
            (SpecStatus::Working, SpecStatus::Discarded),
            (SpecStatus::Review, SpecStatus::Discarded),
        ]
    }

    fn is_transition_allowed(from: SpecStatus, to: SpecStatus) -> bool {
        matches!(
            (from, to),
            (SpecStatus::Backlog, SpecStatus::Ready)
                | (SpecStatus::Ready, SpecStatus::Working)
                | (SpecStatus::Working, SpecStatus::Review)
                | (SpecStatus::Review, SpecStatus::Done)
                | (_, SpecStatus::Discarded)
        )
    }

    fn status_dir(status: SpecStatus) -> Result<String, TransitionError> {
        Ok(status.as_str().to_string())
    }

    fn default_status() -> SpecStatus {
        SpecStatus::Backlog
    }

    fn authority(target: SpecStatus) -> &'static str {
        match target {
            SpecStatus::Ready => "operator",
            SpecStatus::Working | SpecStatus::Review => "implementation-specialist",
            SpecStatus::Done | SpecStatus::Discarded => "project-lead",
            SpecStatus::Backlog => "project-lead",
        }
    }

    fn template_name() -> &'static str {
        "spec.md"
    }

    fn creation_statuses() -> &'static [SpecStatus] {
        &[SpecStatus::Backlog]
    }

    fn moves_for_status() -> bool {
        true
    }
}

pub struct ReviewLifecycle;
impl Lifecycle for ReviewLifecycle {
    type Status = ReviewStatus;

    fn kind() -> ArtifactKind {
        ArtifactKind::Review
    }

    fn transitions() -> &'static [(ReviewStatus, ReviewStatus)] {
        &[
            (ReviewStatus::Pending, ReviewStatus::Accepted),
            (ReviewStatus::Pending, ReviewStatus::ChangesRequested),
            (ReviewStatus::Pending, ReviewStatus::Blocked),
            (ReviewStatus::ChangesRequested, ReviewStatus::ChangesRequested),
            (ReviewStatus::ChangesRequested, ReviewStatus::Accepted),
            (ReviewStatus::ChangesRequested, ReviewStatus::Blocked),
            (ReviewStatus::Blocked, ReviewStatus::Blocked),
            (ReviewStatus::Blocked, ReviewStatus::ChangesRequested),
            (ReviewStatus::Blocked, ReviewStatus::Accepted),
            (ReviewStatus::Pending, ReviewStatus::Superseded),
            (ReviewStatus::ChangesRequested, ReviewStatus::Superseded),
            (ReviewStatus::Blocked, ReviewStatus::Superseded),
        ]
    }

    fn is_transition_allowed(from: ReviewStatus, to: ReviewStatus) -> bool {
        matches!(
            (from, to),
            (ReviewStatus::Pending, ReviewStatus::Accepted)
                | (ReviewStatus::Pending, ReviewStatus::ChangesRequested)
                | (ReviewStatus::Pending, ReviewStatus::Blocked)
                | (ReviewStatus::ChangesRequested, ReviewStatus::ChangesRequested)
                | (ReviewStatus::ChangesRequested, ReviewStatus::Accepted)
                | (ReviewStatus::ChangesRequested, ReviewStatus::Blocked)
                | (ReviewStatus::Blocked, ReviewStatus::Blocked)
                | (ReviewStatus::Blocked, ReviewStatus::ChangesRequested)
                | (ReviewStatus::Blocked, ReviewStatus::Accepted)
                | (_, ReviewStatus::Superseded)
        )
    }

    fn status_dir(status: ReviewStatus) -> Result<String, TransitionError> {
        Ok(status.as_str().to_string())
    }

    fn default_status() -> ReviewStatus {
        ReviewStatus::Pending
    }

    fn authority(target: ReviewStatus) -> &'static str {
        match target {
            ReviewStatus::Accepted => "operator",
            _ => "project-lead",
        }
    }

    fn template_name() -> &'static str {
        "review.md"
    }

    fn creation_statuses() -> &'static [ReviewStatus] {
        &[ReviewStatus::Pending]
    }

    fn moves_for_status() -> bool {
        true
    }
}

pub struct AdrLifecycle;
impl Lifecycle for AdrLifecycle {
    type Status = AdrStatus;

    fn kind() -> ArtifactKind {
        ArtifactKind::Adr
    }

    fn transitions() -> &'static [(AdrStatus, AdrStatus)] {
        &[
            (AdrStatus::Proposed, AdrStatus::Accepted),
            (AdrStatus::Proposed, AdrStatus::Rejected),
            (AdrStatus::Accepted, AdrStatus::Superseded),
            (AdrStatus::Accepted, AdrStatus::Deprecated),
        ]
    }

    fn is_transition_allowed(from: AdrStatus, to: AdrStatus) -> bool {
        matches!(
            (from, to),
            (AdrStatus::Proposed, AdrStatus::Accepted)
                | (AdrStatus::Proposed, AdrStatus::Rejected)
                | (AdrStatus::Accepted, AdrStatus::Superseded)
                | (AdrStatus::Accepted, AdrStatus::Deprecated)
        )
    }

    fn status_dir(status: AdrStatus) -> Result<String, TransitionError> {
        Ok(status.as_str().to_string())
    }

    fn default_status() -> AdrStatus {
        AdrStatus::Proposed
    }

    fn authority(_target: AdrStatus) -> &'static str {
        "controlled-mutation"
    }

    fn template_name() -> &'static str {
        "adr.md"
    }

    fn creation_statuses() -> &'static [AdrStatus] {
        &[AdrStatus::Proposed]
    }

    fn moves_for_status() -> bool {
        false
    }
}

pub struct AgentLifecycle;
impl Lifecycle for AgentLifecycle {
    type Status = AgentStatus;

    fn kind() -> ArtifactKind {
        ArtifactKind::Agent
    }

    fn transitions() -> &'static [(AgentStatus, AgentStatus)] {
        &[
            (AgentStatus::Proposed, AgentStatus::Active),
            (AgentStatus::Proposed, AgentStatus::Inactive),
            (AgentStatus::Active, AgentStatus::Inactive),
            (AgentStatus::Inactive, AgentStatus::Active),
            (AgentStatus::Proposed, AgentStatus::Retired),
            (AgentStatus::Active, AgentStatus::Retired),
            (AgentStatus::Inactive, AgentStatus::Retired),
        ]
    }

    fn is_transition_allowed(from: AgentStatus, to: AgentStatus) -> bool {
        matches!(
            (from, to),
            (AgentStatus::Proposed, AgentStatus::Active)
                | (AgentStatus::Proposed, AgentStatus::Inactive)
                | (AgentStatus::Active, AgentStatus::Inactive)
                | (AgentStatus::Inactive, AgentStatus::Active)
                | (_, AgentStatus::Retired)
        )
    }

    fn status_dir(status: AgentStatus) -> Result<String, TransitionError> {
        Ok(status.as_str().to_string())
    }

    fn default_status() -> AgentStatus {
        AgentStatus::Proposed
    }

    fn authority(_target: AgentStatus) -> &'static str {
        "controlled-mutation"
    }

    fn template_name() -> &'static str {
        "agent-profile.md"
    }

    fn creation_statuses() -> &'static [AgentStatus] {
        &[AgentStatus::Proposed]
    }

    fn moves_for_status() -> bool {
        false
    }
}

pub struct AgentProposalLifecycle;
impl Lifecycle for AgentProposalLifecycle {
    type Status = AgentProposalStatus;

    fn kind() -> ArtifactKind {
        ArtifactKind::AgentProposal
    }

    fn transitions() -> &'static [(AgentProposalStatus, AgentProposalStatus)] {
        &[
            (AgentProposalStatus::Proposed, AgentProposalStatus::Approved),
            (AgentProposalStatus::Proposed, AgentProposalStatus::Rejected),
        ]
    }

    fn is_transition_allowed(from: AgentProposalStatus, to: AgentProposalStatus) -> bool {
        matches!(
            (from, to),
            (AgentProposalStatus::Proposed, AgentProposalStatus::Approved)
                | (AgentProposalStatus::Proposed, AgentProposalStatus::Rejected)
        )
    }

    fn status_dir(status: AgentProposalStatus) -> Result<String, TransitionError> {
        Ok(status.as_str().to_string())
    }

    fn default_status() -> AgentProposalStatus {
        AgentProposalStatus::Proposed
    }

    fn authority(_target: AgentProposalStatus) -> &'static str {
        "controlled-mutation"
    }

    fn template_name() -> &'static str {
        "agent-proposal.md"
    }

    fn creation_statuses() -> &'static [AgentProposalStatus] {
        &[AgentProposalStatus::Proposed]
    }

    fn moves_for_status() -> bool {
        false
    }
}

pub struct McpLifecycle;
impl Lifecycle for McpLifecycle {
    type Status = McpStatus;

    fn kind() -> ArtifactKind {
        ArtifactKind::Mcp
    }

    fn transitions() -> &'static [(McpStatus, McpStatus)] {
        &[
            (McpStatus::Specified, McpStatus::Active),
            (McpStatus::Active, McpStatus::Inactive),
            (McpStatus::Inactive, McpStatus::Active),
            (McpStatus::Specified, McpStatus::Deprecated),
            (McpStatus::Active, McpStatus::Deprecated),
            (McpStatus::Inactive, McpStatus::Deprecated),
        ]
    }

    fn is_transition_allowed(from: McpStatus, to: McpStatus) -> bool {
        matches!(
            (from, to),
            (McpStatus::Specified, McpStatus::Active)
                | (McpStatus::Active, McpStatus::Inactive)
                | (McpStatus::Inactive, McpStatus::Active)
                | (_, McpStatus::Deprecated)
        )
    }

    fn status_dir(status: McpStatus) -> Result<String, TransitionError> {
        Ok(status.as_str().to_string())
    }

    fn default_status() -> McpStatus {
        McpStatus::Specified
    }

    fn authority(_target: McpStatus) -> &'static str {
        "controlled-mutation"
    }

    fn template_name() -> &'static str {
        "mcp-spec.md"
    }

    fn creation_statuses() -> &'static [McpStatus] {
        &[McpStatus::Specified]
    }

    fn moves_for_status() -> bool {
        false
    }
}

pub struct McpProposalLifecycle;
impl Lifecycle for McpProposalLifecycle {
    type Status = McpProposalStatus;

    fn kind() -> ArtifactKind {
        ArtifactKind::McpProposal
    }

    fn transitions() -> &'static [(McpProposalStatus, McpProposalStatus)] {
        &[
            (McpProposalStatus::Proposed, McpProposalStatus::Approved),
            (McpProposalStatus::Proposed, McpProposalStatus::Rejected),
            (McpProposalStatus::Approved, McpProposalStatus::Implemented),
            (McpProposalStatus::Proposed, McpProposalStatus::Blocked),
            (McpProposalStatus::Approved, McpProposalStatus::Blocked),
        ]
    }

    fn is_transition_allowed(from: McpProposalStatus, to: McpProposalStatus) -> bool {
        matches!(
            (from, to),
            (McpProposalStatus::Proposed, McpProposalStatus::Approved)
                | (McpProposalStatus::Proposed, McpProposalStatus::Rejected)
                | (McpProposalStatus::Approved, McpProposalStatus::Implemented)
                | (_, McpProposalStatus::Blocked)
        )
    }

    fn status_dir(status: McpProposalStatus) -> Result<String, TransitionError> {
        Ok(status.as_str().to_string())
    }

    fn default_status() -> McpProposalStatus {
        McpProposalStatus::Proposed
    }

    fn authority(_target: McpProposalStatus) -> &'static str {
        "controlled-mutation"
    }

    fn template_name() -> &'static str {
        "mcp-proposal.md"
    }

    fn creation_statuses() -> &'static [McpProposalStatus] {
        &[McpProposalStatus::Proposed]
    }

    fn moves_for_status() -> bool {
        false
    }
}

pub struct HandoffLifecycle;
impl Lifecycle for HandoffLifecycle {
    type Status = HandoffStatus;

    fn kind() -> ArtifactKind {
        ArtifactKind::Handoff
    }

    fn transitions() -> &'static [(HandoffStatus, HandoffStatus)] {
        &[
            (HandoffStatus::Ready, HandoffStatus::Consumed),
            (HandoffStatus::Ready, HandoffStatus::Superseded),
            (HandoffStatus::Ready, HandoffStatus::Archived),
            (HandoffStatus::Consumed, HandoffStatus::Archived),
            (HandoffStatus::Superseded, HandoffStatus::Archived),
        ]
    }

    fn is_transition_allowed(from: HandoffStatus, to: HandoffStatus) -> bool {
        matches!(
            (from, to),
            (HandoffStatus::Ready, HandoffStatus::Consumed)
                | (HandoffStatus::Ready, HandoffStatus::Superseded)
                | (_, HandoffStatus::Archived)
        )
    }

    fn status_dir(status: HandoffStatus) -> Result<String, TransitionError> {
        match status {
            HandoffStatus::Ready => Ok("active".to_string()),
            HandoffStatus::Consumed | HandoffStatus::Superseded | HandoffStatus::Archived => {
                Ok("archive".to_string())
            }
        }
    }

    fn default_status() -> HandoffStatus {
        HandoffStatus::Ready
    }

    fn authority(_target: HandoffStatus) -> &'static str {
        "controlled-mutation"
    }

    fn template_name() -> &'static str {
        "session-handoff.md"
    }

    fn creation_statuses() -> &'static [HandoffStatus] {
        &[HandoffStatus::Ready]
    }

    fn moves_for_status() -> bool {
        true
    }
}

pub struct SkillLifecycle;
impl Lifecycle for SkillLifecycle {
    type Status = SkillStatus;

    fn kind() -> ArtifactKind {
        ArtifactKind::Skill
    }

    fn transitions() -> &'static [(SkillStatus, SkillStatus)] {
        &[
            (SkillStatus::Proposed, SkillStatus::Active),
            (SkillStatus::Proposed, SkillStatus::Retired),
            (SkillStatus::Active, SkillStatus::Retired),
        ]
    }

    fn is_transition_allowed(from: SkillStatus, to: SkillStatus) -> bool {
        matches!(
            (from, to),
            (SkillStatus::Proposed, SkillStatus::Active)
                | (SkillStatus::Proposed, SkillStatus::Retired)
                | (SkillStatus::Active, SkillStatus::Retired)
        )
    }

    fn status_dir(status: SkillStatus) -> Result<String, TransitionError> {
        Ok(status.as_str().to_string())
    }

    fn default_status() -> SkillStatus {
        SkillStatus::Proposed
    }

    fn authority(_target: SkillStatus) -> &'static str {
        "controlled-mutation"
    }

    fn template_name() -> &'static str {
        "skill.md"
    }

    fn creation_statuses() -> &'static [SkillStatus] {
        &[SkillStatus::Proposed]
    }

    fn moves_for_status() -> bool {
        true
    }
}

pub struct DebtLifecycle;
impl Lifecycle for DebtLifecycle {
    type Status = DebtStatus;

    fn kind() -> ArtifactKind {
        ArtifactKind::Debt
    }

    fn transitions() -> &'static [(DebtStatus, DebtStatus)] {
        &[
            (DebtStatus::Open, DebtStatus::Planned),
            (DebtStatus::Open, DebtStatus::Deferred),
            (DebtStatus::Open, DebtStatus::Resolved),
            (DebtStatus::Open, DebtStatus::AcceptedRisk),
            (DebtStatus::Open, DebtStatus::Superseded),
            (DebtStatus::Planned, DebtStatus::Open),
            (DebtStatus::Planned, DebtStatus::Deferred),
            (DebtStatus::Planned, DebtStatus::Resolved),
            (DebtStatus::Planned, DebtStatus::AcceptedRisk),
            (DebtStatus::Planned, DebtStatus::Superseded),
            (DebtStatus::Deferred, DebtStatus::Open),
            (DebtStatus::Deferred, DebtStatus::Planned),
            (DebtStatus::Deferred, DebtStatus::Resolved),
            (DebtStatus::Deferred, DebtStatus::AcceptedRisk),
            (DebtStatus::Deferred, DebtStatus::Superseded),
        ]
    }

    fn is_transition_allowed(from: DebtStatus, to: DebtStatus) -> bool {
        matches!(
            (from, to),
            (DebtStatus::Open, DebtStatus::Planned)
                | (DebtStatus::Open, DebtStatus::Deferred)
                | (DebtStatus::Open, DebtStatus::Resolved)
                | (DebtStatus::Open, DebtStatus::AcceptedRisk)
                | (DebtStatus::Open, DebtStatus::Superseded)
                | (DebtStatus::Planned, DebtStatus::Open)
                | (DebtStatus::Planned, DebtStatus::Deferred)
                | (DebtStatus::Planned, DebtStatus::Resolved)
                | (DebtStatus::Planned, DebtStatus::AcceptedRisk)
                | (DebtStatus::Planned, DebtStatus::Superseded)
                | (DebtStatus::Deferred, DebtStatus::Open)
                | (DebtStatus::Deferred, DebtStatus::Planned)
                | (DebtStatus::Deferred, DebtStatus::Resolved)
                | (DebtStatus::Deferred, DebtStatus::AcceptedRisk)
                | (DebtStatus::Deferred, DebtStatus::Superseded)
        )
    }

    fn status_dir(status: DebtStatus) -> Result<String, TransitionError> {
        Ok(status.as_str().to_string())
    }

    fn default_status() -> DebtStatus {
        DebtStatus::Open
    }

    fn authority(_target: DebtStatus) -> &'static str {
        "controlled-mutation"
    }

    fn template_name() -> &'static str {
        "debt.md"
    }

    fn creation_statuses() -> &'static [DebtStatus] {
        &[DebtStatus::Open]
    }

    fn moves_for_status() -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    Debt,
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
            Self::Debt => "DEBT",
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
            Self::Debt => "debts",
        }
    }

    pub fn status_dir(self, status: &str) -> Result<String, TransitionError> {
        match self {
            Self::Spec => status
                .parse::<SpecStatus>()
                .map_err(|e| TransitionError::Invariant(e))
                .and_then(SpecLifecycle::status_dir),
            Self::Review => status
                .parse::<ReviewStatus>()
                .map_err(|e| TransitionError::Invariant(e))
                .and_then(ReviewLifecycle::status_dir),
            Self::Adr => status
                .parse::<AdrStatus>()
                .map_err(|e| TransitionError::Invariant(e))
                .and_then(AdrLifecycle::status_dir),
            Self::Agent => status
                .parse::<AgentStatus>()
                .map_err(|e| TransitionError::Invariant(e))
                .and_then(AgentLifecycle::status_dir),
            Self::AgentProposal => status
                .parse::<AgentProposalStatus>()
                .map_err(|e| TransitionError::Invariant(e))
                .and_then(AgentProposalLifecycle::status_dir),
            Self::Mcp => status
                .parse::<McpStatus>()
                .map_err(|e| TransitionError::Invariant(e))
                .and_then(McpLifecycle::status_dir),
            Self::McpProposal => status
                .parse::<McpProposalStatus>()
                .map_err(|e| TransitionError::Invariant(e))
                .and_then(McpProposalLifecycle::status_dir),
            Self::Handoff => status
                .parse::<HandoffStatus>()
                .map_err(|e| TransitionError::Invariant(e))
                .and_then(HandoffLifecycle::status_dir),
            Self::Skill => status
                .parse::<SkillStatus>()
                .map_err(|e| TransitionError::Invariant(e))
                .and_then(SkillLifecycle::status_dir),
            Self::Debt => status
                .parse::<DebtStatus>()
                .map_err(|e| TransitionError::Invariant(e))
                .and_then(DebtLifecycle::status_dir),
        }
    }

    pub fn moves_for_status(self) -> bool {
        match self {
            Self::Spec => SpecLifecycle::moves_for_status(),
            Self::Review => ReviewLifecycle::moves_for_status(),
            Self::Adr => AdrLifecycle::moves_for_status(),
            Self::Agent => AgentLifecycle::moves_for_status(),
            Self::AgentProposal => AgentProposalLifecycle::moves_for_status(),
            Self::Mcp => McpLifecycle::moves_for_status(),
            Self::McpProposal => McpProposalLifecycle::moves_for_status(),
            Self::Handoff => HandoffLifecycle::moves_for_status(),
            Self::Skill => SkillLifecycle::moves_for_status(),
            Self::Debt => DebtLifecycle::moves_for_status(),
        }
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
            Self::Debt => &["open"],
        }
    }

    pub fn default_status(self) -> &'static str {
        match self {
            Self::Spec => SpecLifecycle::default_status().as_str(),
            Self::Review => ReviewLifecycle::default_status().as_str(),
            Self::Adr => AdrLifecycle::default_status().as_str(),
            Self::Agent => AgentLifecycle::default_status().as_str(),
            Self::AgentProposal => AgentProposalLifecycle::default_status().as_str(),
            Self::Mcp => McpLifecycle::default_status().as_str(),
            Self::McpProposal => McpProposalLifecycle::default_status().as_str(),
            Self::Handoff => HandoffLifecycle::default_status().as_str(),
            Self::Skill => SkillLifecycle::default_status().as_str(),
            Self::Debt => DebtLifecycle::default_status().as_str(),
        }
    }

    pub fn template_name(self) -> &'static str {
        match self {
            Self::Spec => SpecLifecycle::template_name(),
            Self::Review => ReviewLifecycle::template_name(),
            Self::Adr => AdrLifecycle::template_name(),
            Self::Agent => AgentLifecycle::template_name(),
            Self::AgentProposal => AgentProposalLifecycle::template_name(),
            Self::Mcp => McpLifecycle::template_name(),
            Self::McpProposal => McpProposalLifecycle::template_name(),
            Self::Handoff => HandoffLifecycle::template_name(),
            Self::Skill => SkillLifecycle::template_name(),
            Self::Debt => DebtLifecycle::template_name(),
        }
    }

    pub fn authority(self, target: &str) -> &'static str {
        match self {
            Self::Spec => target
                .parse::<SpecStatus>()
                .map(SpecLifecycle::authority)
                .unwrap_or("controlled-mutation"),
            Self::Review => target
                .parse::<ReviewStatus>()
                .map(ReviewLifecycle::authority)
                .unwrap_or("controlled-mutation"),
            Self::Adr => target
                .parse::<AdrStatus>()
                .map(AdrLifecycle::authority)
                .unwrap_or("controlled-mutation"),
            Self::Agent => target
                .parse::<AgentStatus>()
                .map(AgentLifecycle::authority)
                .unwrap_or("controlled-mutation"),
            Self::AgentProposal => target
                .parse::<AgentProposalStatus>()
                .map(AgentProposalLifecycle::authority)
                .unwrap_or("controlled-mutation"),
            Self::Mcp => target
                .parse::<McpStatus>()
                .map(McpLifecycle::authority)
                .unwrap_or("controlled-mutation"),
            Self::McpProposal => target
                .parse::<McpProposalStatus>()
                .map(McpProposalLifecycle::authority)
                .unwrap_or("controlled-mutation"),
            Self::Handoff => target
                .parse::<HandoffStatus>()
                .map(HandoffLifecycle::authority)
                .unwrap_or("controlled-mutation"),
            Self::Skill => target
                .parse::<SkillStatus>()
                .map(SkillLifecycle::authority)
                .unwrap_or("controlled-mutation"),
            Self::Debt => target
                .parse::<DebtStatus>()
                .map(DebtLifecycle::authority)
                .unwrap_or("controlled-mutation"),
        }
    }

    pub fn is_allowed(self, from: &str, to: &str) -> bool {
        match self {
            Self::Spec => match (from.parse::<SpecStatus>(), to.parse::<SpecStatus>()) {
                (Ok(f), Ok(t)) => SpecLifecycle::is_transition_allowed(f, t),
                _ => false,
            },
            Self::Review => match (from.parse::<ReviewStatus>(), to.parse::<ReviewStatus>()) {
                (Ok(f), Ok(t)) => ReviewLifecycle::is_transition_allowed(f, t),
                _ => false,
            },
            Self::Adr => match (from.parse::<AdrStatus>(), to.parse::<AdrStatus>()) {
                (Ok(f), Ok(t)) => AdrLifecycle::is_transition_allowed(f, t),
                _ => false,
            },
            Self::Agent => match (from.parse::<AgentStatus>(), to.parse::<AgentStatus>()) {
                (Ok(f), Ok(t)) => AgentLifecycle::is_transition_allowed(f, t),
                _ => false,
            },
            Self::AgentProposal => match (
                from.parse::<AgentProposalStatus>(),
                to.parse::<AgentProposalStatus>(),
            ) {
                (Ok(f), Ok(t)) => AgentProposalLifecycle::is_transition_allowed(f, t),
                _ => false,
            },
            Self::Mcp => match (from.parse::<McpStatus>(), to.parse::<McpStatus>()) {
                (Ok(f), Ok(t)) => McpLifecycle::is_transition_allowed(f, t),
                _ => false,
            },
            Self::McpProposal => match (
                from.parse::<McpProposalStatus>(),
                to.parse::<McpProposalStatus>(),
            ) {
                (Ok(f), Ok(t)) => McpProposalLifecycle::is_transition_allowed(f, t),
                _ => false,
            },
            Self::Handoff => match (from.parse::<HandoffStatus>(), to.parse::<HandoffStatus>()) {
                (Ok(f), Ok(t)) => HandoffLifecycle::is_transition_allowed(f, t),
                _ => false,
            },
            Self::Skill => match (from.parse::<SkillStatus>(), to.parse::<SkillStatus>()) {
                (Ok(f), Ok(t)) => SkillLifecycle::is_transition_allowed(f, t),
                _ => false,
            },
            Self::Debt => match (from.parse::<DebtStatus>(), to.parse::<DebtStatus>()) {
                (Ok(f), Ok(t)) => DebtLifecycle::is_transition_allowed(f, t),
                _ => false,
            },
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
    "verification_gate_events",
    "parking_events",
    "finding_taxonomy_version",
];

/// Managed frontmatter fields whose value is a list. Creation input for these
/// is normalized to inline-array syntax or rejected, and validation reports a
/// scalar value in any of them as a diagnostic instead of silently dropping
/// relationships (KIT-NOTE-002).
pub(crate) const LIST_VALUED_FIELDS: &[&str] = &[
    "depends_on",
    "skills",
    "verification_gates",
    "related_tasks",
    "related_decisions",
    "links",
    "tags",
    "effort_observations",
    "supersedes",
    "superseded_by",
    "finding_categories",
    "evidence_refs",
];

pub(crate) fn is_list_valued_field(key: &str) -> bool {
    LIST_VALUED_FIELDS.contains(&key.trim().to_ascii_lowercase().as_str())
}

/// Normalizes creation input for a list-valued field: inline arrays must parse
/// as arrays, bare scalars are treated as comma-separated ID lists, and
/// anything ambiguous is rejected rather than stored as a scalar the resolvers
/// would silently ignore.
fn normalize_list_field_value(key: &str, value: &str) -> Result<String, TransitionError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok("[]".into());
    }
    if trimmed.starts_with('[') {
        let parsed = crate::frontmatter::parse_inline_value(trimmed).map_err(|error| {
            TransitionError::InvalidField(format!(
                "field '{key}' is list-valued but its value does not parse as an array: {error}"
            ))
        })?;
        if !parsed.is_array() {
            return Err(TransitionError::InvalidField(format!(
                "field '{key}' is list-valued but received a non-array value"
            )));
        }
        return Ok(trimmed.to_string());
    }
    if trimmed.contains('[') || trimmed.contains(']') {
        return Err(TransitionError::InvalidField(format!(
            "field '{key}' is list-valued; use inline array syntax, e.g. [A, B]"
        )));
    }
    let tokens: Vec<&str> = trimmed.split(',').map(str::trim).collect();
    if tokens.iter().any(|token| token.is_empty()) {
        return Err(TransitionError::InvalidField(format!(
            "field '{key}' is list-valued and contains an empty entry; use inline array syntax, e.g. [A, B]"
        )));
    }
    Ok(format!("[{}]", tokens.join(", ")))
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

pub use crate::error::CoreError as TransitionError;

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
    if let Err(reason) = invariants::implementation_agent_resolves(
        guard.root(),
        document.value("implementation_agent").as_deref(),
    ) {
        if !options.force {
            return Err(TransitionError::Invariant(reason));
        }
    }
    if let Err(reason) = invariants::agent_reference_resolves(
        guard.root(),
        "remediation_agent",
        event.remediation_agent.as_deref(),
    ) {
        if !options.force {
            return Err(TransitionError::Invariant(reason));
        }
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
    if kind == ArtifactKind::Debt {
        return Err(TransitionError::Invariant(
            "debt lifecycle changes require a semantic debt operation".into(),
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

    if kind == ArtifactKind::Spec && target == "ready" {
        let gates = document.string_array("verification_gates");
        let (requirements, _, _) =
            crate::context::parse_verification_requirements(&document.body, &gates);
        let declared_executables: Vec<String> = requirements
            .into_iter()
            .filter(|r| r.kind == "executable")
            .map(|r| r.id)
            .collect();
        if !declared_executables.is_empty() {
            let manifest = crate::load_verification_manifest(guard.root()).ok();
            let approval_path = crate::default_verification_approval_path(guard.root());
            let status = crate::verification_manifest_status(guard.root(), &approval_path).ok();
            if status.as_ref().map(|s| &s.state)
                != Some(&crate::VerificationManifestState::Approved)
            {
                let msg = "spec declares executable verification gates but verification manifest is not approved".to_string();
                if !options.force {
                    return Err(TransitionError::Invariant(msg));
                }
            } else if let Some(manifest) = manifest {
                let known: std::collections::BTreeSet<_> =
                    manifest.gates.into_iter().map(|g| g.id).collect();
                let missing: Vec<_> = declared_executables
                    .into_iter()
                    .filter(|g| !known.contains(g))
                    .collect();
                if !missing.is_empty() {
                    let msg = format!("spec declares executable verification gates missing from approved manifest: {}", missing.join(", "));
                    if !options.force {
                        return Err(TransitionError::Invariant(msg));
                    }
                }
            }
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
    kind.authority(target)
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

/// Governed correction for a provably wrong review attribution (#93 /
/// KIT-NOTE-010). Lighter than review_supersede: the frontmatter field is
/// corrected in place while the append-only history gains an
/// `attribution-correction` event recording the previous value, the actor
/// and the reason. The corrected value must resolve strictly — there is no
/// force path to write another placeholder.
pub fn set_review_implementation_agent(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    agent: &str,
    actor: &str,
    reason: &str,
) -> Result<MutationResult, TransitionError> {
    let agent = agent.trim();
    if actor.trim().is_empty() {
        return Err(TransitionError::Invariant(
            "attribution correction requires a non-empty actor".into(),
        ));
    }
    if reason.trim().is_empty() {
        return Err(TransitionError::Invariant(
            "attribution correction requires a non-empty reason".into(),
        ));
    }

    let guard = PathGuard::new(root)?;
    invariants::implementation_agent_resolves(guard.root(), Some(agent))
        .map_err(TransitionError::Invariant)?;
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
            "attribution correction requires a stable REVIEW-* artifact".into(),
        ));
    }
    validate_existing_review_history(&document)?;
    let status = document
        .value("status")
        .ok_or_else(|| TransitionError::Missing("status".into()))?;
    if status == "superseded" {
        return Err(TransitionError::Invariant(
            "cannot correct attribution on a superseded review".into(),
        ));
    }
    let previous = document.value("implementation_agent").unwrap_or_default();
    if previous.trim() == agent {
        return Err(TransitionError::Invariant(format!(
            "implementation_agent is already '{agent}'"
        )));
    }

    document.set("implementation_agent", agent);
    document.set("updated", &today());
    document.append_activity(&format!(
        "corrected implementation_agent {} -> {agent}",
        if previous.trim().is_empty() {
            "(unset)"
        } else {
            previous.trim()
        }
    ));
    let event = ReviewEventInput {
        actor_role: actor.trim().into(),
        reason: format!(
            "attribution corrected from '{}' to '{agent}': {}",
            if previous.trim().is_empty() {
                "(unset)"
            } else {
                previous.trim()
            },
            reason.trim()
        ),
        evidence_refs: Vec::new(),
        remediation_agent: None,
    };
    append_review_event(
        &mut document,
        &id,
        "attribution-correction",
        &status,
        &status,
        &event,
    )?;
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
        forced: false,
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

    let field_valid = valid(guard.root());
    if !field_valid && !options.force {
        return Err(TransitionError::Invariant(format!("invalid {key}")));
    }

    document.set(key, value);
    document.set("updated", &today());
    document.append_activity(&format!("set {key}"));
    if options.force {
        if let Some(reason) = options.reason.as_deref() {
            let status = document.value("status").unwrap_or_default();
            let invariant = if field_valid {
                format!("forced field mutation: {key}")
            } else {
                format!("invalid {key}")
            };
            append_mutation_override(
                &mut document,
                &id,
                &status,
                &status,
                "project-lead",
                reason,
                &invariant,
            )?;
        }
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

/// Result of a governed frontmatter repair: which keys were merged and where.
#[derive(Debug, Clone, Serialize)]
pub struct RepairResult {
    pub id: String,
    pub path: PathBuf,
    pub merged_keys: Vec<String>,
}

/// Governed repair for managed frontmatter corrupted by failed mutations
/// (duplicate top-level keys, e.g. the pre-4.0.2 duplicate `activity:` blocks).
/// Operates textually because the corrupted document cannot parse; refuses any
/// ambiguity (diverging scalars, non-list shapes) and records the repair with
/// its operator-supplied reason in the activity log.
pub fn repair_artifact_frontmatter(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    reason: &str,
) -> Result<RepairResult, TransitionError> {
    let reason = reason.trim();
    if reason.is_empty() {
        return Err(TransitionError::Missing("reason".into()));
    }

    let guard = PathGuard::new(root)?;
    let path = guard.resolve_existing(artifact.as_ref())?;
    let initial = repair_duplicate_top_level_keys(&fs::read_to_string(&path)?)?;
    let Some(repaired) = initial.repaired_source else {
        return Err(TransitionError::Invariant(
            "artifact has no duplicate top-level frontmatter keys; nothing to repair".into(),
        ));
    };
    let id = Document::parse(&repaired)?
        .value("id")
        .ok_or_else(|| TransitionError::Missing("id".into()))?;

    let _lock = ArtifactMutationLock::acquire(guard.root(), &id)?;
    // Re-run the repair on the current content now that the lock is held so a
    // concurrent mutation between the first read and the lock cannot be lost.
    let current = repair_duplicate_top_level_keys(&fs::read_to_string(&path)?)?;
    let Some(repaired) = current.repaired_source else {
        return Err(TransitionError::Invariant(
            "artifact has no duplicate top-level frontmatter keys; nothing to repair".into(),
        ));
    };
    let mut document = Document::parse(&repaired)?;
    let current_id = document
        .value("id")
        .ok_or_else(|| TransitionError::Missing("id".into()))?;
    if current_id != id {
        return Err(TransitionError::Invariant(format!(
            "artifact changed identity while waiting for its mutation lock: expected {id}, found {current_id}"
        )));
    }
    document.set("updated", &today());
    document.append_activity(&format!(
        "repaired duplicate frontmatter keys [{}]: {}",
        current.merged_keys.join(", "),
        reason
    ));
    atomic_write(&path, &document.render())?;
    Ok(RepairResult {
        id,
        path,
        merged_keys: current.merged_keys,
    })
}

pub fn create(
    root: impl AsRef<Path>,
    request: CreateRequest,
) -> Result<MutationResult, TransitionError> {
    if request.kind == ArtifactKind::Debt {
        return Err(TransitionError::Invariant(
            "debts require the semantic debt_create operation".into(),
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
    let index = crate::scan_workspace(root)?;
    let id = format!(
        "{}-{:03}",
        request.kind.prefix(),
        index.next_id(request.kind)
    );
    if index.id_exists(&id) {
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
        if is_list_valued_field(&key) {
            document.set(&key, &normalize_list_field_value(&key, &value)?);
        } else {
            document.set(&key, &value);
        }
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
    } else if id.starts_with("DEBT-") {
        Some(ArtifactKind::Debt)
    } else {
        None
    }
}

pub fn allowed(kind: ArtifactKind, from: &str, to: &str) -> bool {
    kind.is_allowed(from, to)
}

fn invariant_failure(
    root: &Path,
    path: &Path,
    kind: ArtifactKind,
    target: &str,
    document: &Document,
) -> Option<String> {
    match kind {
        ArtifactKind::Spec => {
            let Ok(target_status) = target.parse::<SpecStatus>() else {
                return Some(format!("invalid spec status: {target}"));
            };
            if matches!(target_status, SpecStatus::Ready | SpecStatus::Working) {
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
            if target_status == SpecStatus::Ready {
                if let Err(reason) = invariants::spec_effort_is_declared(document) {
                    return Some(reason);
                }
                if !invariants::recommended_agent_resolves(
                    root,
                    document.value("recommended_agent").as_deref(),
                ) {
                    return Some("recommended_agent does not resolve".into());
                }
            }
            if matches!(target_status, SpecStatus::Review | SpecStatus::Done) {
                let phase = if target_status == SpecStatus::Review {
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
            match target_status {
                SpecStatus::Review => {
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
                SpecStatus::Done => {
                    if !invariants::criteria_complete_with_evidence(&document.body) {
                        Some("a done spec requires its acceptance criteria checked and evidence recorded".into())
                    } else if invariants::waived_findings_are_valid(root, &document.body).is_err() {
                        Some(invariants::waived_findings_are_valid(root, &document.body).unwrap_err())
                    } else if !invariants::spec_has_accepted_review(
                        root,
                        &document.value("id").unwrap_or_default(),
                    ) {
                        Some("a done spec requires an accepted review".into())
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
        ArtifactKind::Review => {
            if let Err(reason) = invariants::implementation_agent_resolves(
                root,
                document.value("implementation_agent").as_deref(),
            ) {
                Some(reason)
            } else {
                None
            }
        }
        ArtifactKind::Handoff => {
            let Ok(target_status) = target.parse::<HandoffStatus>() else {
                return Some(format!("invalid handoff status: {target}"));
            };
            if target_status == HandoffStatus::Ready && !invariants::single_ready_handoff(root, Some(path)) {
                Some("only one ready handoff is allowed".into())
            } else {
                None
            }
        }
        _ => None,
    }
}

fn default_status(kind: ArtifactKind) -> &'static str {
    kind.default_status()
}

fn template_name(kind: ArtifactKind) -> &'static str {
    kind.template_name()
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

// ─── Governed spec metadata (issues #49 and #64) ──────────────────

/// Replaces a spec's descriptive tags. Values are normalized, validated against
/// the spec's own structured fields, and written as one atomic mutation.
pub fn set_spec_tags(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    tags: &[String],
    options: MutationOptions,
) -> Result<MutationResult, TransitionError> {
    set_governed_spec_metadata(root, artifact, options, "set tags", |document, forced| {
        let (normalized, issues) = crate::taxonomy::validate_spec_tags(
            tags,
            document.value("milestone").as_deref(),
            document.value("area").as_deref(),
            document.value("priority").as_deref(),
        );
        if !issues.is_empty() && !forced {
            return Err(issues
                .iter()
                .map(crate::taxonomy::SpecTagIssue::message)
                .collect::<Vec<_>>()
                .join("; "));
        }
        document.set("tags", &render_inline_array(&normalized));
        Ok(if issues.is_empty() {
            None
        } else {
            Some("invalid tags".to_string())
        })
    })
}

/// Sets the Lead-owned implementation estimate. `level` defaults from the tier
/// when omitted, so a Lead states one decision rather than two.
pub fn set_spec_effort(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    tier: &str,
    level: Option<&str>,
    options: MutationOptions,
) -> Result<MutationResult, TransitionError> {
    set_governed_spec_metadata(root, artifact, options, "set effort", |document, forced| {
        let Some(tier) = crate::taxonomy::normalize_capability_tier(tier) else {
            return Err(format!(
                "unknown capability tier `{tier}`; expected one of {}",
                crate::taxonomy::capability_tiers().join(", ")
            ));
        };
        let level = match level {
            Some(raw) => crate::taxonomy::normalize_thinking_level(raw).ok_or_else(|| {
                format!(
                    "unknown thinking level `{raw}`; expected one of {}",
                    crate::taxonomy::thinking_levels().join(", ")
                )
            })?,
            None => crate::taxonomy::default_thinking_level(&tier).to_string(),
        };
        let constrained = crate::taxonomy::thinking_level_allowed(&tier, &level);
        if let Err(reason) = &constrained {
            if !forced {
                return Err(reason.clone());
            }
        }
        document.set("capability_tier", &tier);
        document.set("thinking_level", &level);
        Ok(constrained
            .err()
            .map(|_| "constrained effort combination".to_string()))
    })
}

/// Appends a specialist's observation of the effort the work actually required.
/// It is evidence for a later Lead revision and never rewrites the estimate.
pub fn record_effort_observation(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    observed_tier: &str,
    actor: &str,
    note: &str,
    options: MutationOptions,
) -> Result<MutationResult, TransitionError> {
    let observed = crate::taxonomy::normalize_capability_tier(observed_tier).ok_or_else(|| {
        TransitionError::Invariant(format!(
            "unknown capability tier `{observed_tier}`; expected one of {}",
            crate::taxonomy::capability_tiers().join(", ")
        ))
    })?;
    let actor = actor.trim();
    let note = note.trim();
    if actor.is_empty() {
        return Err(TransitionError::Missing("actor".into()));
    }
    if note.is_empty() {
        return Err(TransitionError::Missing("note".into()));
    }

    set_governed_spec_metadata(
        root,
        artifact,
        options,
        "record effort observation",
        |document, _| {
            let recommended = document.value("capability_tier").unwrap_or_default();
            document
                .append_object(
                    "effort_observations",
                    &[
                        ("timestamp".into(), serde_json::Value::String(today())),
                        ("actor".into(), serde_json::Value::String(actor.into())),
                        (
                            "observed_tier".into(),
                            serde_json::Value::String(observed.clone()),
                        ),
                        (
                            "recommended_tier".into(),
                            serde_json::Value::String(recommended),
                        ),
                        ("note".into(), serde_json::Value::String(note.into())),
                    ],
                )
                .map_err(|error| error.to_string())?;
            Ok(None)
        },
    )
}

fn render_inline_array(values: &[String]) -> String {
    format!("[{}]", values.join(", "))
}

/// Shared body for the governed spec-metadata mutations: the same locking,
/// identity, concurrency, and audit guarantees as `set_field`, with a
/// caller-supplied edit that reports whether it violated an invariant.
fn set_governed_spec_metadata(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    options: MutationOptions,
    activity: &str,
    edit: impl Fn(&mut Document, bool) -> Result<Option<String>, String>,
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
    if kind_for_id(&id) != Some(ArtifactKind::Spec) {
        return Err(TransitionError::Invariant(
            "expected a spec artifact".into(),
        ));
    }

    let violated = edit(&mut document, options.force).map_err(TransitionError::Invariant)?;

    document.set("updated", &today());
    document.append_activity(activity);
    if options.force {
        if let Some(reason) = options.reason.as_deref() {
            let status = document.value("status").unwrap_or_default();
            let invariant = violated.unwrap_or_else(|| format!("forced mutation: {activity}"));
            append_mutation_override(
                &mut document,
                &id,
                &status,
                &status,
                "project-lead",
                reason,
                &invariant,
            )?;
        }
    }

    if fs::read_to_string(&path)? != current_source {
        return Err(TransitionError::Invariant(
            "artifact changed while the metadata mutation was being prepared".into(),
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

/// Retire `superseded_id` in favour of the ADR at `artifact`, writing both
/// sides of the relationship (issue #48).
///
/// Two files cannot be written atomically without a journal. The design makes
/// the partial state benign instead: both artifacts are locked before either is
/// read (in lexicographic ID order, so concurrent supersessions cannot
/// deadlock), every check runs before any write, and the *superseding* ADR is
/// written first. A crash between the two writes therefore leaves a one-sided
/// claim -- which `diagnose_decisions` reports and re-running this verb repairs.
/// The opposite order would strip a decision of its authority with no successor
/// recorded anywhere, a silent loss no check could see.
///
/// The operation is idempotent: re-running it on an already-consistent pair
/// succeeds without touching either file.
pub fn supersede_adr(
    root: impl AsRef<Path>,
    artifact: impl AsRef<Path>,
    superseded_id: &str,
    options: MutationOptions,
) -> Result<MutationResult, TransitionError> {
    require_force_reason(&options)?;

    let guard = PathGuard::new(root)?;
    let superseding_path = guard.resolve_existing(artifact.as_ref())?;
    let superseding_id = Document::parse(&fs::read_to_string(&superseding_path)?)?
        .value("id")
        .ok_or_else(|| TransitionError::Missing("id".into()))?;
    if kind_for_id(&superseding_id) != Some(ArtifactKind::Adr) {
        return Err(TransitionError::Invariant(
            "expected a decision artifact".into(),
        ));
    }

    let superseded_id = superseded_id.trim().to_ascii_uppercase();
    if kind_for_id(&superseded_id) != Some(ArtifactKind::Adr) {
        return Err(TransitionError::Invariant(format!(
            "{superseded_id} is not a decision ID"
        )));
    }
    if superseded_id == superseding_id {
        return Err(TransitionError::Invariant(
            "a decision cannot supersede itself".into(),
        ));
    }
    let superseded_path = decision_path_for_id(guard.root(), &superseded_id).ok_or_else(|| {
        TransitionError::Invariant(format!("{superseded_id} does not exist in this workspace"))
    })?;
    let superseded_path = guard.resolve_existing(&superseded_path)?;

    // Lock both artifacts before reading either, ordered by ID so two
    // concurrent supersessions acquire them in the same sequence.
    let (first, second) = if superseding_id <= superseded_id {
        (superseding_id.as_str(), superseded_id.as_str())
    } else {
        (superseded_id.as_str(), superseding_id.as_str())
    };
    let _first_lock = ArtifactMutationLock::acquire(guard.root(), first)?;
    let _second_lock = ArtifactMutationLock::acquire(guard.root(), second)?;

    let superseding_source = fs::read_to_string(&superseding_path)?;
    let mut superseding = Document::parse(&superseding_source)?;
    let superseded_source = fs::read_to_string(&superseded_path)?;
    let mut superseded = Document::parse(&superseded_source)?;

    for (document, expected) in [
        (&superseding, &superseding_id),
        (&superseded, &superseded_id),
    ] {
        let actual = document
            .value("id")
            .ok_or_else(|| TransitionError::Missing("id".into()))?;
        if &actual != expected {
            return Err(TransitionError::Invariant(format!(
                "artifact changed identity while waiting for its mutation lock: expected {expected}, found {actual}"
            )));
        }
    }

    let superseding_status = superseding.value("status").unwrap_or_default();
    let superseded_status = superseded.value("status").unwrap_or_default();
    let mut declares = superseding.string_array("supersedes");
    let mut retired_by = superseded.string_array("superseded_by");

    // Already consistent on both sides: nothing to do.
    if superseded_status == "superseded"
        && declares.iter().any(|value| value == &superseded_id)
        && retired_by.iter().any(|value| value == &superseding_id)
    {
        return Ok(MutationResult {
            id: superseding_id,
            status: superseding_status,
            path: superseding_path,
            forced: options.force,
        });
    }

    let violated = if superseding_status != "accepted" {
        Some(format!(
            "{superseding_id} is {superseding_status}: a decision must be accepted before it can supersede another"
        ))
    } else if !matches!(superseded_status.as_str(), "accepted" | "superseded") {
        Some(format!(
            "{superseded_id} is {superseded_status}: only an accepted decision can be superseded"
        ))
    } else {
        None
    };
    if let Some(reason) = violated.as_deref() {
        if !options.force {
            return Err(TransitionError::Invariant(reason.into()));
        }
    }

    let activity = format!("supersede {superseded_id}");

    if !declares.iter().any(|value| value == &superseded_id) {
        declares.push(superseded_id.clone());
    }
    superseding.set("supersedes", &render_inline_array(&declares));
    superseding.set("updated", &today());
    superseding.append_activity(&activity);

    if !retired_by.iter().any(|value| value == &superseding_id) {
        retired_by.push(superseding_id.clone());
    }
    superseded.set("superseded_by", &render_inline_array(&retired_by));
    superseded.set("status", "superseded");
    superseded.set("updated", &today());
    superseded.append_activity(&format!("superseded by {superseding_id}"));

    if options.force {
        if let Some(reason) = options.reason.as_deref() {
            let invariant = violated
                .clone()
                .unwrap_or_else(|| format!("forced mutation: {activity}"));
            append_mutation_override(
                &mut superseding,
                &superseding_id,
                &superseding_status,
                &superseding_status,
                "project-lead",
                reason,
                &invariant,
            )?;
            append_mutation_override(
                &mut superseded,
                &superseded_id,
                &superseded_status,
                "superseded",
                "project-lead",
                reason,
                &invariant,
            )?;
        }
    }

    if fs::read_to_string(&superseding_path)? != superseding_source
        || fs::read_to_string(&superseded_path)? != superseded_source
    {
        return Err(TransitionError::Invariant(
            "a decision changed while the supersession was being prepared".into(),
        ));
    }

    atomic_write(&superseding_path, &superseding.render())?;
    atomic_write(&superseded_path, &superseded.render())?;

    Ok(MutationResult {
        id: superseding_id,
        status: superseding.value("status").unwrap_or_default(),
        path: superseding_path,
        forced: options.force,
    })
}

/// Decisions live flat in `.lmbrain/decisions/`, so locating one by ID is a
/// single directory scan rather than a status-folder walk.
fn decision_path_for_id(root: &Path, id: &str) -> Option<PathBuf> {
    let entries = fs::read_dir(root.join(".lmbrain/decisions")).ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let matches = fs::read_to_string(&path)
            .ok()
            .and_then(|source| Document::parse(&source).ok())
            .and_then(|document| document.value("id"))
            .is_some_and(|value| value == id);
        if matches {
            return Some(path);
        }
    }
    None
}
