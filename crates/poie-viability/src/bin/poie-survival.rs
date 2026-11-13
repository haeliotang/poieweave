// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S2 first slice: deterministic failure-envelope coverage over Base-05.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::Model;
use poie_viability::survival::failure_envelope;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn main() -> ExitCode {
    let root = root();
    let model = Model::load(&root.join("scenarios/base-05/model.toml")).unwrap_or_else(|error| {
        panic!("load Base-05: {error}");
    });
    let envelope = failure_envelope(&model, "potable-water", 12.0 * 3600.0, 1).unwrap();
    println!(
        "service={} horizon_s={:.0} total={} recoverable={} coverage={:.6}",
        envelope.service,
        envelope.horizon_s,
        envelope.total(),
        envelope.recoverable,
        envelope.coverage()
    );
    for case in &envelope.cases {
        let failed = if case.failed.is_empty() { "none".to_string() } else { case.failed.join(",") };
        let detail = match &case.blocked_by {
            Some(reason) => format!("BLOCKED\treason={reason}"),
            None => format!("RESTORABLE\tmax_plan_s={:.0}", case.max_plan_s),
        };
        println!("case\tfailed={failed}\t{detail}");
    }
    ExitCode::SUCCESS
}
