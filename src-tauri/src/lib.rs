pub mod commands;
mod errors;
pub mod models;

use std::path::{Path, PathBuf};

use commands::contract;
use commands::design;
use commands::filesystem::PathGuard;
use commands::git;
use commands::sessions::SessionManager;
use commands::watcher::FileWatcherService;
use commands::workspace::WorkspaceService;
use models::design::{DesignMockup, DesignMockupHtml};
use models::file::{FileContent, GitInfo, ParsedDocument};
use models::pulse::PulseData;
use models::session::{OllamaModel, SessionInfo, SessionStartRequest};
use models::workspace::{WorkspaceInfo, WorkspaceSummary};
use tauri::{http, AppHandle, Manager, Runtime, State};

// ─── Application State ───────────────────────────────────────────

pub struct AppState {
    pub path_guard: PathGuard,
    pub workspace_service: WorkspaceService,
    pub watcher: FileWatcherService,
    pub sessions: SessionManager,
}

// ─── Tauri Commands ───────────────────────────────────────────────

#[tauri::command]
fn open_workspace(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<WorkspaceInfo, String> {
    let root = Path::new(&path);
    if !root.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    // Resolve the bundled kit path to do version verification
    let bundled_path = bundled_kit_path(&app).ok();

    // Validate the workspace first — only set root on success
    let info = state
        .workspace_service
        .validate_workspace(&path, bundled_path.as_deref())
        .map_err(|e| e.to_string())?;

    // Set the path guard root after successful validation
    state.path_guard.set_root(root);

    // Register the repository-scoped lmbrain-mcp server so agents working in this
    // workspace receive the controlled-mutation tools. Best-effort: never block
    // opening a workspace if registration cannot be written.
    let mcp_command = commands::mcp_registration::resolve_mcp_command_for_root(root);
    let _ = commands::mcp_registration::register_mcp_server(root, &mcp_command);
    let _ = commands::codex_registration::register_codex_mcp_server(root, &mcp_command);
    let _ = commands::codex_registration::ensure_codex_workspace_trusted(root);
    let _ = commands::codex_registration::scaffold_agents_md(root);

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
    let info = state
        .workspace_service
        .initialize_kit(Path::new(&path), &template)
        .map_err(|e| e.to_string())?;
    let root = Path::new(&path);
    let mcp_command = commands::mcp_registration::resolve_mcp_command_for_root(root);
    let _ = commands::mcp_registration::register_mcp_server(root, &mcp_command);
    let _ = commands::codex_registration::register_codex_mcp_server(root, &mcp_command);
    let _ = commands::codex_registration::ensure_codex_workspace_trusted(root);
    let _ = commands::codex_registration::scaffold_agents_md(root);
    Ok(info)
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
fn get_agent_proposals(
    state: State<'_, AppState>,
) -> Result<Vec<models::agent::AgentProposal>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    contract::build_agent_proposals(&root).map_err(|e| e.to_string())
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
fn get_skills(state: State<'_, AppState>) -> Result<Vec<models::skill::Skill>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    contract::build_skills(&root).map_err(|e| e.to_string())
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
fn get_design_mockups(state: State<'_, AppState>) -> Result<Vec<DesignMockup>, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    design::scan_design_mockups(&root).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_design_mockup_html(
    state: State<'_, AppState>,
    entry_path: String,
) -> Result<DesignMockupHtml, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    design::read_design_html(&root, Path::new(&entry_path)).map_err(|e| e.to_string())
}

#[tauri::command]
fn read_design_mockup_preview_html(
    state: State<'_, AppState>,
    entry_path: String,
) -> Result<DesignMockupHtml, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    design::read_design_preview_html(&root, Path::new(&entry_path)).map_err(|e| e.to_string())
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
fn get_milestone_overview(
    state: State<'_, AppState>,
) -> Result<models::roadmap::MilestoneOverview, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    contract::build_milestone_overview(&root).map_err(|e| e.to_string())
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
fn session_start(
    app: AppHandle,
    state: State<'_, AppState>,
    request: SessionStartRequest,
) -> Result<String, String> {
    let root = state
        .path_guard
        .get_root()
        .ok_or_else(|| "No workspace open".to_string())?;
    if matches!(request.mode, models::session::SessionMode::Claude) {
        let mcp_command = commands::mcp_registration::resolve_mcp_command_for_root(&root);
        let _ = commands::mcp_registration::register_mcp_server(&root, &mcp_command);
    }
    state
        .sessions
        .start(&root, app, request)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn session_write(state: State<'_, AppState>, id: String, data: String) -> Result<(), String> {
    state
        .sessions
        .write(&id, &data)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn session_resize(
    state: State<'_, AppState>,
    id: String,
    cols: u16,
    rows: u16,
) -> Result<(), String> {
    state
        .sessions
        .resize(&id, cols, rows)
        .map_err(|err| err.to_string())
}

#[tauri::command]
fn session_kill(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.sessions.kill(&id).map_err(|err| err.to_string())
}

#[tauri::command]
fn session_attach(state: State<'_, AppState>, id: String) -> Result<String, String> {
    state.sessions.attach(&id).map_err(|err| err.to_string())
}

#[tauri::command]
fn session_list(state: State<'_, AppState>) -> Vec<SessionInfo> {
    state.sessions.list()
}

#[tauri::command]
fn list_ollama_models() -> Result<Vec<OllamaModel>, String> {
    commands::sessions::list_ollama_models().map_err(|err| err.to_string())
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

fn design_preview_protocol<R: Runtime>(
    app: &AppHandle<R>,
    request: http::Request<Vec<u8>>,
) -> http::Response<Vec<u8>> {
    let Some(state) = app.try_state::<AppState>() else {
        return text_response(
            http::StatusCode::SERVICE_UNAVAILABLE,
            "No application state",
        );
    };
    let Some(root) = state.path_guard.get_root() else {
        return text_response(http::StatusCode::SERVICE_UNAVAILABLE, "No workspace open");
    };

    match design::read_design_asset(&root, request.uri().path()) {
        Ok(asset) => http::Response::builder()
            .status(http::StatusCode::OK)
            .header(http::header::CONTENT_TYPE, asset.mime_type)
            .header(http::header::CONTENT_SECURITY_POLICY, design_preview_csp())
            .body(asset.content)
            .unwrap_or_else(|_| {
                text_response(http::StatusCode::INTERNAL_SERVER_ERROR, "Response error")
            }),
        Err(error) => text_response(http::StatusCode::FORBIDDEN, &error.to_string()),
    }
}

fn text_response(status: http::StatusCode, body: &str) -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(status)
        .header(http::header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(body.as_bytes().to_vec())
        .unwrap()
}

fn design_preview_csp() -> &'static str {
    "default-src 'self' data: blob: https:; script-src 'self' 'unsafe-inline' 'unsafe-eval' blob: https:; style-src 'self' 'unsafe-inline' https:; img-src 'self' data: blob: https:; font-src 'self' data: https:; connect-src 'self' https:; worker-src 'self' blob:; frame-src 'self' data: blob: https:"
}

// ─── Application Entry Point ─────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .register_uri_scheme_protocol("lmbrain-design", |ctx, request| {
            design_preview_protocol(ctx.app_handle(), request)
        })
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
                sessions: SessionManager::new(),
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
            get_agent_proposals,
            get_mcp_records,
            get_mcp_proposals,
            get_skills,
            get_handoffs,
            get_design_mockups,
            read_design_mockup_html,
            read_design_mockup_preview_html,
            get_roadmap,
            get_milestone_overview,
            get_wikilink_index,
            get_diagnostics,
            search_content,
            get_wiki_tree,
            get_wiki_page,
            get_git_info,
            start_watcher,
            stop_watcher,
            watcher_status,
            session_start,
            session_write,
            session_resize,
            session_kill,
            session_attach,
            session_list,
            list_ollama_models,
            set_artifact_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
