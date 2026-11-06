//! Cross-node replay packet: a portable, digest-only comparison record.
//!
//! Transport is deliberately out of scope. Two nodes exchange the JSON packet
//! by any channel; `compare` only establishes whether the recorded execution
//! inputs and derived plan/action digests match.

use serde::{Deserialize, Serialize};

pub const REPLAY_SCHEMA: &str = "poie.replay-packet/0.1";
pub const REPLAY_REPORT_SCHEMA: &str = "poie.replay-report/0.1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayPacket {
    pub schema: String,
    pub producer: String,
    pub proposal_id: String,
    pub result_sha256: String,
    pub snapshot_sha256: String,
    pub variant_sha256: String,
    pub inputs_sha256: String,
    pub plan_sha256: String,
    pub actions_sha256: String,
    pub created_at: i64,
    #[serde(default)]
    pub signer_public_key: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplayReport {
    pub schema: String,
    pub ok: bool,
    pub mismatches: Vec<String>,
}

fn is_sha(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}

/// Bytes covered by an optional signature. Signature fields are omitted so a
/// remote node can verify without trusting the packet's own metadata.
pub fn signing_payload(packet: &ReplayPacket) -> Result<Vec<u8>, String> {
    let mut unsigned = packet.clone();
    unsigned.signature = None;
    serde_json::to_vec(&unsigned).map_err(|error| error.to_string())
}

pub fn validate(packet: &ReplayPacket) -> Result<(), String> {
    if packet.schema != REPLAY_SCHEMA {
        return Err(format!("unsupported_schema: {:?}", packet.schema));
    }
    if packet.producer.trim().is_empty() || packet.proposal_id.trim().is_empty() {
        return Err("replay_invalid: producer/proposalId required".into());
    }
    for (name, value) in [
        ("resultSha256", &packet.result_sha256),
        ("snapshotSha256", &packet.snapshot_sha256),
        ("variantSha256", &packet.variant_sha256),
        ("inputsSha256", &packet.inputs_sha256),
        ("planSha256", &packet.plan_sha256),
        ("actionsSha256", &packet.actions_sha256),
    ] {
        if !is_sha(value) {
            return Err(format!("replay_invalid: {name} must be 64 hex characters"));
        }
    }
    Ok(())
}

pub fn compare(local: &ReplayPacket, remote: &ReplayPacket) -> Result<ReplayReport, String> {
    validate(local)?;
    validate(remote)?;
    let mut mismatches = Vec::new();
    for (name, left, right) in [
        ("resultSha256", &local.result_sha256, &remote.result_sha256),
        ("snapshotSha256", &local.snapshot_sha256, &remote.snapshot_sha256),
        ("variantSha256", &local.variant_sha256, &remote.variant_sha256),
        ("inputsSha256", &local.inputs_sha256, &remote.inputs_sha256),
        ("planSha256", &local.plan_sha256, &remote.plan_sha256),
        ("actionsSha256", &local.actions_sha256, &remote.actions_sha256),
    ] {
        if left != right {
            mismatches.push(format!("{name}: local={left} remote={right}"));
        }
    }
    Ok(ReplayReport { schema: REPLAY_REPORT_SCHEMA.into(), ok: mismatches.is_empty(), mismatches })
}

pub fn parse(text: &str) -> Result<ReplayPacket, String> {
    let packet: ReplayPacket = serde_json::from_str(text).map_err(|error| format!("replay_invalid: {error}"))?;
    validate(&packet)?;
    Ok(packet)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet() -> ReplayPacket {
        ReplayPacket {
            schema: REPLAY_SCHEMA.into(),
            producer: "node-a".into(),
            proposal_id: "prom-1".into(),
            result_sha256: "a".repeat(64),
            snapshot_sha256: "b".repeat(64),
            variant_sha256: "c".repeat(64),
            inputs_sha256: "d".repeat(64),
            plan_sha256: "e".repeat(64),
            actions_sha256: "f".repeat(64),
            created_at: 1,
            signer_public_key: None,
            signature: None,
        }
    }

    #[test]
    fn matching_packets_compare_ok() {
        let mut remote = packet();
        remote.producer = "node-b".into();
        let report = compare(&packet(), &remote).unwrap();
        assert!(report.ok);
        assert!(report.mismatches.is_empty());
    }

    #[test]
    fn plan_mismatch_is_reported() {
        let mut remote = packet();
        remote.plan_sha256 = "0".repeat(64);
        let report = compare(&packet(), &remote).unwrap();
        assert!(!report.ok);
        assert_eq!(report.mismatches.len(), 1);
        assert!(report.mismatches[0].starts_with("planSha256"));
    }

    #[test]
    fn invalid_digest_is_rejected() {
        let mut packet = packet();
        packet.actions_sha256 = "nope".into();
        assert!(validate(&packet).is_err());
    }
}
