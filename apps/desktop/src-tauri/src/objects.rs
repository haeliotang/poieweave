// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Read-only Ontology product commands for the desktop host.

use poie_objects::{ObjectDetail, ObjectPage, OntologyCatalog};
use tauri::State;

use crate::AppState;

#[tauri::command]
pub fn ontology_catalog(state: State<'_, AppState>) -> Result<OntologyCatalog, String> {
    Ok(state.catalog.clone())
}

#[tauri::command]
pub fn ontology_objects(
    type_id: String,
    cursor: Option<String>,
    limit: Option<usize>,
    state: State<'_, AppState>,
) -> Result<ObjectPage, String> {
    poie_objects::object_page(&state.model, &state.catalog, &type_id, cursor.as_deref(), limit.unwrap_or(100))
        .ok_or_else(|| format!("unknown object type {type_id:?}"))
}

#[tauri::command]
pub fn ontology_object(type_id: String, id: String, state: State<'_, AppState>) -> Result<ObjectDetail, String> {
    poie_objects::object_detail(&state.model, &state.catalog, &type_id, &id)
        .ok_or_else(|| format!("unknown object {type_id}:{id}"))
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionPreview {
    pub schema: String,
    pub model_digest: String,
    pub catalog_digest: String,
    pub plan_fingerprint: String,
    pub action: poie_objects::PlannedActionView,
    pub assumptions: Vec<String>,
    pub reasons: Vec<String>,
}

#[tauri::command]
pub fn ontology_action_preview(
    plan_id: String,
    action_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<ActionPreview, String> {
    if plan_id != state.domain.plan.plan_id {
        return Err(format!("unknown plan {plan_id:?}"));
    }
    let source = state
        .domain
        .plan
        .actions
        .iter()
        .find(|action| action.id == action_id)
        .ok_or_else(|| format!("unknown action {action_id:?}"))?;
    Ok(ActionPreview {
        schema: "poie.action-preview/0.1".to_string(),
        model_digest: state.catalog.model_digest.clone(),
        catalog_digest: state.catalog.catalog_digest.clone(),
        plan_fingerprint: state.domain.plan.digest.clone(),
        action: poie_objects::PlannedActionView {
            action_id: source.id.clone(),
            process: source.process.clone(),
            target: source.target.clone(),
            runs: source.runs,
            start_s: source.start_s,
            end_s: source.end_s,
            preconditions: source.preconditions.clone(),
            allocations: source.allocations.clone(),
            occupies: source.occupies.clone(),
            restores: Vec::new(),
            verification: source.verification.clone(),
        },
        assumptions: vec!["SAP dispatch preview is not evaluated in A1; no authorization is created".to_string()],
        reasons: vec![],
    })
}
