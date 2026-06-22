use lmbrain_lib::commands::filesystem::{clean_path, PathGuard};
use std::fs;

fn setup_test_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temp dir")
}

#[test]
fn test_path_guard_initial_state() {
    let guard = PathGuard::new();
    assert!(guard.get_root().is_none());
}

#[test]
fn test_path_guard_set_root() {
    let guard = PathGuard::new();
    let dir = setup_test_dir();
    guard.set_root(dir.path());
    assert!(guard.get_root().is_some());
}

#[test]
fn test_path_guard_resolve_within_root() {
    let guard = PathGuard::new();
    let dir = setup_test_dir();

    // Create a file within the root
    let file_path = dir.path().join("test.md");
    fs::write(&file_path, "hello").unwrap();

    guard.set_root(dir.path());
    let resolved = guard.resolve("test.md").unwrap();
    assert_eq!(resolved, clean_path(&file_path.canonicalize().unwrap()));
}

#[test]
fn test_path_guard_resolve_absolute_within_root() {
    let guard = PathGuard::new();
    let dir = setup_test_dir();

    let file_path = dir.path().join("test.md");
    fs::write(&file_path, "hello").unwrap();

    guard.set_root(dir.path());
    let resolved = guard.resolve(&file_path.to_string_lossy()).unwrap();
    assert_eq!(resolved, clean_path(&file_path.canonicalize().unwrap()));
}

#[test]
fn test_path_guard_rejects_traversal() {
    let guard = PathGuard::new();
    let dir = setup_test_dir();

    guard.set_root(dir.path());
    let result = guard.resolve("../outside");
    assert!(result.is_err());
}

#[test]
fn test_path_guard_rejects_nonexistent() {
    let guard = PathGuard::new();
    let dir = setup_test_dir();

    guard.set_root(dir.path());
    let result = guard.resolve("nonexistent.md");
    assert!(result.is_err());
}

#[test]
fn test_path_guard_read_file() {
    let guard = PathGuard::new();
    let dir = setup_test_dir();

    let file_path = dir.path().join("hello.md");
    fs::write(&file_path, "Hello, world!").unwrap();

    guard.set_root(dir.path());
    let content = guard.read_file("hello.md").unwrap();
    assert_eq!(content.content, "Hello, world!");
}

#[test]
fn test_path_guard_read_file_outside_root() {
    let guard = PathGuard::new();
    let dir = setup_test_dir();
    let outside = setup_test_dir();

    let file_path = outside.path().join("secret.md");
    fs::write(&file_path, "secret").unwrap();

    guard.set_root(dir.path());
    let result = guard.read_file(&file_path.to_string_lossy());
    assert!(result.is_err());
}

#[test]
fn test_path_guard_list_directory() {
    let guard = PathGuard::new();
    let dir = setup_test_dir();

    fs::write(dir.path().join("a.md"), "a").unwrap();
    fs::write(dir.path().join("b.md"), "b").unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();

    guard.set_root(dir.path());
    let entries = guard.list_directory(".").unwrap();
    assert!(entries.len() >= 3);
}
