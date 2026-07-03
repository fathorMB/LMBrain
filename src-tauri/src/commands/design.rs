use std::{fs, path::Path, time::SystemTime};

use percent_encoding::percent_decode_str;
use serde::Deserialize;

use crate::{
    errors::AppError,
    models::design::{DesignMockup, DesignMockupHtml, DesignMockupKind},
};

const DESIGN_DIR: &str = ".lmbrain/design";

#[derive(Debug, Deserialize)]
struct DesignManifest {
    title: Option<String>,
    description: Option<String>,
}

pub fn scan_design_mockups(root: &Path) -> Result<Vec<DesignMockup>, AppError> {
    let design_root = root.join(DESIGN_DIR);
    if !design_root.exists() {
        return Ok(Vec::new());
    }
    if !design_root.is_dir() {
        return Err(AppError::InvalidKit(format!(
            "{} is not a directory",
            design_root.display()
        )));
    }

    let mut mockups = Vec::new();
    for entry in fs::read_dir(&design_root)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }

        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            let entry_path = path.join("index.html");
            if entry_path.is_file() {
                mockups.push(build_package(root, &path, &entry_path, name)?);
            }
        } else if file_type.is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some("html")
        {
            mockups.push(build_html_file(root, &path, name)?);
        }
    }

    mockups.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(mockups)
}

pub fn read_design_html(root: &Path, entry_path: &Path) -> Result<DesignMockupHtml, AppError> {
    let resolved = resolve_design_entry(root, entry_path)?;

    Ok(DesignMockupHtml {
        path: resolved.to_string_lossy().to_string(),
        content: fs::read_to_string(&resolved)?,
    })
}

pub fn read_design_preview_html(
    root: &Path,
    entry_path: &Path,
) -> Result<DesignMockupHtml, AppError> {
    let design_root = root.join(DESIGN_DIR).canonicalize().map_err(|_| {
        AppError::PathSafety(format!("Design directory does not exist: {DESIGN_DIR}"))
    })?;
    let resolved = resolve_design_entry(root, entry_path)?;
    let content = fs::read_to_string(&resolved)?;
    let entry_dir = resolved.parent().ok_or_else(|| {
        AppError::PathSafety(format!(
            "Design entry has no parent directory: {}",
            entry_path.display()
        ))
    })?;

    Ok(DesignMockupHtml {
        path: resolved.to_string_lossy().to_string(),
        content: inline_preview_assets(&design_root, entry_dir, &content)?,
    })
}

fn resolve_design_entry(root: &Path, entry_path: &Path) -> Result<std::path::PathBuf, AppError> {
    let design_root = root.join(DESIGN_DIR).canonicalize().map_err(|_| {
        AppError::PathSafety(format!("Design directory does not exist: {DESIGN_DIR}"))
    })?;
    let path = if entry_path.is_absolute() {
        entry_path.to_path_buf()
    } else {
        root.join(entry_path)
    };
    let resolved = path.canonicalize().map_err(|_| {
        AppError::PathSafety(format!(
            "Design entry does not exist: {}",
            entry_path.display()
        ))
    })?;

    if !resolved.starts_with(&design_root) {
        return Err(AppError::PathSafety(format!(
            "Design entry is outside {DESIGN_DIR}: {}",
            entry_path.display()
        )));
    }
    if resolved.extension().and_then(|ext| ext.to_str()) != Some("html") {
        return Err(AppError::PathSafety(format!(
            "Design entry is not an HTML file: {}",
            entry_path.display()
        )));
    }

    Ok(resolved)
}

pub struct DesignAsset {
    pub content: Vec<u8>,
    pub mime_type: String,
}

pub fn read_design_asset(root: &Path, request_path: &str) -> Result<DesignAsset, AppError> {
    let design_root = root.join(DESIGN_DIR).canonicalize().map_err(|_| {
        AppError::PathSafety(format!("Design directory does not exist: {DESIGN_DIR}"))
    })?;
    let decoded = percent_decode_str(request_path.trim_start_matches('/'))
        .decode_utf8()
        .map_err(|error| AppError::PathSafety(format!("Invalid design path encoding: {error}")))?;
    let decoded = decoded.replace('\\', "/");
    let path = Path::new(decoded.trim_start_matches('/'));

    if path.is_absolute() {
        return Err(AppError::PathSafety(format!(
            "Design asset path must be workspace-relative: {decoded}"
        )));
    }

    let resolved = root
        .join(path)
        .canonicalize()
        .map_err(|_| AppError::PathSafety(format!("Design asset does not exist: {decoded}")))?;

    if !resolved.starts_with(&design_root) {
        return Err(AppError::PathSafety(format!(
            "Design asset is outside {DESIGN_DIR}: {decoded}"
        )));
    }
    if !resolved.is_file() {
        return Err(AppError::FileNotFound(format!(
            "Design asset is not a file: {decoded}"
        )));
    }

    Ok(DesignAsset {
        mime_type: mime_type(&resolved).to_string(),
        content: fs::read(&resolved)?,
    })
}

fn build_package(
    root: &Path,
    path: &Path,
    entry_path: &Path,
    name: String,
) -> Result<DesignMockup, AppError> {
    let readme = path.join("README.md");
    let manifest = path.join("manifest.json");
    let parsed_manifest = read_manifest(&manifest);
    let summary = parsed_manifest
        .as_ref()
        .and_then(|manifest| manifest.description.clone())
        .or_else(|| read_summary(&readme));

    Ok(DesignMockup {
        id: slug(&name),
        name,
        path: relative(root, path),
        entry_path: relative(root, entry_path),
        kind: DesignMockupKind::Package,
        modified: modified(path),
        size: directory_size(path),
        summary,
        manifest_title: parsed_manifest
            .as_ref()
            .and_then(|manifest| manifest.title.clone()),
        manifest_description: parsed_manifest.and_then(|manifest| manifest.description),
        has_manifest: manifest.is_file(),
        has_readme: readme.is_file(),
    })
}

fn build_html_file(root: &Path, path: &Path, name: String) -> Result<DesignMockup, AppError> {
    Ok(DesignMockup {
        id: slug(name.trim_end_matches(".html")),
        name: name.trim_end_matches(".html").to_string(),
        path: relative(root, path),
        entry_path: relative(root, path),
        kind: DesignMockupKind::HtmlFile,
        modified: modified(path),
        size: fs::metadata(path)
            .map(|metadata| metadata.len())
            .unwrap_or(0),
        summary: None,
        manifest_title: None,
        manifest_description: None,
        has_manifest: false,
        has_readme: false,
    })
}

fn inline_preview_assets(
    design_root: &Path,
    entry_dir: &Path,
    html: &str,
) -> Result<String, AppError> {
    let html = inline_stylesheets(design_root, entry_dir, html)?;
    inline_scripts(design_root, entry_dir, &html)
}

fn inline_stylesheets(
    design_root: &Path,
    entry_dir: &Path,
    html: &str,
) -> Result<String, AppError> {
    let mut output = String::new();
    let mut cursor = 0;

    while let Some(start_rel) = find_ascii_ci(&html[cursor..], "<link") {
        let start = cursor + start_rel;
        let Some(end_rel) = html[start..].find('>') else {
            break;
        };
        let end = start + end_rel + 1;
        let tag = &html[start..end];

        output.push_str(&html[cursor..start]);
        if attr_contains_ascii_ci(tag, "rel", "stylesheet") {
            if let Some(href) = attr_value(tag, "href") {
                if let Some(css) = read_local_text_asset(design_root, entry_dir, &href)? {
                    output.push_str("<style data-lmbrain-inline=\"");
                    output.push_str(&escape_html_attr(&href));
                    output.push_str("\">\n");
                    output.push_str(&css);
                    output.push_str("\n</style>");
                } else {
                    output.push_str(tag);
                }
            } else {
                output.push_str(tag);
            }
        } else {
            output.push_str(tag);
        }
        cursor = end;
    }

    output.push_str(&html[cursor..]);
    Ok(output)
}

fn inline_scripts(design_root: &Path, entry_dir: &Path, html: &str) -> Result<String, AppError> {
    let mut output = String::new();
    let mut cursor = 0;

    while let Some(start_rel) = find_ascii_ci(&html[cursor..], "<script") {
        let start = cursor + start_rel;
        let Some(open_end_rel) = html[start..].find('>') else {
            break;
        };
        let open_end = start + open_end_rel + 1;
        let tag = &html[start..open_end];
        let Some(close_start_rel) = find_ascii_ci(&html[open_end..], "</script>") else {
            break;
        };
        let close_end = open_end + close_start_rel + "</script>".len();

        output.push_str(&html[cursor..start]);
        if let Some(src) = attr_value(tag, "src") {
            if let Some(js) = read_local_text_asset(design_root, entry_dir, &src)? {
                output.push_str("<script");
                if let Some(script_type) = attr_value(tag, "type") {
                    output.push_str(" type=\"");
                    output.push_str(&escape_html_attr(&script_type));
                    output.push('"');
                }
                output.push_str(" data-lmbrain-inline=\"");
                output.push_str(&escape_html_attr(&src));
                output.push_str("\">\n");
                output.push_str(&js);
                output.push_str("\n</script>");
            } else {
                output.push_str(&html[start..close_end]);
            }
        } else {
            output.push_str(&html[start..close_end]);
        }
        cursor = close_end;
    }

    output.push_str(&html[cursor..]);
    Ok(output)
}

fn read_local_text_asset(
    design_root: &Path,
    entry_dir: &Path,
    reference: &str,
) -> Result<Option<String>, AppError> {
    let reference = reference.trim();
    if reference.is_empty()
        || reference.starts_with('#')
        || reference.starts_with("//")
        || reference.starts_with("data:")
        || reference.starts_with("blob:")
        || reference.contains("://")
    {
        return Ok(None);
    }

    let reference = reference
        .split_once('#')
        .map(|(path, _)| path)
        .unwrap_or(reference)
        .split_once('?')
        .map(|(path, _)| path)
        .unwrap_or(reference);
    let resolved = entry_dir
        .join(reference.replace('\\', "/"))
        .canonicalize()
        .map_err(|_| AppError::FileNotFound(format!("Design asset not found: {reference}")))?;

    if !resolved.starts_with(design_root) {
        return Err(AppError::PathSafety(format!(
            "Design asset is outside {DESIGN_DIR}: {reference}"
        )));
    }
    if !resolved.is_file() {
        return Err(AppError::FileNotFound(format!(
            "Design asset is not a file: {reference}"
        )));
    }

    Ok(Some(fs::read_to_string(resolved)?))
}

fn find_ascii_ci(haystack: &str, needle: &str) -> Option<usize> {
    haystack.to_lowercase().find(&needle.to_lowercase())
}

fn attr_value(tag: &str, name: &str) -> Option<String> {
    let lower = tag.to_lowercase();
    let pattern = format!("{}=", name.to_lowercase());
    let start = lower.find(&pattern)? + pattern.len();
    let rest = tag[start..].trim_start();
    let quote = rest.chars().next()?;
    if quote == '"' || quote == '\'' {
        let value_start = quote.len_utf8();
        let value_end = rest[value_start..].find(quote)? + value_start;
        Some(rest[value_start..value_end].to_string())
    } else {
        Some(
            rest.split_whitespace()
                .next()
                .unwrap_or_default()
                .trim_end_matches('>')
                .to_string(),
        )
    }
}

fn attr_contains_ascii_ci(tag: &str, name: &str, expected: &str) -> bool {
    attr_value(tag, name)
        .map(|value| value.to_lowercase().contains(&expected.to_lowercase()))
        .unwrap_or(false)
}

fn escape_html_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn read_manifest(path: &Path) -> Option<DesignManifest> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

fn read_summary(path: &Path) -> Option<String> {
    let content = fs::read_to_string(path).ok()?;
    content
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| truncate(line, 180))
}

fn directory_size(path: &Path) -> u64 {
    let Ok(entries) = fs::read_dir(path) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| {
            let path = entry.path();
            if path.is_dir() {
                directory_size(&path)
            } else {
                entry.metadata().map(|metadata| metadata.len()).unwrap_or(0)
            }
        })
        .sum()
}

fn modified(path: &Path) -> Option<String> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    format_system_time(modified)
}

fn format_system_time(modified: SystemTime) -> Option<String> {
    let elapsed = SystemTime::now().duration_since(modified).ok()?;
    let total_minutes = elapsed.as_secs() / 60;
    let days = total_minutes / (24 * 60);
    let hours = (total_minutes % (24 * 60)) / 60;
    let minutes = total_minutes % 60;
    Some(format!("{days}d {hours}h {minutes}m ago"))
}

fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .ok()
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn slug(input: &str) -> String {
    input
        .to_lowercase()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn truncate(input: &str, max: usize) -> String {
    if input.len() <= max {
        input.to_string()
    } else {
        format!("{}...", &input[..max])
    }
}

fn mime_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| extension.to_ascii_lowercase())
        .as_deref()
    {
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("json" | "map") => "application/json; charset=utf-8",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("ico") => "image/x-icon",
        Some("wasm") => "application/wasm",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("otf") => "font/otf",
        _ => "application/octet-stream",
    }
}
