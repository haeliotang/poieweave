// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::PathBuf;

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/delivery").join(name);
    std::fs::read_to_string(path).expect("fixture")
}

#[test]
fn golden_v2_bundle_parses_case_packet_and_deletions() {
    let parsed = poie_delivery::parse_bundle_full(&fixture("site-bundle-v2.json")).expect("valid bundle");
    assert_eq!(parsed.extensions.audit_count, Some(2));
    assert_eq!(parsed.extensions.deletions.len(), 1);
    let packet = parsed.extensions.case_packet.expect("case packet");
    assert_eq!(packet["schema"], "poie.case-packet/0.1");
}

#[test]
fn duplicate_deletion_fixture_is_rejected() {
    let error = poie_delivery::parse_bundle_full(&fixture("site-bundle-invalid-duplicate-deletion.json"))
        .expect_err("duplicate deletion");
    assert!(error.contains("duplicate deletion"));
}
