//! Phase C1 what-if commands: static snapshots, Patch variants, compare.
//!
//! The source model is read-only here. Results are written only to the
//! `whatif_*` tables; no live/SAP/federation/approval path is touched.

use poie_whatif::{
    compare, run_plan, run_structure, CompareResult, PatchDto, Snapshot, SnapshotInput, Variant, VariantResult,
    VARIANT_SCHEMA,
};
use serde::Deserialize;
use tauri::State;

use crate::auth;
use crate::AppState;

fn random_id(prefix: &str) -> Result<String, String> {
    let mut bytes = [0u8; 12];
    getrandom::getrandom(&mut bytes).map_err(|error| format!("random id: {error}"))?;
    Ok(format!("{prefix}-{}", hex::encode(bytes)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VariantCreateInput {
    pub snapshot_id: String,
    pub label: String,
    #[serde(default)]
    pub description: String,
    pub patch: PatchDto,
    #[serde(default)]
    pub goal: Option<String>,
    #[serde(default)]
    pub min_verification: Option<String>,
    #[serde(default)]
    pub deadline_s: Option<f64>,
}

#[tauri::command]
pub fn whatif_snapshot_create(input: SnapshotInput, state: State<'_, AppState>) -> Result<Snapshot, String> {
    let operator = auth::require(&state.session)?;
    let id = random_id("snap")?;
    let snapshot = Snapshot::create(input, id.clone(), auth::now_ms(), operator.display_name.clone())?;
    snapshot.verify_model(&state.model)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let json = serde_json::to_string(&snapshot).map_err(|error| error.to_string())?;
    db.save_whatif_snapshot(
        &id,
        &snapshot.schema_version,
        &operator.session_id,
        &json,
        &snapshot.snapshot_sha256,
        auth::now_ms(),
    )?;
    db.append_audit(auth::now_ms(), &operator.display_name, "whatif.snapshot", &format!("{id} {}", snapshot.goal))?;
    Ok(snapshot)
}

#[tauri::command]
pub fn whatif_snapshot_list(limit: Option<u32>, state: State<'_, AppState>) -> Result<Vec<Snapshot>, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.whatif_snapshots(limit.unwrap_or(20).clamp(1, 100))?
        .into_iter()
        .map(|json| serde_json::from_str::<Snapshot>(&json).map_err(|error| error.to_string()))
        .collect()
}

#[tauri::command]
pub fn whatif_variant_create(input: VariantCreateInput, state: State<'_, AppState>) -> Result<Variant, String> {
    let operator = auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let snapshot_json =
        db.whatif_snapshot(&input.snapshot_id)?.ok_or_else(|| format!("unknown_snapshot: {}", input.snapshot_id))?;
    let snapshot: Snapshot = serde_json::from_str(&snapshot_json).map_err(|error| error.to_string())?;
    snapshot.verify_model(&state.model)?;
    let variant = Variant {
        schema: poie_whatif::VARIANT_SCHEMA.into(),
        schema_version: "0.1".into(),
        id: random_id("var")?,
        snapshot_id: snapshot.id.clone(),
        label: input.label,
        description: input.description,
        patch: input.patch,
        goal: input.goal,
        min_verification: input.min_verification,
        deadline_s: input.deadline_s,
        created_at: auth::now_ms(),
        created_by: operator.display_name.clone(),
    };
    let json = serde_json::to_string(&variant).map_err(|error| error.to_string())?;
    db.save_whatif_variant(&variant.id, &variant.snapshot_id, VARIANT_SCHEMA, &json, auth::now_ms())?;
    db.append_audit(
        auth::now_ms(),
        &operator.display_name,
        "whatif.variant",
        &format!("{} {}", variant.id, variant.snapshot_id),
    )?;
    Ok(variant)
}

#[tauri::command]
pub fn whatif_variant_list(snapshot_id: String, state: State<'_, AppState>) -> Result<Vec<Variant>, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.whatif_variants(&snapshot_id)?
        .into_iter()
        .map(|json| serde_json::from_str::<Variant>(&json).map_err(|error| error.to_string()))
        .collect()
}

#[tauri::command]
pub fn whatif_variant_run(
    variant_id: String,
    mode: String,
    state: State<'_, AppState>,
) -> Result<VariantResult, String> {
    let operator = auth::require(&state.session)?;
    if mode != "structure" && mode != "plan" {
        return Err("invalid_parameter: mode must be structure or plan".into());
    }
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let variant_json = db.whatif_variant(&variant_id)?.ok_or_else(|| format!("unknown_variant: {variant_id}"))?;
    let variant: Variant = serde_json::from_str(&variant_json).map_err(|error| error.to_string())?;
    let snapshot_json = db
        .whatif_snapshot(&variant.snapshot_id)?
        .ok_or_else(|| format!("unknown_snapshot: {}", variant.snapshot_id))?;
    let snapshot: Snapshot = serde_json::from_str(&snapshot_json).map_err(|error| error.to_string())?;
    let result = if mode == "structure" {
        run_structure(&state.model, &snapshot, &variant)?
    } else {
        run_plan(&state.model, &snapshot, &variant)?
    };
    let settings = serde_json::to_string(&result.settings).map_err(|error| error.to_string())?;
    let json = serde_json::to_string(&result).map_err(|error| error.to_string())?;
    db.save_whatif_result(
        &result.result_id,
        &result.variant_id,
        &result.snapshot_id,
        &result.schema_version,
        &result.mode,
        &settings,
        &json,
        &result.result_sha256,
        auth::now_ms(),
    )?;
    db.append_audit(
        auth::now_ms(),
        &operator.display_name,
        "whatif.run",
        &format!("{} {} {}", result.result_id, result.mode, result.result_sha256),
    )?;
    Ok(result)
}

#[tauri::command]
pub fn whatif_result_get(result_id: String, state: State<'_, AppState>) -> Result<VariantResult, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let json = db.whatif_result(&result_id)?.ok_or_else(|| format!("unknown_result: {result_id}"))?;
    serde_json::from_str(&json).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn whatif_result_list(
    snapshot_id: String,
    variant_id: Option<String>,
    limit: Option<u32>,
    state: State<'_, AppState>,
) -> Result<Vec<VariantResult>, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.whatif_results(&snapshot_id, variant_id.as_deref(), limit.unwrap_or(50).clamp(1, 100))?
        .into_iter()
        .map(|json| serde_json::from_str::<VariantResult>(&json).map_err(|error| error.to_string()))
        .collect()
}

#[tauri::command]
pub fn whatif_compare(
    snapshot_id: String,
    baseline_result_id: Option<String>,
    result_ids: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<CompareResult, String> {
    auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let results: Vec<VariantResult> = db
        .whatif_results(&snapshot_id, None, 200)?
        .into_iter()
        .map(|json| serde_json::from_str::<VariantResult>(&json).map_err(|error| error.to_string()))
        .collect::<Result<_, _>>()?;
    compare(&snapshot_id, &results, baseline_result_id.as_deref(), result_ids.unwrap_or_default().as_slice())
}
