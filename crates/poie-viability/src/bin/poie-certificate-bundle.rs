// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bundle the S1/S2/S3 certificates into one content-addressed index.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_digest::hex_sha256;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let certificates = [
        ("S1", "experiments/certificate-s1-closure/certificate.json", "poie.closure-certificate/1"),
        ("S2", "experiments/certificate-s2-load/certificate.json", "poie.viability-certificate/1"),
        ("S2-joint", "experiments/certificate-s2-joint/certificate.json", "poie.coverage-certificate/1"),
        ("S2-regen", "experiments/certificate-s2-regeneration/certificate.json", "poie.regeneration-certificate/1"),
        ("S2-metrics", "experiments/certificate-s2-metrics/certificate.json", "poie.metrics-certificate/1"),
        ("S2-order", "experiments/certificate-s2-order/certificate.json", "poie.order-certificate/1"),
        ("S3", "experiments/certificate-s3-recovery/certificate.json", "poie.recovery-certificate/1"),
        ("S3-multi", "experiments/certificate-s3-multi/certificate.json", "poie.multi-recovery-certificate/1"),
        ("S4", "experiments/certificate-s4-differential/certificate.json", "poie.s4-certificate/1"),
        ("S4-approval", "experiments/certificate-s4-approval/certificate.json", "poie.approval-certificate/1"),
    ];
    let mut rows = Vec::new();
    for (layer, relative, expected_schema) in certificates {
        let path = root.join(relative);
        let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {relative}: {error}"));
        let value: serde_json::Value = serde_json::from_slice(&bytes).expect("certificate JSON");
        assert_eq!(value.get("schema").and_then(|v| v.as_str()), Some(expected_schema));
        rows.push(serde_json::json!({
            "layer": layer,
            "path": relative,
            "sha256": hex_sha256(&bytes),
            "schema": expected_schema,
        }));
    }
    let bundle = serde_json::json!({
        "schema": "poie.certificate-bundle/1",
        "certificates": rows,
        "claim_scope": "model/simulation certificates only; no physical safety or authority claim",
    });
    println!("{}", serde_json::to_string_pretty(&bundle).expect("bundle JSON"));
    ExitCode::SUCCESS
}
