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
