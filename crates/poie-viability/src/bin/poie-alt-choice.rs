// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! S1: pick the cheaper recovery path when a spare and an alternative restorer
//! are both available.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use poie_ontology::model::{AssetStatus, Flow, Model, Verification};
use poie_viability::closure::{analyze, BlockReason, ClosureOutcome};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn detail(reason: &BlockReason) -> String {
    match reason {
        BlockReason::Cycle { kind, id } => format!("cycle:{kind}:{id}"),
        other => format!("{other:?}"),
    }
}

fn main() -> ExitCode {
    let root = root();
    let base = Model::load(&root.join("scenarios/base-08/model.toml")).unwrap();
    println!("case	spare	alt_part	outcome	detail");
    for (case, spare, part) in
        [("cycle", 0.0, 0.0), ("spare-only", 1.0, 0.0), ("alt-only", 0.0, 1.0), ("both", 1.0, 1.0)]
    {
        let mut model = base.clone();
        model.assets.get_mut("lathe-1").unwrap().status = AssetStatus::Failed;
        model.lots.iter_mut().find(|lot| lot.id == "lot-lathe-nut").unwrap().si = spare;
        let template = model.processes.iter().find(|process| process.id == "replace-lathe-nut").unwrap().clone();
        let mut alternative = template;
        alternative.id = "replace-lathe-nut-external".into();
        alternative.inputs = vec![
            Flow { resource: "external-lathe-nut".into(), si: 1.0 },
            Flow { resource: "electric-energy".into(), si: 0.3 },
        ];
        alternative.outputs = Vec::new();
        alternative.requires_capabilities = vec!["lathe-repair".into()];
        alternative.restores = vec!["lathe-1".into()];
        alternative.duration_s = Some(4.0 * 3600.0);
        alternative.verification = Verification::Demonstrated;
        model.add_process(alternative);
        model.lots.push(poie_ontology::model::Lot {
            id: "lot-external-lathe-nut".into(),
            resource: "external-lathe-nut".into(),
            si: part,
            quality: poie_ontology::model::LotQuality::Accepted,
        });
        match analyze(&model, "lathe-1", 24.0 * 3600.0) {
            ClosureOutcome::Restorable(plan) => {
                println!("{case}	{spare:.0}	{part:.0}	restorable	{:.0}s", plan.total_duration_s);
            }
            ClosureOutcome::Blocked(blocked) => {
                println!("{case}	{spare:.0}	{part:.0}	blocked	{}", detail(&blocked.reasons[0]));
            }
        }
    }
    ExitCode::SUCCESS
}
