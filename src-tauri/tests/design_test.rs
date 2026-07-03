use std::fs;

use lmbrain_lib::commands::design::{
    read_design_asset, read_design_html, read_design_preview_html, scan_design_mockups,
};

#[test]
fn scan_design_mockups_finds_packages_and_html_files() {
    let dir = tempfile::tempdir().unwrap();
    let design_dir = dir.path().join(".lmbrain/design");
    let package = design_dir.join("checkout-flow");
    fs::create_dir_all(&package).unwrap();
    fs::write(package.join("index.html"), "<html>checkout</html>").unwrap();
    fs::write(
        package.join("manifest.json"),
        r#"{"title":"Checkout Flow","description":"Responsive checkout."}"#,
    )
    .unwrap();
    fs::write(design_dir.join("single.html"), "<html>single</html>").unwrap();

    let mockups = scan_design_mockups(dir.path()).unwrap();

    assert_eq!(mockups.len(), 2);
    assert!(mockups
        .iter()
        .any(|mockup| mockup.manifest_title.as_deref() == Some("Checkout Flow")));
    assert!(mockups
        .iter()
        .any(|mockup| mockup.entry_path == ".lmbrain/design/single.html"));
}

#[test]
fn read_design_html_rejects_entries_outside_design_tree() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".lmbrain/design")).unwrap();
    fs::write(dir.path().join("outside.html"), "<html>secret</html>").unwrap();

    let result = read_design_html(dir.path(), dir.path().join("outside.html").as_path());

    assert!(result.is_err());
}

#[test]
fn read_design_html_rejects_non_html_entries() {
    let dir = tempfile::tempdir().unwrap();
    let design_dir = dir.path().join(".lmbrain/design");
    fs::create_dir_all(&design_dir).unwrap();
    fs::write(design_dir.join("notes.txt"), "not html").unwrap();

    let result = read_design_html(dir.path(), &design_dir.join("notes.txt"));

    assert!(result.is_err());
}

#[test]
fn read_design_asset_serves_files_under_design_tree() {
    let dir = tempfile::tempdir().unwrap();
    let design_dir = dir.path().join(".lmbrain/design/mockup");
    fs::create_dir_all(&design_dir).unwrap();
    fs::write(design_dir.join("app.js"), "console.log('ok')").unwrap();

    let asset = read_design_asset(dir.path(), "/.lmbrain/design/mockup/app.js").unwrap();

    assert_eq!(asset.mime_type, "text/javascript; charset=utf-8");
    assert_eq!(asset.content, b"console.log('ok')");
}

#[test]
fn read_design_asset_rejects_traversal_outside_design_tree() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join(".lmbrain/design/mockup")).unwrap();
    fs::write(dir.path().join(".lmbrain/STATUS.md"), "secret").unwrap();

    let result = read_design_asset(dir.path(), "/.lmbrain/design/mockup/../../STATUS.md");

    assert!(result.is_err());
}

#[test]
fn read_design_preview_html_inlines_local_stylesheets_and_scripts() {
    let dir = tempfile::tempdir().unwrap();
    let design_dir = dir.path().join(".lmbrain/design/mockup");
    let assets = design_dir.join("assets");
    fs::create_dir_all(&assets).unwrap();
    fs::write(assets.join("design-system.css"), ".app { color: red; }").unwrap();
    fs::write(assets.join("data.js"), "window.DATA = { ok: true };").unwrap();
    fs::write(
        assets.join("app.js"),
        "document.body.dataset.ready = 'true';",
    )
    .unwrap();
    fs::write(
        design_dir.join("index.html"),
        r#"<!doctype html>
<html>
<head><link rel="stylesheet" href="assets/design-system.css"></head>
<body>
<script src="assets/data.js"></script>
<script src="assets/app.js"></script>
</body>
</html>"#,
    )
    .unwrap();

    let preview = read_design_preview_html(dir.path(), &design_dir.join("index.html")).unwrap();

    assert!(preview
        .content
        .contains("<style data-lmbrain-inline=\"assets/design-system.css\""));
    assert!(preview.content.contains(".app { color: red; }"));
    assert!(preview
        .content
        .contains("<script data-lmbrain-inline=\"assets/data.js\""));
    assert!(preview.content.contains("window.DATA = { ok: true };"));
    assert!(preview
        .content
        .contains("<script data-lmbrain-inline=\"assets/app.js\""));
    assert!(!preview
        .content
        .contains("href=\"assets/design-system.css\""));
    assert!(!preview.content.contains("src=\"assets/data.js\""));
    assert!(!preview.content.contains("src=\"assets/app.js\""));
}
