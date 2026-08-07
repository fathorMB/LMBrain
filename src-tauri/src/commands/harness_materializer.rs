//! Atomic materialization of approved harness configuration into native host
//! files. The engine lives in `lmbrain_core::harness_environment` (#87) so the
//! MCP server enforces the same staging, preservation, rollback, and drift
//! rules; this module only re-exports it for the Tauri command surface.

pub use lmbrain_core::harness_environment::{
    apply_harness_configuration, detect_drift, AppliedNativeFile, HarnessApplyResult,
    HarnessDriftEntry,
};
