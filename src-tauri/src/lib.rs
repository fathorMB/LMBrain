pub mod commands;
mod errors;
pub mod models;

use std::path::{Path, PathBuf};

use commands::contract;
use commands::filesystem::PathGuard;
use commands::git;
use commands::watcher::FileWatcherService;
use commands::workspace::WorkspaceService;
use models::file::{FileContent, GitInfo, ParsedDocument};
use models::pulse::PulseData;
use models::workspace::{WorkspaceInfo, WorkspaceSummary};
use tauri::{AppHandle, Manager, State};

// ─── Application State ───────────────────────────────────────────

pub struct AppState {
    pub path_guard: PathGuard,
    pub workspace_service: WorkspaceService,
    pub watcher: FileWatcherService,
}

// ─── Tauri Commands ───────────────────────────────────────────────

#[tauri::command]
fn open_workspace(state: State<'_, AppState>, path: String) -> Result<WorkspaceInfo, String> {
    let root = Path::new(&path);
    if !root.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    // Validate the workspace first — only set root on success
    let info = state
        .workspace_service
        .validate_workspace(&path)
        .map_err(|e| e.to_string())?;

    // Set the path guard root after successful validation
    state.path_guard.set_root(root);

    // Register the repository-scoped lmbrain-mcp server so agents working in this
    // workspace receive the controlled-mutation tools. Best-effort: never block
    // opening a workspace if registration cannot be written.
    let _ = commands::mcp_registration::register_mcp_server(
        root,
        &commands::mcp_registration::resolve_mcp_command(),
    );

    // Add to recent workspaces
    let summary = WorkspaceSummary {
        path: path.clone(),
        name: info.name.clone(),
        health: info.health.clone(),
        last_opened: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        branch: None,
        is_clean: None,
    };
    let _ = state.workspace_service.add_recent(summary);

    Ok(info)
}

#[tauri::command]
fn initialize_workspace_kit(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<WorkspaceInfo, String> {
    let template = bundled_kit_path(&app).map_err(|e| e.to_string())?;
    state
        .workspace_service
        .initialize_kit(Path::new(&path), &template)
        .map_err(|e| e.to_string())
}

fn bundled_kit_path(app: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if cfg!(debug_assertions) {
        return Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../kit/.lmbrain"));
    }
    Ok(app.path().resource_dir()?.join("kit/.lmbrain"))
}

#[tauri::command]
fn list_recent_workspaces(state: State<'_, AppState>) -> Vec<WorkspaceSummary> {
    state.workspace_service.list_recent()
}

#[tauri::command]
fn remove_recent_workspace(state: State<'_, AppState>, path: String) -> Result<(), String> {
    state
        .workspace_service
        .remove_recent(&path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn read_file(state: State<'_, AppState>, path: String) -> Result<FileContent, String> {
    state.path_guard.read_file(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_directory(
    state: State<'_, AppState>,
    path: String,
) -> Result<Vec<models::file::DirEntry>, String> {
    state
        .path_guard
        .list_directory(&path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn parse_markdown(state: State<'_, AppState>, path: String) -> Result<ParsedDocument, String> {
    let content = state
        .path_guard
        .read_file(&path)
        .map_err(|e| e.to_string())?;
    let parsed = commands::parser::parse_markdown_file(&path, &content.content);
    Ok(parsed)
}

#[tauri::command]
fn get_pulse_data(state: State<'_, AppState>) -> Result<PulseData, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;

    let specs = contract::build_specs(&root).map_err(|e| e.to_string())?;
    let reviews = contract::build_reviews(&root).map_err(|e| e.to_string())?;
    let adrs = contract::build_adrs(&root).map_err(|e| e.to_string())?;
    let handoffs = contract::build_handoffs(&root).map_err(|e| e.to_string())?;

    contract::build_pulse_data(&root, &specs, &reviews, &adrs, &handoffs).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_specs(state: State<'_, AppState>) -> Result<Vec<models::spec::Spec>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    contract::build_specs(&root).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_reviews(state: State<'_, AppState>) -> Result<Vec<models::review::Review>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    contract::build_reviews(&root).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_adrs(state: State<'_, AppState>) -> Result<Vec<models::adr::Adr>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    contract::build_adrs(&root).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_agents(state: State<'_, AppState>) -> Result<Vec<models::agent::AgentProfile>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    contract::build_agents(&root).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_mcp_records(state: State<'_, AppState>) -> Result<Vec<models::mcp::McpRecord>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    contract::build_mcp_records(&root).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_mcp_proposals(state: State<'_, AppState>) -> Result<Vec<models::mcp::McpProposal>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    contract::build_mcp_proposals(&root).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_handoffs(state: State<'_, AppState>) -> Result<Vec<models::handoff::Handoff>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    contract::build_handoffs(&root).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_roadmap(state: State<'_, AppState>) -> Result<models::roadmap::Roadmap, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    contract::build_roadmap(&root).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_wikilink_index(
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, Vec<String>>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    Ok(contract::build_wikilink_index(&root))
}

#[tauri::command]
fn get_diagnostics(
    state: State<'_, AppState>,
) -> Result<Vec<models::workspace::KitDiagnostic>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    Ok(contract::build_diagnostics(&root))
}

#[tauri::command]
fn search_content(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<contract::SearchResult>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    Ok(contract::search_content(&root, &query))
}

#[tauri::command]
fn get_wiki_tree(state: State<'_, AppState>) -> Result<models::wiki::WikiTree, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    contract::build_wiki_tree(&root).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_wiki_page(
    state: State<'_, AppState>,
    path: String,
) -> Result<models::wiki::WikiPage, String> {
    let content = state
        .path_guard
        .read_file(&path)
        .map_err(|e| e.to_string())?;
    let parsed = commands::parser::parse_markdown_file(&path, &content.content);

    // Convert frontmatter to string map
    let frontmatter: std::collections::HashMap<String, String> = parsed
        .frontmatter
        .into_iter()
        .map(|(k, v)| (k, serde_json::to_string(&v).unwrap_or_default()))
        .collect();

    Ok(models::wiki::WikiPage {
        path: path.clone(),
        name: Path::new(&path)
            .file_stem()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default(),
        content_html: parsed.body,
        frontmatter,
        wikilinks: parsed.wikilinks,
        backlinks: Vec::new(), // Would need full index
        updated: None,
        word_count: Some(content.content.split_whitespace().count()),
    })
}

#[tauri::command]
fn get_git_info(state: State<'_, AppState>) -> Result<GitInfo, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    Ok(git::get_git_info(&root.to_string_lossy()))
}

#[tauri::command]
fn start_watcher(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    state
        .watcher
        .start(&root.to_string_lossy(), app)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn stop_watcher(state: State<'_, AppState>) -> Result<(), String> {
    state.watcher.stop();
    Ok(())
}

#[tauri::command]
fn watcher_status(state: State<'_, AppState>) -> bool {
    state.watcher.is_active()
}

#[tauri::command]
fn set_artifact_status(
    state: State<'_, AppState>,
    path: String,
    target_status: String,
) -> Result<String, String> {
    contract::set_artifact_status(&state.path_guard, &path, &target_status)
        .map(|new_path| new_path.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

// ─── Application Entry Point ─────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize workspace service with app data dir
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");

            let workspace_service = WorkspaceService::new();
            workspace_service
                .initialize(&app_data_dir)
                .expect("Failed to initialize workspace service");

            // Store application state
            app.manage(AppState {
                path_guard: PathGuard::new(),
                workspace_service,
                watcher: FileWatcherService::new(),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_workspace,
            initialize_workspace_kit,
            list_recent_workspaces,
            remove_recent_workspace,
            read_file,
            list_directory,
            parse_markdown,
            get_pulse_data,
            get_specs,
            get_reviews,
            get_adrs,
            get_agents,
            get_mcp_records,
            get_mcp_proposals,
            get_handoffs,
            get_roadmap,
            get_wikilink_index,
            get_diagnostics,
            search_content,
            get_wiki_tree,
            get_wiki_page,
            get_git_info,
            start_watcher,
            stop_watcher,
            watcher_status,
            set_artifact_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
