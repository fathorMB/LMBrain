use lmbrain_lib::commands::workspace::WorkspaceService;
use lmbrain_lib::models::workspace::KitMigrationStatus;
use std::fs;
use std::path::Path;

fn setup_test_workspace(dir: &Path, project_version: Option<&str>) {
    let lmbrain = dir.join(".lmbrain");
    fs::create_dir_all(&lmbrain).unwrap();
    if let Some(v) = project_version {
        fs::write(lmbrain.join("VERSION"), v).unwrap();
    }
    fs::write(lmbrain.join("STATUS.md"), "## Current focus\nTest\n").unwrap();
}

fn setup_bundled_kit(dir: &Path, bundled_version: Option<&str>, migrations_content: Option<&str>) {
    let lmbrain = dir.join(".lmbrain");
    fs::create_dir_all(&lmbrain).unwrap();
    if let Some(v) = bundled_version {
        fs::write(lmbrain.join("VERSION"), v).unwrap();
    }
    if let Some(content) = migrations_content {
        fs::write(lmbrain.join("MIGRATIONS.md"), content).unwrap();
    }
}

#[test]
fn test_migration_status_up_to_date() {
    let workspace_dir = tempfile::tempdir().unwrap();
    let bundled_dir = tempfile::tempdir().unwrap();

    setup_test_workspace(workspace_dir.path(), Some("2.2.7"));
    setup_bundled_kit(bundled_dir.path(), Some("2.2.7"), None);

    let service = WorkspaceService::new();
    let info = service
        .validate_workspace(
            &workspace_dir.path().to_string_lossy(),
            Some(&bundled_dir.path().join(".lmbrain")),
        )
        .unwrap();

    assert_eq!(info.project_kit_version, "2.2.7");
    assert_eq!(info.bundled_kit_version, "2.2.7");
    assert!(info.bundled_kit_path.ends_with(".lmbrain"));
    assert_eq!(info.kit_migration_status, KitMigrationStatus::UpToDate);
}

#[test]
fn test_migration_status_available_with_guidance() {
    let workspace_dir = tempfile::tempdir().unwrap();
    let bundled_dir = tempfile::tempdir().unwrap();

    setup_test_workspace(workspace_dir.path(), Some("2.1.2"));
    setup_bundled_kit(
        bundled_dir.path(),
        Some("2.2.7"),
        Some("# Kit Migrations\n\n### 2.2.7 (v3 context)\nGuidance here\n"),
    );

    let service = WorkspaceService::new();
    let info = service
        .validate_workspace(
            &workspace_dir.path().to_string_lossy(),
            Some(&bundled_dir.path().join(".lmbrain")),
        )
        .unwrap();

    assert_eq!(info.project_kit_version, "2.1.2");
    assert_eq!(info.bundled_kit_version, "2.2.7");
    assert!(info.bundled_kit_path.ends_with(".lmbrain"));
    assert_eq!(
        info.kit_migration_status,
        KitMigrationStatus::MigrationAvailable
    );
}

#[test]
fn test_bundled_kit_path_display_strips_windows_extended_prefix() {
    let dir = tempfile::tempdir().unwrap();
    setup_test_workspace(dir.path(), Some("2.1.2"));

    let extended = Path::new(r"\\?\C:\Program Files\LMBrain\kit\.lmbrain");
    let service = WorkspaceService::new();
    let info = service
        .validate_workspace(&dir.path().to_string_lossy(), Some(extended))
        .unwrap();

    assert_eq!(
        info.bundled_kit_path,
        "C:/Program Files/LMBrain/kit/.lmbrain"
    );
}

#[test]
fn test_migration_status_guidance_missing() {
    let workspace_dir = tempfile::tempdir().unwrap();
    let bundled_dir = tempfile::tempdir().unwrap();

    setup_test_workspace(workspace_dir.path(), Some("2.1.2"));
    // MIGRATIONS.md exists but has no ### 2.2.7 section
    setup_bundled_kit(
        bundled_dir.path(),
        Some("2.2.7"),
        Some("# Kit Migrations\n\n### 1.1.0\nGuidance here\n"),
    );

    let service = WorkspaceService::new();
    let info = service
        .validate_workspace(
            &workspace_dir.path().to_string_lossy(),
            Some(&bundled_dir.path().join(".lmbrain")),
        )
        .unwrap();

    assert_eq!(
        info.kit_migration_status,
        KitMigrationStatus::MigrationGuidanceMissing
    );
}

#[test]
fn test_migration_status_project_newer() {
    let workspace_dir = tempfile::tempdir().unwrap();
    let bundled_dir = tempfile::tempdir().unwrap();

    setup_test_workspace(workspace_dir.path(), Some("2.3.0"));
    setup_bundled_kit(bundled_dir.path(), Some("2.2.7"), None);

    let service = WorkspaceService::new();
    let info = service
        .validate_workspace(
            &workspace_dir.path().to_string_lossy(),
            Some(&bundled_dir.path().join(".lmbrain")),
        )
        .unwrap();

    assert_eq!(
        info.kit_migration_status,
        KitMigrationStatus::ProjectNewerThanApp
    );
}

#[test]
fn test_migration_status_unknown_project_version() {
    let workspace_dir = tempfile::tempdir().unwrap();
    let bundled_dir = tempfile::tempdir().unwrap();

    // No VERSION file in project
    setup_test_workspace(workspace_dir.path(), None);
    setup_bundled_kit(bundled_dir.path(), Some("2.2.7"), None);

    let service = WorkspaceService::new();
    let info = service
        .validate_workspace(
            &workspace_dir.path().to_string_lossy(),
            Some(&bundled_dir.path().join(".lmbrain")),
        )
        .unwrap();

    assert_eq!(
        info.kit_migration_status,
        KitMigrationStatus::UnknownProjectVersion
    );
}

#[test]
fn test_migration_status_unparsable_versions() {
    let workspace_dir = tempfile::tempdir().unwrap();
    let bundled_dir = tempfile::tempdir().unwrap();

    setup_test_workspace(workspace_dir.path(), Some("not_semver"));
    setup_bundled_kit(bundled_dir.path(), Some("2.2.7"), None);

    let service = WorkspaceService::new();
    let info = service
        .validate_workspace(
            &workspace_dir.path().to_string_lossy(),
            Some(&bundled_dir.path().join(".lmbrain")),
        )
        .unwrap();

    assert_eq!(
        info.kit_migration_status,
        KitMigrationStatus::UnknownProjectVersion
    );
}
