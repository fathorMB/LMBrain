//! Deterministic native-file planning for the governed project environment.
//! The engine lives in `lmbrain_core::harness_environment` (#87) so the MCP
//! server exposes the identical plan the app displays; this module only
//! re-exports it for the Tauri command surface.

pub use lmbrain_core::harness_environment::{
    plan_harness_configuration, BrowserMcpReadiness, HarnessConfigurationPlan, HostPlan,
    LspReadiness, NativeFilePreview, PreviewAction, ToolReadiness,
};
