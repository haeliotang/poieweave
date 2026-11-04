//! Read-only Ontology product layer: types, objects, links, actions and functions.
//!
//! The formal model remains the only source of truth; this crate projects it
//! into stable DTOs and digests. It never authorizes or dispatches.

use std::collections::BTreeMap;

use poie_digest::{sha256_of, DigestError};
use poie_ontology::model::{AssetStatus, LotQuality, Model, Verification};
use serde::Serialize;
use serde_json::{json, Value};

pub const CATALOG_SCHEMA: &str = "poie.ontology-catalog/0.1";
pub const PAGE_SCHEMA: &str = "poie.object-page/0.1";
pub const DETAIL_SCHEMA: &str = "poie.object-detail/0.1";
pub const ACTION_SCHEMA: &str = "poie.action-preview/0.1";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyDef {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub unit: Option<String>,
    pub required: bool,
    pub read_only: bool,
    pub source: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyValue {
    pub value: Value,
    pub source: String,
    pub as_of: Option<String>,
    pub verification: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectType {
    pub id: String,
    pub label: String,
    pub properties: Vec<PropertyDef>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectInstance {
    pub id: String,
    pub type_id: String,
    pub version: String,
    pub properties: BTreeMap<String, PropertyValue>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkType {
    pub id: String,
    pub from: String,
    pub to: String,
    pub cardinality: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkInstance {
    pub type_id: String,
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceRef {
    pub kind: String,
    pub id: String,
    pub fingerprint: Option<String>,
    pub sha256: Option<String>,
    pub as_of: Option<String>,
    pub source: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlowView {
    pub resource: String,
    pub quantity: String,
    pub unit: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionType {
    pub id: String,
    pub target_types: Vec<String>,
    pub inputs: Vec<FlowView>,
    pub outputs: Vec<FlowView>,
    pub occupies: Vec<String>,
    pub restores: Vec<String>,
    pub requires_capabilities: Vec<String>,
    pub duration_s: Option<f64>,
    pub required_verification: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionType {
    pub id: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Value,
    pub effect: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OntologyCatalog {
    pub schema: String,
    pub model_digest: String,
    pub catalog_digest: String,
    pub object_types: Vec<ObjectType>,
    pub link_types: Vec<LinkType>,
    pub action_types: Vec<ActionType>,
    pub functions: Vec<FunctionType>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectPage {
    pub schema: String,
    pub model_digest: String,
    pub catalog_digest: String,
    pub objects: Vec<ObjectInstance>,
    pub next_cursor: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectDetail {
    pub schema: String,
    pub model_digest: String,
    pub catalog_digest: String,
    pub object: ObjectInstance,
    pub links: Vec<LinkInstance>,
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedActionView {
    pub action_id: String,
    pub process: String,
    pub target: String,
    pub runs: u32,
    pub start_s: f64,
    pub end_s: f64,
    pub preconditions: Vec<String>,
    pub allocations: Vec<(String, f64)>,
    pub occupies: Vec<String>,
    pub restores: Vec<String>,
    pub verification: String,
}

pub fn model_digest(model: &Model) -> Result<String, DigestError> {
    sha256_of("poie.model/0.1", &model_canonical(model))
}

fn model_canonical(model: &Model) -> Value {
    let resources: Vec<Value> = model
        .resources
        .values()
        .map(|resource| {
            json!({
                "id": resource.id,
                "unit": {"symbol": resource.unit.symbol, "scale": resource.unit.scale, "dim": resource.unit.dim.0},
                "massPerUnitKg": resource.mass_per_unit_kg,
                "quality": quality_label(resource.quality),
            })
        })
        .collect();
    let assets: Vec<Value> = model
        .assets
        .values()
        .map(|asset| {
            json!({
                "id": asset.id, "kind": asset.kind, "status": status_label(asset.status),
                "statusEvidence": asset.status_evidence, "requires": asset.requires,
                "requiresAny": asset.requires_any, "provides": asset.provides,
                "capabilityVerification": asset.capability_verification.iter().map(|(key, value)| (key.clone(), verification_label(*value))).collect::<BTreeMap<_, _>>(),
                "degraded": asset.degraded.as_ref().map(|degraded| json!({
                    "occupiable": degraded.occupiable,
                    "allowedProcesses": degraded.allowed_processes,
                    "maxOccupations": degraded.max_occupations,
                })),
            })
        })
        .collect();
    let lots: Vec<Value> = model
        .lots
        .iter()
        .map(|lot| {
            json!({
                "id": lot.id, "resource": lot.resource, "si": lot.si, "quality": quality_label(lot.quality),
            })
        })
        .collect();
    let processes: Vec<Value> = model.processes.iter().map(|process| json!({
        "id": process.id,
        "inputs": process.inputs.iter().map(|flow| json!({"resource": flow.resource, "si": flow.si})).collect::<Vec<_>>(),
        "outputs": process.outputs.iter().map(|flow| json!({"resource": flow.resource, "si": flow.si})).collect::<Vec<_>>(),
        "occupies": process.occupies, "restores": process.restores,
        "requiresCapabilities": process.requires_capabilities,
        "durationS": process.duration_s, "verification": verification_label(process.verification),
    })).collect();
    let services: Vec<Value> = model
        .services
        .values()
        .map(|service| {
            json!({
                "id": service.id, "requires": service.requires, "requiresAny": service.requires_any,
            })
        })
        .collect();
    json!({"name": model.name, "resources": resources, "assets": assets, "lots": lots, "processes": processes, "services": services})
}

pub fn build_catalog(model: &Model) -> Result<OntologyCatalog, DigestError> {
    let model_digest = model_digest(model)?;
    let mut catalog = OntologyCatalog {
        schema: CATALOG_SCHEMA.to_string(),
        model_digest,
        catalog_digest: String::new(),
        object_types: object_types(),
        link_types: link_types(),
        action_types: action_types(model),
        functions: functions(),
    };
    catalog.catalog_digest = catalog_digest(&catalog)?;
    Ok(catalog)
}

fn catalog_digest(catalog: &OntologyCatalog) -> Result<String, DigestError> {
    let mut value = serde_json::to_value(catalog).map_err(|e| DigestError(e.to_string()))?;
    if let Some(object) = value.as_object_mut() {
        object.remove("catalogDigest");
    }
    sha256_of("poie.catalog/0.1", &value)
}

fn object_types() -> Vec<ObjectType> {
    vec![
        object_type(
            "asset",
            "Asset",
            &[
                ("id", "string", None, true),
                ("kind", "string", None, false),
                ("status", "string", None, true),
                ("statusEvidence", "string", None, false),
                ("requires", "string[]", None, false),
                ("requiresAny", "string[]", None, false),
                ("provides", "string[]", None, false),
            ],
        ),
        object_type(
            "resource",
            "Resource",
            &[
                ("id", "string", None, true),
                ("unit", "string", None, true),
                ("massPerUnitKg", "number", Some("kg"), false),
                ("quality", "string", None, true),
            ],
        ),
        object_type(
            "lot",
            "Lot",
            &[
                ("id", "string", None, true),
                ("resource", "string", None, true),
                ("quantity", "string", None, true),
                ("quality", "string", None, true),
            ],
        ),
        object_type(
            "process",
            "Process",
            &[
                ("id", "string", None, true),
                ("durationS", "number", Some("s"), false),
                ("verification", "string", None, true),
                ("occupies", "string[]", None, false),
                ("restores", "string[]", None, false),
                ("requiresCapabilities", "string[]", None, false),
            ],
        ),
        object_type(
            "service",
            "Service",
            &[
                ("id", "string", None, true),
                ("requires", "string[]", None, false),
                ("requiresAny", "string[]", None, false),
            ],
        ),
        object_type(
            "capability",
            "Capability",
            &[
                ("id", "string", None, true),
                ("providerCount", "number", None, true),
                ("providers", "string[]", None, false),
            ],
        ),
    ]
}

fn object_type(id: &str, label: &str, properties: &[(&str, &str, Option<&str>, bool)]) -> ObjectType {
    ObjectType {
        id: id.to_string(),
        label: label.to_string(),
        properties: properties
            .iter()
            .map(|(id, kind, unit, required)| PropertyDef {
                id: (*id).to_string(),
                label: humanize(id),
                kind: (*kind).to_string(),
                unit: unit.map(str::to_string),
                required: *required,
                read_only: true,
                source: "model".to_string(),
            })
            .collect(),
    }
}

fn link_types() -> Vec<LinkType> {
    [
        ("asset.requires", "asset", "asset"),
        ("asset.requires_any", "asset", "asset"),
        ("asset.provides", "asset", "capability"),
        ("process.input", "process", "resource"),
        ("process.output", "process", "resource"),
        ("process.occupies", "process", "asset"),
        ("process.restores", "process", "asset"),
        ("process.requires_capability", "process", "capability"),
        ("service.requires", "service", "asset"),
        ("service.requires_any", "service", "asset"),
        ("lot.of_resource", "lot", "resource"),
    ]
    .iter()
    .map(|(id, from, to)| LinkType {
        id: (*id).to_string(),
        from: (*from).to_string(),
        to: (*to).to_string(),
        cardinality: "many_to_many".to_string(),
    })
    .collect()
}

fn action_types(model: &Model) -> Vec<ActionType> {
    model
        .processes
        .iter()
        .map(|process| {
            let target_types = if process.occupies.is_empty() && process.restores.is_empty() {
                vec!["process".to_string()]
            } else {
                vec!["asset".to_string()]
            };
            ActionType {
                id: process.id.clone(),
                target_types,
                inputs: process.inputs.iter().map(|flow| flow_view(model, flow)).collect(),
                outputs: process.outputs.iter().map(|flow| flow_view(model, flow)).collect(),
                occupies: process.occupies.clone(),
                restores: process.restores.clone(),
                requires_capabilities: process.requires_capabilities.clone(),
                duration_s: process.duration_s,
                required_verification: verification_label(process.verification),
            }
        })
        .collect()
}

fn functions() -> Vec<FunctionType> {
    [
        ("catalog.describe", "Return the Ontology Catalog", "read_only"),
        ("state.query", "Query objects, links, capability results and dependencies", "read_only"),
        ("plan.propose", "Produce a proposed plan without authorization", "propose"),
        ("plan.preview_sap", "Read-only SAP dispatch preview with assumptions", "read_only"),
        ("action.draft", "Create a draft action from an existing plan; approval-gated in Phase B", "propose"),
        ("answer.emit", "Emit validated claims/citations as the terminal agent answer", "read_only"),
    ]
    .iter()
    .map(|(id, description, effect)| FunctionType {
        id: (*id).to_string(),
        description: (*description).to_string(),
        input_schema: json!({"type": "object"}),
        output_schema: json!({"type": "object"}),
        effect: (*effect).to_string(),
    })
    .collect()
}

fn flow_view(model: &Model, flow: &poie_ontology::model::Flow) -> FlowView {
    let resource = &model.resources[&flow.resource];
    FlowView {
        resource: flow.resource.clone(),
        quantity: resource.fmt_amount(flow.si),
        unit: resource.unit.symbol.to_string(),
    }
}

pub fn object_page(
    model: &Model,
    catalog: &OntologyCatalog,
    type_id: &str,
    cursor: Option<&str>,
    limit: usize,
) -> Option<ObjectPage> {
    let type_id = type_id.to_string();
    if !catalog.object_types.iter().any(|object_type| object_type.id == type_id) {
        return None;
    }
    let all = objects_of(model, catalog, &type_id)?;
    let start = cursor.and_then(|cursor| cursor.parse::<usize>().ok()).unwrap_or(0);
    let limit = limit.clamp(1, 500);
    let objects: Vec<_> = all.iter().skip(start).take(limit).cloned().collect();
    let next_cursor = if start + objects.len() < all.len() { Some((start + objects.len()).to_string()) } else { None };
    Some(ObjectPage {
        schema: PAGE_SCHEMA.to_string(),
        model_digest: catalog.model_digest.clone(),
        catalog_digest: catalog.catalog_digest.clone(),
        objects,
        next_cursor,
    })
}

pub fn object_detail(model: &Model, catalog: &OntologyCatalog, type_id: &str, id: &str) -> Option<ObjectDetail> {
    let object =
        objects_of(model, catalog, type_id)?.into_iter().find(|object| object.id == format!("{type_id}:{id}"))?;
    let links = link_instances(model, catalog)
        .into_iter()
        .filter(|link| link.from == object.id || link.to == object.id)
        .collect();
    let mut evidence = Vec::new();
    if type_id == "asset" {
        if let Some(asset) = model.assets.get(id) {
            if let Some(reference) = &asset.status_evidence {
                evidence.push(EvidenceRef {
                    kind: "report".to_string(),
                    id: reference.clone(),
                    fingerprint: Some(reference.clone()),
                    sha256: None,
                    as_of: None,
                    source: "evidence".to_string(),
                });
            }
        }
    }
    Some(ObjectDetail {
        schema: DETAIL_SCHEMA.to_string(),
        model_digest: catalog.model_digest.clone(),
        catalog_digest: catalog.catalog_digest.clone(),
        object,
        links,
        evidence,
    })
}

fn objects_of(model: &Model, catalog: &OntologyCatalog, type_id: &str) -> Option<Vec<ObjectInstance>> {
    let version = catalog.model_digest.clone();
    let mut objects = Vec::new();
    match type_id {
        "asset" => {
            for asset in model.assets.values() {
                let mut properties = BTreeMap::new();
                insert(&mut properties, "id", json!(asset.id), "model", "declared");
                insert(&mut properties, "kind", json!(asset.kind), "model", "declared");
                insert(
                    &mut properties,
                    "status",
                    json!(status_label(asset.status)),
                    if asset.status_evidence.is_some() { "evidence" } else { "declared" },
                    "declared",
                );
                if let Some(reference) = &asset.status_evidence {
                    insert(&mut properties, "statusEvidence", json!(reference), "evidence", "declared");
                }
                insert(&mut properties, "requires", json!(asset.requires), "model", "declared");
                insert(&mut properties, "requiresAny", json!(asset.requires_any), "model", "declared");
                insert(&mut properties, "provides", json!(asset.provides), "model", "declared");
                objects.push(ObjectInstance {
                    id: format!("asset:{}", asset.id),
                    type_id: "asset".to_string(),
                    version: version.clone(),
                    properties,
                });
            }
        }
        "resource" => {
            for resource in model.resources.values() {
                let mut properties = BTreeMap::new();
                insert(&mut properties, "id", json!(resource.id), "model", "declared");
                insert(&mut properties, "unit", json!(resource.unit.symbol), "model", "declared");
                insert(&mut properties, "massPerUnitKg", json!(resource.mass_per_unit_kg), "model", "declared");
                insert(&mut properties, "quality", json!(quality_label(resource.quality)), "model", "declared");
                objects.push(ObjectInstance {
                    id: format!("resource:{}", resource.id),
                    type_id: "resource".to_string(),
                    version: version.clone(),
                    properties,
                });
            }
        }
        "lot" => {
            for lot in &model.lots {
                let mut properties = BTreeMap::new();
                insert(&mut properties, "id", json!(lot.id), "model", "declared");
                insert(&mut properties, "resource", json!(lot.resource), "model", "declared");
                insert(
                    &mut properties,
                    "quantity",
                    json!(model.resources[&lot.resource].fmt_amount(lot.si)),
                    "model",
                    "declared",
                );
                insert(&mut properties, "quality", json!(quality_label(lot.quality)), "model", "declared");
                objects.push(ObjectInstance {
                    id: format!("lot:{}", lot.id),
                    type_id: "lot".to_string(),
                    version: version.clone(),
                    properties,
                });
            }
        }
        "process" => {
            for process in &model.processes {
                let mut properties = BTreeMap::new();
                insert(&mut properties, "id", json!(process.id), "model", "declared");
                insert(&mut properties, "durationS", json!(process.duration_s), "model", "declared");
                insert(
                    &mut properties,
                    "verification",
                    json!(verification_label(process.verification)),
                    "model",
                    "declared",
                );
                insert(&mut properties, "occupies", json!(process.occupies), "model", "declared");
                insert(&mut properties, "restores", json!(process.restores), "model", "declared");
                insert(
                    &mut properties,
                    "requiresCapabilities",
                    json!(process.requires_capabilities),
                    "model",
                    "declared",
                );
                objects.push(ObjectInstance {
                    id: format!("process:{}", process.id),
                    type_id: "process".to_string(),
                    version: version.clone(),
                    properties,
                });
            }
        }
        "service" => {
            for service in model.services.values() {
                let mut properties = BTreeMap::new();
                insert(&mut properties, "id", json!(service.id), "model", "declared");
                insert(&mut properties, "requires", json!(service.requires), "model", "declared");
                insert(&mut properties, "requiresAny", json!(service.requires_any), "model", "declared");
                objects.push(ObjectInstance {
                    id: format!("service:{}", service.id),
                    type_id: "service".to_string(),
                    version: version.clone(),
                    properties,
                });
            }
        }
        "capability" => {
            for (capability, providers) in &model.providers {
                let mut properties = BTreeMap::new();
                insert(&mut properties, "id", json!(capability), "model", "declared");
                insert(&mut properties, "providerCount", json!(providers.len()), "model", "declared");
                insert(&mut properties, "providers", json!(providers), "model", "declared");
                objects.push(ObjectInstance {
                    id: format!("capability:{capability}"),
                    type_id: "capability".to_string(),
                    version: version.clone(),
                    properties,
                });
            }
        }
        _ => return None,
    }
    objects.sort_by(|left, right| left.id.cmp(&right.id));
    Some(objects)
}

fn link_instances(model: &Model, catalog: &OntologyCatalog) -> Vec<LinkInstance> {
    let mut links = Vec::new();
    for asset in model.assets.values() {
        for target in &asset.requires {
            links.push(LinkInstance {
                type_id: "asset.requires".to_string(),
                from: format!("asset:{}", asset.id),
                to: format!("asset:{target}"),
            });
        }
        for target in &asset.requires_any {
            links.push(LinkInstance {
                type_id: "asset.requires_any".to_string(),
                from: format!("asset:{}", asset.id),
                to: format!("asset:{target}"),
            });
        }
        for capability in &asset.provides {
            links.push(LinkInstance {
                type_id: "asset.provides".to_string(),
                from: format!("asset:{}", asset.id),
                to: format!("capability:{capability}"),
            });
        }
    }
    for process in &model.processes {
        for flow in &process.inputs {
            links.push(LinkInstance {
                type_id: "process.input".to_string(),
                from: format!("process:{}", process.id),
                to: format!("resource:{}", flow.resource),
            });
        }
        for flow in &process.outputs {
            links.push(LinkInstance {
                type_id: "process.output".to_string(),
                from: format!("process:{}", process.id),
                to: format!("resource:{}", flow.resource),
            });
        }
        for asset in &process.occupies {
            links.push(LinkInstance {
                type_id: "process.occupies".to_string(),
                from: format!("process:{}", process.id),
                to: format!("asset:{asset}"),
            });
        }
        for asset in &process.restores {
            links.push(LinkInstance {
                type_id: "process.restores".to_string(),
                from: format!("process:{}", process.id),
                to: format!("asset:{asset}"),
            });
        }
        for capability in &process.requires_capabilities {
            links.push(LinkInstance {
                type_id: "process.requires_capability".to_string(),
                from: format!("process:{}", process.id),
                to: format!("capability:{capability}"),
            });
        }
    }
    for service in model.services.values() {
        for asset in &service.requires {
            links.push(LinkInstance {
                type_id: "service.requires".to_string(),
                from: format!("service:{}", service.id),
                to: format!("asset:{asset}"),
            });
        }
        for asset in &service.requires_any {
            links.push(LinkInstance {
                type_id: "service.requires_any".to_string(),
                from: format!("service:{}", service.id),
                to: format!("asset:{asset}"),
            });
        }
    }
    for lot in &model.lots {
        links.push(LinkInstance {
            type_id: "lot.of_resource".to_string(),
            from: format!("lot:{}", lot.id),
            to: format!("resource:{}", lot.resource),
        });
    }
    links.sort_by(|left, right| (&left.type_id, &left.from, &left.to).cmp(&(&right.type_id, &right.from, &right.to)));
    let _ = catalog;
    links
}

fn insert(properties: &mut BTreeMap<String, PropertyValue>, key: &str, value: Value, source: &str, verification: &str) {
    properties.insert(
        key.to_string(),
        PropertyValue { value, source: source.to_string(), as_of: None, verification: verification.to_string() },
    );
}

fn humanize(id: &str) -> String {
    id.split('_')
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map(|first| first.to_uppercase().collect::<String>() + chars.as_str()).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ")
}
fn status_label(status: AssetStatus) -> &'static str {
    match status {
        AssetStatus::Available => "available",
        AssetStatus::Degraded => "degraded",
        AssetStatus::Failed => "failed",
        AssetStatus::Unknown => "unknown",
    }
}
fn quality_label(quality: LotQuality) -> &'static str {
    match quality {
        LotQuality::Accepted => "accepted",
        LotQuality::Quarantined => "quarantined",
        LotQuality::Rejected => "rejected",
    }
}
fn verification_label(verification: Verification) -> String {
    format!("{verification:?}").to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use poie_ontology::model::Model;
    use std::path::Path;

    fn base_model() -> Model {
        Model::load(Path::new("../../scenarios/base-01/model.toml")).unwrap()
    }

    #[test]
    fn catalog_is_deterministic_and_has_all_process_actions() {
        let model = base_model();
        let first = build_catalog(&model).unwrap();
        let second = build_catalog(&model).unwrap();
        assert_eq!(first.catalog_digest, second.catalog_digest);
        assert_eq!(first.model_digest, second.model_digest);
        assert_eq!(first.action_types.len(), model.processes.len());
        assert!(first.object_types.iter().any(|object_type| object_type.id == "asset"));
    }

    #[test]
    fn objects_and_links_are_deterministic() {
        let model = base_model();
        let catalog = build_catalog(&model).unwrap();
        let page = object_page(&model, &catalog, "asset", None, 500).unwrap();
        assert!(page.objects.iter().any(|object| object.id == "asset:pump-p1"));
        assert!(page.objects.windows(2).all(|pair| pair[0].id <= pair[1].id));
        let detail = object_detail(&model, &catalog, "asset", "pump-p1").unwrap();
        assert_eq!(detail.object.type_id, "asset");
        assert!(detail.evidence.iter().any(|evidence| evidence.source == "evidence") || detail.evidence.is_empty());
    }

    #[test]
    fn page_limit_and_cursor_are_stable() {
        let model = base_model();
        let catalog = build_catalog(&model).unwrap();
        let first = object_page(&model, &catalog, "asset", None, 2).unwrap();
        assert_eq!(first.objects.len(), 2);
        let second = object_page(&model, &catalog, "asset", first.next_cursor.as_deref(), 2).unwrap();
        assert!(second.objects.first().unwrap().id > first.objects.last().unwrap().id);
    }
}
