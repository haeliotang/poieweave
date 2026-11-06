//! Poieweave desktop host.
//!
//! The webview only gets a small, explicit command surface: host/domain snapshots,
//! local operator auth/audit, inference configuration, and a size-limited
//! read/write sandbox under the app data directory. Packaging lives in
//! `tauri.conf.json`.

mod agent;
mod anchor;
mod auth;
mod db;
mod delivery;
mod domain;
mod execution;
mod federation_view;
mod http;
mod inference;
mod objects;
mod sandbox;
mod secrets;
mod whatif;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{Manager, State, WebviewUrl, WebviewWindowBuilder};

use crate::auth::{AuthStatus, SessionState};
use crate::db::{AuditRow, Db};
use crate::domain::DomainOverview;
use crate::execution::{ExecutionSnapshot, GateState};
use crate::federation_view::FederationSnapshot;
use crate::http::NativeHttpTransport;
use crate::inference::{InferenceResult, InferenceStatus};
use crate::secrets::SystemSecretStore;

struct AppState {
    db: Arc<Mutex<Db>>,
    data_dir: PathBuf,
    sandbox_root: PathBuf,
    domain: DomainOverview,
    model: Arc<poie_ontology::model::Model>,
    catalog: poie_objects::OntologyCatalog,
    execution: Mutex<ExecutionSnapshot>,
    gate: Mutex<GateState>,
    federation: FederationSnapshot,
    session: Arc<SessionState>,
    secrets: Arc<dyn secrets::SecretStore>,
    http: Arc<dyn http::HttpTransport>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HostInfo {
    version: &'static str,
    data_dir: String,
    db_path: String,
    sandbox_root: String,
    platform: &'static str,
}

fn lock_db(state: &AppState) -> Result<std::sync::MutexGuard<'_, Db>, String> {
    state.db.lock().map_err(|_| "database lock poisoned".to_string())
}

#[tauri::command]
fn auth_status(state: State<'_, AppState>) -> Result<AuthStatus, String> {
    let db = lock_db(&state)?;
    auth::status(&db, &state.session)
}

#[tauri::command]
fn auth_register(display_name: String, passphrase: String, state: State<'_, AppState>) -> Result<AuthStatus, String> {
    let db = lock_db(&state)?;
    auth::register(&db, &state.session, &display_name, &passphrase)
}

#[tauri::command]
fn auth_sign_in(
    passphrase: String,
    display_name: Option<String>,
    state: State<'_, AppState>,
) -> Result<AuthStatus, String> {
    let db = lock_db(&state)?;
    auth::sign_in(&db, &state.session, &passphrase, display_name.as_deref())
}

#[tauri::command]
fn auth_add_operator(
    display_name: String,
    passphrase: String,
    state: State<'_, AppState>,
) -> Result<AuthStatus, String> {
    let db = lock_db(&state)?;
    auth::add_operator(&db, &state.session, &display_name, &passphrase)
}

#[tauri::command]
fn auth_sign_out(state: State<'_, AppState>) -> Result<AuthStatus, String> {
    let db = lock_db(&state)?;
    auth::sign_out(&db, &state.session)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AnchorReceipt {
    count: u64,
    head: String,
    hmac: String,
}

#[tauri::command]
fn audit_verify(state: State<'_, AppState>) -> Result<Option<i64>, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.verify_audit_chain()
}

#[tauri::command]
fn audit_anchor(state: State<'_, AppState>) -> Result<AnchorReceipt, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let (count, head, hmac) = anchor::anchor_current(&db, &*state.secrets)?;
    Ok(AnchorReceipt { count, head, hmac })
}

#[tauri::command]
fn audit_anchor_verify(state: State<'_, AppState>) -> Result<bool, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    anchor::verify_current(&db, &*state.secrets)
}

#[tauri::command]
fn audit_list(limit: Option<usize>, state: State<'_, AppState>) -> Result<Vec<AuditRow>, String> {
    auth::require(&state.session)?;
    let db = lock_db(&state)?;
    db.audit(limit.unwrap_or(20).clamp(1, 100))
}

#[tauri::command]
fn inference_status(state: State<'_, AppState>) -> Result<InferenceStatus, String> {
    let db = lock_db(&state)?;
    inference::status(&db)
}

#[tauri::command]
fn inference_configure(
    endpoint: String,
    model: String,
    api_key: String,
    state: State<'_, AppState>,
) -> Result<InferenceStatus, String> {
    let db = lock_db(&state)?;
    inference::configure(&db, state.secrets.as_ref(), &state.session, &endpoint, &model, &api_key)
}

#[tauri::command]
fn inference_forget(state: State<'_, AppState>) -> Result<InferenceStatus, String> {
    let db = lock_db(&state)?;
    inference::forget(&db, state.secrets.as_ref(), &state.session)
}

async fn run_inference(prompt: String, state: State<'_, AppState>) -> Result<InferenceResult, String> {
    let db = Arc::clone(&state.db);
    let secrets = Arc::clone(&state.secrets);
    let http = Arc::clone(&state.http);
    let session = Arc::clone(&state.session);
    tauri::async_runtime::spawn_blocking(move || {
        let db = db.lock().map_err(|_| "database lock poisoned".to_string())?;
        inference::complete(&db, secrets.as_ref(), &session, http.as_ref(), &prompt)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn inference_complete(prompt: String, state: State<'_, AppState>) -> Result<InferenceResult, String> {
    run_inference(prompt, state).await
}

#[tauri::command]
async fn inference_test(prompt: String, state: State<'_, AppState>) -> Result<InferenceResult, String> {
    run_inference(prompt, state).await
}

#[tauri::command]
fn open_workspace_window(app: tauri::AppHandle, view: String) -> Result<(), String> {
    let allowed = ["overview", "capabilities", "plan", "execution", "federation", "evidence"];
    if !allowed.contains(&view.as_str()) {
        return Err(format!("unknown workspace view {view:?}"));
    }
    let label = format!("poie-view-{view}");
    if let Some(window) = app.get_webview_window(&label) {
        window.set_focus().map_err(|error| error.to_string())?;
        return Ok(());
    }
    let (width, height, title) = match view.as_str() {
        "federation" => (1240.0, 760.0, "Poieweave · federation wall"),
        "execution" => (1100.0, 760.0, "Poieweave · execution monitor"),
        "evidence" => (1100.0, 800.0, "Poieweave · evidence"),
        _ => (1180.0, 820.0, "Poieweave · focused view"),
    };
    WebviewWindowBuilder::new(&app, label, WebviewUrl::App(format!("index.html?view={view}&focus=1").into()))
        .title(title)
        .inner_size(width, height)
        .min_inner_size(900.0, 620.0)
        .build()
        .map_err(|error| format!("open window: {error}"))?;
    Ok(())
}

#[tauri::command]
fn host_info(state: State<'_, AppState>) -> HostInfo {
    HostInfo {
        version: env!("CARGO_PKG_VERSION"),
        data_dir: state.data_dir.display().to_string(),
        db_path: state.data_dir.join("poieweave.sqlite3").display().to_string(),
        sandbox_root: state.sandbox_root.display().to_string(),
        platform: std::env::consts::OS,
    }
}

#[tauri::command]
fn domain_overview(state: State<'_, AppState>) -> DomainOverview {
    state.domain.clone()
}

#[tauri::command]
fn execution_snapshot(state: State<'_, AppState>) -> Result<ExecutionSnapshot, String> {
    Ok(state.execution.lock().map_err(|_| "execution lock poisoned".to_string())?.clone())
}

#[tauri::command]
fn execution_gate_set(takeover: bool, degraded: bool, state: State<'_, AppState>) -> Result<ExecutionSnapshot, String> {
    let operator = auth::require(&state.session)?;
    let gate = GateState { takeover, degraded };
    let snapshot = crate::execution::snapshot(&state.domain, gate)?;
    {
        let mut current = state.gate.lock().map_err(|_| "gate lock poisoned".to_string())?;
        *current = gate;
    }
    {
        let mut current = state.execution.lock().map_err(|_| "execution lock poisoned".to_string())?;
        *current = snapshot.clone();
    }
    let db = lock_db(&state)?;
    db.append_audit(
        auth::now_ms(),
        &operator.display_name,
        "execution.gate",
        &format!("takeover={takeover} degraded={degraded}"),
    )?;
    Ok(snapshot)
}

#[tauri::command]
fn federation_snapshot(state: State<'_, AppState>) -> FederationSnapshot {
    state.federation.clone()
}

#[tauri::command]
fn sandbox_read(relative: String, state: State<'_, AppState>) -> Result<String, String> {
    auth::require(&state.session)?;
    sandbox::read(&state.sandbox_root, &relative)
}

#[tauri::command]
fn sandbox_write(relative: String, contents: String, state: State<'_, AppState>) -> Result<String, String> {
    auth::require(&state.session)?;
    sandbox::write(&state.sandbox_root, &relative, &contents)
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir().map_err(|error| error.to_string())?;
            std::fs::create_dir_all(&data_dir).map_err(|error| format!("data dir: {error}"))?;
            let sandbox_root = data_dir.join("sandbox");
            std::fs::create_dir_all(&sandbox_root).map_err(|error| format!("sandbox: {error}"))?;
            let db = Db::open(&data_dir.join("poieweave.sqlite3"))?;
            let model = Arc::new(domain::source_model().map_err(|error| format!("source model: {error}"))?);
            let catalog = poie_objects::build_catalog(&model).map_err(|error| format!("ontology catalog: {error}"))?;
            let domain = domain::overview().map_err(|error| format!("domain snapshot: {error}"))?;
            let gate = GateState::default();
            let execution =
                execution::snapshot(&domain, gate).map_err(|error| format!("execution snapshot: {error}"))?;
            let federation = federation_view::snapshot().map_err(|error| format!("federation snapshot: {error}"))?;
            app.manage(AppState {
                db: Arc::new(Mutex::new(db)),
                data_dir,
                sandbox_root,
                domain,
                model,
                catalog,
                execution: Mutex::new(execution),
                gate: Mutex::new(gate),
                federation,
                session: Arc::new(SessionState::new(None)),
                secrets: Arc::new(SystemSecretStore),
                http: Arc::new(NativeHttpTransport),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth_status,
            auth_register,
            auth_add_operator,
            auth_sign_in,
            auth_sign_out,
            audit_list,
            audit_verify,
            audit_anchor,
            audit_anchor_verify,
            inference_status,
            inference_configure,
            inference_forget,
            inference_complete,
            inference_test,
            open_workspace_window,
            host_info,
            domain_overview,
            execution_snapshot,
            execution_gate_set,
            federation_snapshot,
            sandbox_read,
            sandbox_write,
            objects::ontology_catalog,
            objects::ontology_objects,
            objects::ontology_object,
            objects::ontology_action_preview,
            agent::agent_run,
            agent::agent_status,
            agent::agent_approve,
            agent::agent_resume,
            agent::agent_reject,
            agent::agent_cancel,
            whatif::whatif_model_digest,
            whatif::whatif_snapshot_create,
            whatif::whatif_snapshot_list,
            whatif::whatif_variant_create,
            whatif::whatif_variant_list,
            whatif::whatif_variant_run,
            whatif::whatif_result_get,
            whatif::whatif_result_list,
            whatif::whatif_compare,
            whatif::whatif_timed_scenario_import,
            whatif::whatif_timed_scenario_list,
            whatif::whatif_timed_snapshot_create,
            whatif::whatif_timed_variant_create,
            whatif::whatif_timed_variant_list,
            whatif::whatif_timed_run,
            whatif::whatif_timed_result_get,
            whatif::whatif_timed_result_list,
            whatif::whatif_timed_compare,
            whatif::whatif_promote_propose,
            whatif::whatif_promote_get,
            whatif::whatif_promote_policy,
            whatif::whatif_promote_set_policy,
            whatif::whatif_promote_replay,
            whatif::whatif_promote_recuse,
            whatif::whatif_promote_appeal,
            whatif::whatif_promote_appeals,
            whatif::whatif_promote_list,
            whatif::whatif_promote_approve,
            whatif::whatif_promote_reject,
            whatif::whatif_promote_cancel,
            whatif::whatif_promote_commit,
            delivery::delivery_manifest_inspect,
            delivery::delivery_import,
            delivery::delivery_state,
            delivery::delivery_bundle_meta,
            delivery::delivery_tombstones,
            delivery::delivery_rollback,
            delivery::delivery_clear,
            delivery::delivery_diagnostics_export
        ])
        .run(tauri::generate_context!())
        .expect("error while running Poieweave desktop");
}
