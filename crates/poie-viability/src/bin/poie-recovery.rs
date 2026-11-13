// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S3 first slice: approval-gated recovery loop on Base-05.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::{simulate_recovery, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-05/model.toml")).unwrap_or_else(|error| {
        panic!("load Base-05: {error}");
    });
    let events = vec![
        FailureEvent { at_s: 10.0 * 86400.0, asset: "pump-p1".into() },
        FailureEvent { at_s: 20.0 * 86400.0, asset: "pump-p1".into() },
    ];
    for approved in [false, true] {
        let report = simulate_recovery(&model, "potable-water", 30.0 * 86400.0, &events, approved).unwrap();
        let blocked = report.log.iter().find_map(|entry| entry.blocked_by.as_deref()).unwrap_or("none");
        println!(
            "approved={approved}\tsurvived_days={:.0}\tevents={}\tblocked_by={blocked}",
            report.survived_s / 86400.0,
            report.log.len()
        );
    }
    ExitCode::SUCCESS
}
