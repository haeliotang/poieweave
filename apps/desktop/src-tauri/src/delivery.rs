//! D1 offline metadata delivery: sandbox JSON only, SQLite state, no network.
use poie_delivery::{
    clear, diagnostics, import, parse_bundle, parse_state, rollback, state_json, DeliveryState, DiagnosticsSummary,
    SiteManifest,
};
use serde::Serialize;
use tauri::State;

use crate::auth;
use crate::sandbox;
use crate::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InspectResult {
    pub manifest: SiteManifest,
    pub manifest_sha256: String,
}

fn now_ms() -> i64 {
    auth::now_ms()
}

fn load_state(db: &crate::db::Db) -> Result<DeliveryState, String> {
    match db.delivery_state()? {
        Some(json) => parse_state(&json),
        None => Ok(DeliveryState::empty(now_ms())),
    }
}

fn save_state(
    state: &State<'_, AppState>,
    db: &crate::db::Db,
    next: &DeliveryState,
    event: &str,
    detail: &str,
) -> Result<(), String> {
    let operator = auth::require(&state.session)?;
    let json = state_json(next)?;
    db.save_delivery_state(&json, next.updated_at)?;
    db.append_audit(now_ms(), &operator.display_name, event, detail)?;
    Ok(())
}

#[tauri::command]
pub fn delivery_manifest_inspect(path: String, state: State<'_, AppState>) -> Result<InspectResult, String> {
    auth::require(&state.session)?;
    let text = sandbox::read(&state.sandbox_root, &path).map_err(|error| format!("io_error: {error}"))?;
    let (manifest, manifest_sha256) = parse_bundle(&text)?;
    Ok(InspectResult { manifest, manifest_sha256 })
}

#[tauri::command]
pub fn delivery_import(
    path: String,
    expected_manifest_sha256: String,
    state: State<'_, AppState>,
) -> Result<DeliveryState, String> {
    let operator = auth::require(&state.session)?;
    let text = sandbox::read(&state.sandbox_root, &path).map_err(|error| format!("io_error: {error}"))?;
    let (manifest, sha) = parse_bundle(&text)?;
    if expected_manifest_sha256.trim().is_empty() || expected_manifest_sha256 != sha {
        return Err("stale_manifest: expected manifest sha does not match".into());
    }
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let current = load_state(&db)?;
    let next = import(&current, &manifest, &sha, now_ms(), &operator.display_name)?;
    let manifest_json = poie_delivery::manifest_json(&manifest)?;
    db.save_delivery_import(&manifest_json, &sha, &state_json(&next)?, now_ms())?;
    db.append_audit(
        now_ms(),
        &operator.display_name,
        "delivery.import",
        &format!("{} {} {}", next.current.as_ref().map(|c| c.site_id.as_str()).unwrap_or(""), manifest.target, sha),
    )?;
    Ok(next)
}

#[tauri::command]
pub fn delivery_state(state: State<'_, AppState>) -> Result<DeliveryState, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    load_state(&db)
}

#[tauri::command]
pub fn delivery_rollback(state: State<'_, AppState>) -> Result<DeliveryState, String> {
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let current = load_state(&db)?;
    let next = rollback(&current, now_ms())?;
    save_state(
        &state,
        &db,
        &next,
        "delivery.rollback",
        next.current.as_ref().map(|c| c.manifest_sha256.as_str()).unwrap_or(""),
    )?;
    Ok(next)
}

#[tauri::command]
pub fn delivery_clear(state: State<'_, AppState>) -> Result<DeliveryState, String> {
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let current = load_state(&db)?;
    let next = clear(&current, now_ms());
    save_state(&state, &db, &next, "delivery.clear", "current/previous cleared")?;
    Ok(next)
}

#[tauri::command]
pub fn delivery_diagnostics_export(
    output_dir: String,
    overwrite: Option<bool>,
    state: State<'_, AppState>,
) -> Result<DiagnosticsSummary, String> {
    let operator = auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let current = load_state(&db)?;
    let summary =
        diagnostics(&current, now_ms(), env!("CARGO_PKG_VERSION"), std::env::consts::OS, std::env::consts::ARCH, &[]);
    let path = format!("{}/delivery-diagnostics.json", output_dir.trim_end_matches('/'));
    if !overwrite.unwrap_or(false) && sandbox::read(&state.sandbox_root, &path).is_ok() {
        return Err("delivery_conflict: diagnostics file exists".into());
    }
    let text = serde_json::to_string_pretty(&summary).map_err(|error| error.to_string())?;
    sandbox::write(&state.sandbox_root, &path, &text).map_err(|error| format!("io_error: {error}"))?;
    db.append_audit(now_ms(), &operator.display_name, "delivery.diagnostics", &path)?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn real_file_import_rollback_and_diagnostics() {
        let root = std::env::temp_dir().join(format!("poie-delivery-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let mut manifest = poie_delivery::SiteManifest {
            schema: poie_delivery::MANIFEST_SCHEMA.into(),
            schema_version: "0.1".into(),
            site_id: "base-01".into(),
            site_name: "Base 01".into(),
            target: "linux-x86_64".into(),
            channel: "stable".into(),
            released_at: "2026-11-05T00:00:00Z".into(),
            model_sha256: None,
            evidence_sha256: None,
            policy: poie_delivery::ManifestPolicy { profile: "default".into() },
            notes: String::new(),
            manifest_sha256: String::new(),
        };
        manifest.manifest_sha256 = poie_delivery::manifest_digest(&manifest).unwrap();
        let bundle = serde_json::to_string(&poie_delivery::SiteBundle {
            schema: poie_delivery::BUNDLE_SCHEMA.into(),
            manifest: manifest.clone(),
        })
        .unwrap();
        sandbox::write(&root, "delivery/site.json", &bundle).unwrap();
        let text = sandbox::read(&root, "delivery/site.json").unwrap();
        let (parsed, sha) = parse_bundle(&text).unwrap();
        let db = crate::db::Db::open_in_memory().unwrap();
        let state = crate::delivery::import(&DeliveryState::empty(1), &parsed, &sha, 2, "acceptance").unwrap();
        db.save_delivery_import(&poie_delivery::manifest_json(&parsed).unwrap(), &sha, &state_json(&state).unwrap(), 2)
            .unwrap();
        assert_eq!(load_state(&db).unwrap().current.unwrap().site_id, "base-01");
        let rolled = rollback(&load_state(&db).unwrap(), 3);
        assert!(rolled.is_err());
        let diag = diagnostics(&state, 4, "0.1.1", "macos", "aarch64", &[]);
        assert_eq!(diag.schema, poie_delivery::DIAGNOSTICS_SCHEMA);
        assert!(!diag.truncated);
        let _ = std::fs::remove_dir_all(&root);
    }
}
