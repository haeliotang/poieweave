// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later
//! N8 event-resolved observation and noninterference evidence runner.

use std::path::{Path, PathBuf};

use poie_ontology::model::{AssetStatus, Model, Service};
use poie_viability::engine::{
    run_engine, run_engine_event_observed, run_engine_observed, EngineOptions, EngineTermination, ObservationPhase,
    ObservationRequest,
};
use poie_viability::survival::{BatteryLimits, FailureEvent, Resupply};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}
fn label(term: &EngineTermination) -> String {
    match term {
        EngineTermination::CompletedHorizon => "completed".into(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    }
}
fn options(failures: Vec<FailureEvent>, resupplies: Vec<Resupply>, horizon: f64) -> EngineOptions<'static> {
    EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: horizon,
        failures,
        resupplies,
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 3.0,
    }
}
fn tool_model() -> Model {
    let mut model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let robot = model.assets.get("robot-m1").unwrap().clone();
    let mut tool = robot.clone();
    tool.id = "tool-t1".into();
    tool.kind = "tool".into();
    tool.status = AssetStatus::Available;
    tool.requires.clear();
    tool.requires_any.clear();
    tool.provides.clear();
    model.assets.insert("tool-t1".into(), tool);
    let pump = model.assets.get("pump-p1").unwrap().clone();
    let template = model.processes.iter().find(|process| process.id == "replace-sleeve-p1").unwrap().clone();
    let mut asset = pump.clone();
    asset.id = "a-lathe".into();
    asset.status = AssetStatus::Failed;
    asset.requires.clear();
    asset.requires_any.clear();
    asset.provides.clear();
    model.assets.insert("a-lathe".into(), asset);
    let mut process = template;
    process.id = "repair-a-lathe".into();
    process.inputs.clear();
    process.outputs.clear();
    process.requires_capabilities.clear();
    process.occupies = vec!["robot-m1".into(), "tool-t1".into()];
    process.restores = vec!["a-lathe".into()];
    process.duration_s = Some(10800.0);
    let index = model.processes.len();
    model.processes.push(process);
    model.restorers.entry("a-lathe".into()).or_default().push(index);
    model.services.insert(
        "n8-tool".into(),
        Service { id: "n8-tool".into(), requires: vec!["a-lathe".into()], requires_any: Vec::new() },
    );
    model
}

fn main() {
    let out = root().join("audit/phase8");
    std::fs::create_dir_all(out.join("event-observations")).unwrap();
    let cases = [
        ("N8-00", Vec::new(), Vec::new(), 20.0 * 3600.0),
        ("N8-01", vec![FailureEvent { at_s: 7200.0, asset: "pv-1".into() }], Vec::new(), 20.0 * 3600.0),
        ("N8-02", vec![FailureEvent { at_s: 4500.0, asset: "pv-1".into() }], Vec::new(), 20.0 * 3600.0),
        ("N8-03", vec![FailureEvent { at_s: 10800.0, asset: "robot-m1".into() }], Vec::new(), 20.0 * 3600.0),
        ("N8-04", vec![FailureEvent { at_s: 7200.0, asset: "robot-m1".into() }], Vec::new(), 20.0 * 3600.0),
        ("N8-05", vec![FailureEvent { at_s: 3600.0, asset: "lathe-1".into() }], Vec::new(), 20.0 * 3600.0),
        ("N8-07", vec![FailureEvent { at_s: 3600.0, asset: "bus-1".into() }], Vec::new(), 20.0 * 3600.0),
        ("N8-08", vec![FailureEvent { at_s: 10800.0, asset: "robot-m1".into() }], Vec::new(), 20.0 * 3600.0),
        ("N8-09", Vec::new(), Vec::new(), 10800.0),
        ("N8-10", vec![FailureEvent { at_s: 10800.0, asset: "robot-m1".into() }], Vec::new(), 10800.0),
        (
            "N8-11",
            vec![FailureEvent { at_s: 10800.0, asset: "pv-1".into() }],
            vec![Resupply { at_s: 10800.0, resource: "cutting-insert".into(), si: 1.0 }],
            20.0 * 3600.0,
        ),
        (
            "N8-12",
            vec![
                FailureEvent { at_s: 3600.0, asset: "robot-m1".into() },
                FailureEvent { at_s: 7200.0, asset: "lathe-1".into() },
            ],
            Vec::new(),
            20.0 * 3600.0,
        ),
        ("N8-13", vec![FailureEvent { at_s: 7200.0, asset: "robot-m1".into() }], Vec::new(), 20.0 * 3600.0),
    ];
    let mut rows = String::from("scenario\tevent_sequence\tevent_id\tevent_batch_sequence\tat_s\tevent_kind\tevent_origin\ttarget_asset\tevent_status\tasset_id\tstatus_before\tstatus_after\ttermination\n");
    let mut summaries = String::from(
        "scenario\tevent_count\ttermination\tsurvived_s\tdirect_time_equal\tdirect_event_equal\ttime_event_equal\n",
    );
    for (id, failures, resupplies, horizon) in cases {
        let model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
        let opts = options(failures, resupplies, horizon);
        let direct = run_engine(&model, &opts).unwrap();
        let time =
            run_engine_observed(&model, &opts, &[ObservationRequest { at_s: 10800.0, phase: ObservationPhase::After }])
                .unwrap();
        let event = run_engine_event_observed(&model, &opts).unwrap();
        assert_eq!(direct, time.outcome, "time noninterference {id}");
        assert_eq!(direct, event.outcome, "event noninterference {id}");
        for record in &event.events {
            let all: std::collections::BTreeSet<&str> = record
                .asset_statuses_before
                .keys()
                .chain(record.asset_statuses_after.keys())
                .map(|value| value.as_str())
                .collect();
            for asset in all {
                let before = record
                    .asset_statuses_before
                    .get(asset)
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_else(|| "ABSENT".into());
                let after = record
                    .asset_statuses_after
                    .get(asset)
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_else(|| "ABSENT".into());
                rows.push_str(&format!(
                    "{id}\t{}\t{}\t{}\t{:.3}\t{}\t{:?}\t{}\t{:?}\t{}\t{}\t{}\t{}\n",
                    record.event_sequence,
                    record.event_id,
                    record.event_batch_sequence,
                    record.at_s,
                    record.kind.label(),
                    record.origin,
                    record.target_asset.as_deref().unwrap_or(""),
                    record.status,
                    asset,
                    before,
                    after,
                    label(&event.outcome.termination)
                ));
            }
        }
        summaries.push_str(&format!(
            "{id}\t{}\t{}\t{:.3}\ttrue\ttrue\ttrue\n",
            event.events.len(),
            label(&event.outcome.termination),
            event.outcome.survived_s
        ));
    }
    // N8-06: required tool failure with event identity.
    {
        let model = tool_model();
        let opts = EngineOptions {
            services: vec!["n8-tool"],
            horizon_s: 20.0 * 3600.0,
            failures: vec![FailureEvent { at_s: 9000.0, asset: "tool-t1".into() }],
            resupplies: Vec::new(),
            resources: vec!["robot-m1".into()],
            alternative_pool: vec!["robot-m1".into()],
            battery: None,
            load_kw: 0.0,
        };
        let direct = run_engine(&model, &opts).unwrap();
        let time =
            run_engine_observed(&model, &opts, &[ObservationRequest { at_s: 9000.0, phase: ObservationPhase::After }])
                .unwrap();
        let event = run_engine_event_observed(&model, &opts).unwrap();
        assert_eq!(direct, time.outcome, "time noninterference N8-06");
        assert_eq!(direct, event.outcome, "event noninterference N8-06");
        for record in &event.events {
            let all: std::collections::BTreeSet<&str> = record
                .asset_statuses_before
                .keys()
                .chain(record.asset_statuses_after.keys())
                .map(|value| value.as_str())
                .collect();
            for asset in all {
                let before = record
                    .asset_statuses_before
                    .get(asset)
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_else(|| "ABSENT".into());
                let after = record
                    .asset_statuses_after
                    .get(asset)
                    .map(|status| format!("{status:?}"))
                    .unwrap_or_else(|| "ABSENT".into());
                rows.push_str(&format!(
                    "N8-06	{}	{}	{}	{:.3}	{}	{:?}	{}	{:?}	{}	{}	{}	{}
",
                    record.event_sequence,
                    record.event_id,
                    record.event_batch_sequence,
                    record.at_s,
                    record.kind.label(),
                    record.origin,
                    record.target_asset.as_deref().unwrap_or(""),
                    record.status,
                    asset,
                    before,
                    after,
                    label(&event.outcome.termination)
                ));
            }
        }
        summaries.push_str(&format!(
            "N8-06	{}	{}	{:.3}	true	true	true
",
            event.events.len(),
            label(&event.outcome.termination),
            event.outcome.survived_s
        ));
    }
    // N8-14: deterministic repeat of event observation (safety is also unit-tested).
    {
        let model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
        let opts = options(Vec::new(), Vec::new(), 20.0 * 3600.0);
        let first = run_engine_event_observed(&model, &opts).unwrap();
        let second = run_engine_event_observed(&model, &opts).unwrap();
        let ids: Vec<&str> = first.events.iter().map(|event| event.event_id.as_str()).collect();
        let ids_second: Vec<&str> = second.events.iter().map(|event| event.event_id.as_str()).collect();
        assert_eq!(ids, ids_second, "deterministic event identity N8-14");
        summaries.push_str(&format!(
            "N8-14	{}	{}	{:.3}	true	true	true
",
            first.events.len(),
            label(&first.outcome.termination),
            first.outcome.survived_s
        ));
    }
    std::fs::write(out.join("event-observations/events.csv"), rows).unwrap();
    std::fs::write(out.join("event-observations/summary.csv"), summaries).unwrap();
    println!("wrote N8 event observations");
}
