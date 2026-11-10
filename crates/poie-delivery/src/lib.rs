// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! D1 metadata-only site delivery.
//!
//! JSON manifest/bundle validation, deterministic digest, in-memory state
//! transitions, rollback/clear and redacted diagnostics. No file/network/
//! SQLite/Tauri access; the host owns IO and persistence.

use poie_digest::sha256_of;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const MANIFEST_SCHEMA: &str = "poie.site-manifest/0.1";
pub const BUNDLE_SCHEMA: &str = "poie.site-bundle/0.1";
pub const BUNDLE_SCHEMA_V2: &str = "poie.site-bundle/0.2";
pub const CASE_PACKET_SCHEMA: &str = "poie.case-packet/0.1";
pub const STATE_SCHEMA: &str = "poie.delivery-state/0.1";
pub const DIAGNOSTICS_SCHEMA: &str = "poie.delivery-diagnostics/0.1";
pub const MAX_BUNDLE_BYTES: usize = 256 * 1024;
pub const MAX_MANIFEST_BYTES: usize = 128 * 1024;
pub const MAX_STRING_BYTES: usize = 1024;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestPolicy {
    pub profile: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SiteManifest {
    pub schema: String,
    pub schema_version: String,
    pub site_id: String,
    pub site_name: String,
    pub target: String,
    pub channel: String,
    pub released_at: String,
    #[serde(default)]
    pub model_sha256: Option<String>,
    #[serde(default)]
    pub evidence_sha256: Option<String>,
    pub policy: ManifestPolicy,
    #[serde(default)]
    pub notes: String,
    pub manifest_sha256: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SiteBundle {
    pub schema: String,
    pub manifest: SiteManifest,
    #[serde(default)]
    pub audit_head: Option<String>,
    #[serde(default)]
    pub audit_count: Option<u64>,
    #[serde(default)]
    pub deletions: Vec<DeletionMarker>,
    #[serde(default)]
    pub case_packet: Option<Value>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DeletionMarker {
    pub kind: String,
    pub id: String,
    #[serde(default)]
    pub reason: String,
    pub at: i64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BundleExtensions {
    pub audit_head: Option<String>,
    pub audit_count: Option<u64>,
    pub deletions: Vec<DeletionMarker>,
    pub case_packet: Option<Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ParsedBundle {
    pub manifest: SiteManifest,
    pub manifest_sha256: String,
    pub extensions: BundleExtensions,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ManifestSummary {
    pub site_id: String,
    pub site_name: String,
    pub target: String,
    pub channel: String,
    pub released_at: String,
    pub manifest_sha256: String,
    pub imported_at: i64,
    pub imported_by: String,
    pub source: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeliveryState {
    pub schema: String,
    pub schema_version: String,
    pub current: Option<ManifestSummary>,
    pub previous: Option<ManifestSummary>,
    pub recalled: bool,
    pub updated_at: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticsSummary {
    pub schema: String,
    pub truncated: bool,
    pub generated_at: i64,
    pub app_version: String,
    pub os: String,
    pub arch: String,
    pub current: Option<ManifestSummary>,
    pub previous: Option<ManifestSummary>,
    pub recalled: bool,
    pub checks: Vec<DiagCheck>,
    pub error_codes: Vec<String>,
    pub log_tail: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagCheck {
    pub id: String,
    pub status: String,
    pub detail_code: String,
}

impl DeliveryState {
    pub fn empty(now: i64) -> DeliveryState {
        DeliveryState {
            schema: STATE_SCHEMA.into(),
            schema_version: "0.1".into(),
            current: None,
            previous: None,
            recalled: false,
            updated_at: now,
        }
    }
}

pub fn manifest_digest(manifest: &SiteManifest) -> Result<String, String> {
    let mut value = serde_json::to_value(manifest).map_err(|error| error.to_string())?;
    if let Some(object) = value.as_object_mut() {
        object.remove("manifestSha256");
    }
    sha256_of("poie.manifest/0.1", &value).map_err(|error| error.to_string())
}

fn check_manifest(manifest: &SiteManifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!("unsupported_schema: {:?}", manifest.schema));
    }
    if !matches!(manifest.channel.as_str(), "stable" | "canary" | "recall") {
        return Err("manifest_invalid: channel".into());
    }
    if manifest.site_id.is_empty() || manifest.target.is_empty() || manifest.site_name.is_empty() {
        return Err("manifest_invalid: identity".into());
    }
    for (name, value) in [
        ("siteId", &manifest.site_id),
        ("siteName", &manifest.site_name),
        ("target", &manifest.target),
        ("releasedAt", &manifest.released_at),
        ("notes", &manifest.notes),
    ] {
        if value.len() > MAX_STRING_BYTES {
            return Err(format!("invalid_parameter: {name} too long"));
        }
    }
    if manifest.manifest_sha256.is_empty() {
        return Err("manifest_invalid: manifestSha256 missing".into());
    }
    Ok(())
}

fn check_case_packet(packet: &Value) -> Result<(), String> {
    let object = packet.as_object().ok_or_else(|| "manifest_invalid: casePacket must be an object".to_string())?;
    if object.get("schema").and_then(Value::as_str) != Some(CASE_PACKET_SCHEMA) {
        return Err(format!("unsupported_schema: casePacket must be {CASE_PACKET_SCHEMA}"));
    }
    for key in ["caseId", "scope"] {
        let value =
            object.get(key).and_then(Value::as_str).ok_or_else(|| format!("manifest_invalid: casePacket.{key}"))?;
        if value.trim().is_empty() || value.len() > MAX_STRING_BYTES {
            return Err(format!("manifest_invalid: casePacket.{key} invalid"));
        }
    }
    for key in ["provenance", "evidenceRefs"] {
        let array = object
            .get(key)
            .and_then(Value::as_array)
            .ok_or_else(|| format!("manifest_invalid: casePacket.{key} must be an array"))?;
        if array.len() > 64 {
            return Err(format!("invalid_parameter: casePacket.{key} too long"));
        }
    }
    Ok(())
}

pub fn parse_bundle(text: &str) -> Result<(SiteManifest, String), String> {
    let parsed = parse_bundle_full(text)?;
    Ok((parsed.manifest, parsed.manifest_sha256))
}

pub fn parse_bundle_full(text: &str) -> Result<ParsedBundle, String> {
    if text.len() > MAX_BUNDLE_BYTES {
        return Err("invalid_parameter: bundle too large".into());
    }
    let bundle: SiteBundle = serde_json::from_str(text).map_err(|error| format!("manifest_invalid: {error}"))?;
    if bundle.schema != BUNDLE_SCHEMA && bundle.schema != BUNDLE_SCHEMA_V2 {
        return Err(format!("unsupported_schema: {:?}", bundle.schema));
    }
    check_manifest(&bundle.manifest)?;
    let manifest_size = serde_json::to_string(&bundle.manifest).map_err(|error| error.to_string())?.len();
    if manifest_size > MAX_MANIFEST_BYTES {
        return Err("invalid_parameter: manifest too large".into());
    }
    let digest = manifest_digest(&bundle.manifest)?;
    if digest != bundle.manifest.manifest_sha256 {
        return Err("manifest_invalid: manifestSha256 does not match".into());
    }
    if let Some(head) = &bundle.audit_head {
        if head.len() != 64 || !head.chars().all(|character| character.is_ascii_hexdigit()) {
            return Err("manifest_invalid: auditHead must be 64 hex characters".into());
        }
    }
    if let Some(count) = bundle.audit_count {
        if count == 0 {
            return Err("manifest_invalid: auditCount must be positive".into());
        }
    }
    if bundle.deletions.len() > 256 {
        return Err("invalid_parameter: too many deletion markers".into());
    }
    let mut seen = std::collections::BTreeSet::new();
    for marker in &bundle.deletions {
        if marker.kind.trim().is_empty() || marker.id.trim().is_empty() {
            return Err("manifest_invalid: deletion kind/id required".into());
        }
        if marker.kind.len() > MAX_STRING_BYTES
            || marker.id.len() > MAX_STRING_BYTES
            || marker.reason.len() > MAX_STRING_BYTES
        {
            return Err("invalid_parameter: deletion marker too large".into());
        }
        if !seen.insert((marker.kind.clone(), marker.id.clone())) {
            return Err("manifest_invalid: duplicate deletion marker".into());
        }
    }
    if let Some(packet) = &bundle.case_packet {
        check_case_packet(packet)?;
        let size = serde_json::to_string(packet).map_err(|error| error.to_string())?.len();
        if size > 64 * 1024 {
            return Err("invalid_parameter: case packet too large".into());
        }
    }
    Ok(ParsedBundle {
        manifest: bundle.manifest,
        manifest_sha256: digest,
        extensions: BundleExtensions {
            audit_head: bundle.audit_head,
            audit_count: bundle.audit_count,
            deletions: bundle.deletions,
            case_packet: bundle.case_packet,
        },
    })
}

pub fn summary(manifest: &SiteManifest, sha: &str, imported_at: i64, imported_by: &str) -> ManifestSummary {
    ManifestSummary {
        site_id: manifest.site_id.clone(),
        site_name: manifest.site_name.clone(),
        target: manifest.target.clone(),
        channel: manifest.channel.clone(),
        released_at: manifest.released_at.clone(),
        manifest_sha256: sha.into(),
        imported_at,
        imported_by: imported_by.into(),
        source: "offline".into(),
    }
}

pub fn import(
    state: &DeliveryState,
    manifest: &SiteManifest,
    sha: &str,
    now: i64,
    imported_by: &str,
) -> Result<DeliveryState, String> {
    if state.current.as_ref().is_some_and(|current| current.site_id != manifest.site_id) {
        return Err("delivery_conflict: current state belongs to another site; clear first".into());
    }
    if state.current.as_ref().is_some_and(|current| current.manifest_sha256 == sha) {
        return Ok(state.clone());
    }
    let mut next = state.clone();
    next.previous = next.current.clone();
    next.current = Some(summary(manifest, sha, now, imported_by));
    next.recalled = manifest.channel == "recall";
    next.updated_at = now;
    Ok(next)
}

pub fn rollback(state: &DeliveryState, now: i64) -> Result<DeliveryState, String> {
    let previous = state.previous.clone().ok_or_else(|| "rollback_unavailable: no previous manifest".to_string())?;
    let mut next = state.clone();
    next.current = Some(previous);
    next.previous = state.current.clone();
    next.recalled = next.current.as_ref().is_some_and(|current| current.channel == "recall");
    next.updated_at = now;
    Ok(next)
}

pub fn clear(state: &DeliveryState, now: i64) -> DeliveryState {
    let mut next = state.clone();
    next.current = None;
    next.previous = None;
    next.recalled = false;
    next.updated_at = now;
    next
}

fn redact(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    for token in value.split_whitespace() {
        if token.len() > 8
            && (token.contains("sk-")
                || token.contains("Bearer")
                || token.contains('/')
                || token.chars().filter(|c| *c == '-').count() >= 4)
        {
            output.push_str("[redacted] ");
        } else {
            output.push_str(token);
            output.push(' ');
        }
    }
    output.trim_end().to_string()
}

pub fn diagnostics(
    state: &DeliveryState,
    now: i64,
    app_version: &str,
    os: &str,
    arch: &str,
    log_tail: &[String],
) -> DiagnosticsSummary {
    let mut lines: Vec<String> = log_tail.iter().take(50).map(|line| redact(line)).collect();
    let mut truncated = false;
    let mut bytes = serde_json::to_vec(&lines).map(|value| value.len()).unwrap_or(0);
    while bytes > 128 * 1024 && !lines.is_empty() {
        lines.pop();
        truncated = true;
        bytes = serde_json::to_vec(&lines).map(|value| value.len()).unwrap_or(0);
    }
    DiagnosticsSummary {
        schema: DIAGNOSTICS_SCHEMA.into(),
        truncated,
        generated_at: now,
        app_version: app_version.into(),
        os: os.into(),
        arch: arch.into(),
        current: state.current.clone(),
        previous: state.previous.clone(),
        recalled: state.recalled,
        checks: vec![
            DiagCheck {
                id: "manifest".into(),
                status: "ok".into(),
                detail_code: if state.current.is_some() { "manifest_ok" } else { "state_absent" }.into(),
            },
            DiagCheck {
                id: "recall".into(),
                status: "ok".into(),
                detail_code: if state.recalled { "state_recalled" } else { "manifest_ok" }.into(),
            },
        ],
        error_codes: Vec::new(),
        log_tail: lines,
    }
}

pub fn state_json(state: &DeliveryState) -> Result<String, String> {
    serde_json::to_string(state).map_err(|error| error.to_string())
}
pub fn parse_state(text: &str) -> Result<DeliveryState, String> {
    serde_json::from_str(text).map_err(|error| error.to_string())
}
pub fn manifest_json(manifest: &SiteManifest) -> Result<String, String> {
    serde_json::to_string(manifest).map_err(|error| error.to_string())
}
pub fn parse_manifest(text: &str) -> Result<SiteManifest, String> {
    serde_json::from_str(text).map_err(|error| error.to_string())
}
pub fn summary_json(summary: &ManifestSummary) -> Result<String, String> {
    serde_json::to_string(summary).map_err(|error| error.to_string())
}
pub fn _example() -> Value {
    json!({})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> SiteManifest {
        let mut manifest = SiteManifest {
            schema: MANIFEST_SCHEMA.into(),
            schema_version: "0.1".into(),
            site_id: "base-01".into(),
            site_name: "Base 01".into(),
            target: "linux-x86_64".into(),
            channel: "stable".into(),
            released_at: "2026-11-05T00:00:00Z".into(),
            model_sha256: None,
            evidence_sha256: None,
            policy: ManifestPolicy { profile: "default".into() },
            notes: String::new(),
            manifest_sha256: String::new(),
        };
        manifest.manifest_sha256 = manifest_digest(&manifest).unwrap();
        manifest
    }

    #[test]
    fn digest_and_bundle_roundtrip() {
        let manifest = manifest();
        let bundle = serde_json::to_string(&SiteBundle {
            schema: BUNDLE_SCHEMA.into(),
            manifest: manifest.clone(),
            audit_head: None,
            audit_count: None,
            deletions: Vec::new(),
            case_packet: None,
        })
        .unwrap();
        let (parsed, digest) = parse_bundle(&bundle).unwrap();
        assert_eq!(digest, manifest.manifest_sha256);
        assert_eq!(parsed.site_id, "base-01");
    }

    #[test]
    fn import_rollback_and_clear() {
        let first = manifest();
        let mut second = manifest();
        second.site_name = "Base 02".into();
        second.manifest_sha256 = manifest_digest(&second).unwrap();
        let state = DeliveryState::empty(1);
        let state = import(&state, &first, &first.manifest_sha256, 2, "Ada").unwrap();
        let state = import(&state, &second, &second.manifest_sha256, 3, "Ada").unwrap();
        assert_eq!(state.current.as_ref().unwrap().site_name, "Base 02");
        assert_eq!(state.previous.as_ref().unwrap().site_name, "Base 01");
        let rolled = rollback(&state, 4).unwrap();
        assert_eq!(rolled.current.as_ref().unwrap().site_name, "Base 01");
        assert_eq!(rolled.previous.as_ref().unwrap().site_name, "Base 02");
        let cleared = clear(&rolled, 5);
        assert!(cleared.current.is_none() && cleared.previous.is_none());
    }

    #[test]
    fn same_sha_import_is_idempotent() {
        let manifest = manifest();
        let state = DeliveryState::empty(1);
        let first = import(&state, &manifest, &manifest.manifest_sha256, 2, "Ada").unwrap();
        let again = import(&first, &manifest, &manifest.manifest_sha256, 3, "Ada").unwrap();
        assert_eq!(first, again);
    }

    #[test]
    fn conflict_on_other_site() {
        let manifest = manifest();
        let state = DeliveryState::empty(1);
        let state = import(&state, &manifest, &manifest.manifest_sha256, 2, "Ada").unwrap();
        let mut other = manifest.clone();
        other.site_id = "other".into();
        other.manifest_sha256 = manifest_digest(&other).unwrap();
        assert!(import(&state, &other, &other.manifest_sha256, 3, "Ada").is_err());
    }

    #[test]
    fn unknown_fields_are_rejected() {
        let bundle = format!(
            r#"{{"schema":"{}","manifest":{{"schema":"{}","schemaVersion":"0.1","siteId":"x","siteName":"x","target":"t","channel":"stable","releasedAt":"2026","policy":{{"profile":"p"}},"manifestSha256":"x","extra":1}}}}"#,
            BUNDLE_SCHEMA, MANIFEST_SCHEMA
        );
        assert!(parse_bundle(&bundle).is_err());
    }

    #[test]
    fn diagnostics_redacts_secrets() {
        let state = DeliveryState::empty(1);
        let summary =
            diagnostics(&state, 2, "0.1.1", "linux", "x86_64", &["token sk-secret-123 /Users/ada/secret".into()]);
        assert!(summary.log_tail.join(" ").contains("[redacted]"));
        assert!(!summary.log_tail.join(" ").contains("sk-secret"));
    }

    #[test]
    fn bundle_v2_parses_audit_head_deletions_and_case_packet() {
        let manifest = manifest();
        let bundle = serde_json::to_string(&SiteBundle {
            schema: BUNDLE_SCHEMA_V2.into(),
            manifest: manifest.clone(),
            audit_head: Some("a".repeat(64)),
            audit_count: Some(7),
            deletions: vec![DeletionMarker {
                kind: "memory".into(),
                id: "m1".into(),
                reason: "withdrawn".into(),
                at: 1,
            }],
            case_packet: Some(json!({
                "schema": CASE_PACKET_SCHEMA,
                "caseId": "case-1",
                "scope": "site:base-01",
                "provenance": [],
                "evidenceRefs": []
            })),
        })
        .unwrap();
        let parsed = parse_bundle_full(&bundle).unwrap();
        assert_eq!(parsed.extensions.audit_count, Some(7));
        assert_eq!(parsed.extensions.deletions.len(), 1);
        assert!(parsed.extensions.case_packet.is_some());
    }

    #[test]
    fn bundle_v2_rejects_duplicate_deletion_markers() {
        let manifest = manifest();
        let marker = DeletionMarker { kind: "memory".into(), id: "m1".into(), reason: String::new(), at: 1 };
        let bundle = serde_json::to_string(&SiteBundle {
            schema: BUNDLE_SCHEMA_V2.into(),
            manifest,
            audit_head: None,
            audit_count: None,
            deletions: vec![marker.clone(), marker],
            case_packet: None,
        })
        .unwrap();
        assert!(parse_bundle_full(&bundle).is_err());
    }
}
