// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! N4 canonical event runner: PV failure cases 00/01/02.

use std::path::{Path, PathBuf};

use poie_ontology::model::Model;
use poie_viability::engine::{
    run_engine, run_engine_observed, EngineOptions, EngineTermination, ObservationPhase, ObservationRequest,
};
use poie_viability::survival::{BatteryLimits, FailureEvent};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn termination_label(termination: &EngineTermination) -> String {
    match termination {
        EngineTermination::CompletedHorizon => "completed".into(),
        EngineTermination::Blocked(reason) => format!("blocked:{reason}"),
        EngineTermination::IncompleteAtHorizon(reason) => format!("incomplete:{reason}"),
    }
}

fn options(failures: Vec<FailureEvent>) -> EngineOptions<'static> {
    EngineOptions {
        services: vec!["potable-water", "workshop-operability"],
        horizon_s: 20.0 * 3600.0,
        failures,
        resupplies: Vec::new(),
        resources: vec!["robot-m1".into(), "lathe-1".into()],
        alternative_pool: vec!["robot-m1".into()],
        battery: Some(BatteryLimits { capacity_kwh: 200.0, charge_kw: 10.0 }),
        load_kw: 3.0,
    }
}

fn main() {
    let model = Model::load(&root().join("scenarios/base-08/model.toml")).unwrap();
    let cases = [("N4-00", None), ("N4-01", Some(7200.0)), ("N4-02", Some(4500.0)), ("N4-03", Some(10800.0))];
    let out = root().join("audit/phase4/canonical");
    std::fs::create_dir_all(&out).unwrap();
    for (case, failure_s) in cases {
        let failures =
            failure_s.into_iter().map(|at_s| FailureEvent { at_s, asset: "pv-1".into() }).collect::<Vec<_>>();
        let options = options(failures);
        let mut requests = Vec::new();
        for at_s in [0.0, 3600.0, 7200.0, 10800.0, 14400.0] {
            requests.push(ObservationRequest { at_s, phase: ObservationPhase::After });
        }
        if let Some(failure_s) = failure_s {
            requests.push(ObservationRequest { at_s: failure_s, phase: ObservationPhase::Before });
            requests.push(ObservationRequest { at_s: failure_s, phase: ObservationPhase::After });
        }
        requests.push(ObservationRequest { at_s: 10800.0, phase: ObservationPhase::Before });
        requests.sort_by(|left, right| left.at_s.total_cmp(&right.at_s).then_with(|| left.phase.cmp(&right.phase)));
        requests.dedup();
        let direct = run_engine(&model, &options).unwrap();
        let observed = run_engine_observed(&model, &options, &requests).unwrap();
        assert_eq!(direct, observed.outcome, "native observation noninterference failed for {case}");
        let mut rows = String::from(
            "at_s\tphase\tstatus\tenergy_kwh\tgenerated_kwh\tload_kwh\trepair_kwh\tcurtailed_kwh\tresupply_kwh\trepair_active\trepair_label\ttermination\n",
        );
        for row in &observed.observations {
            rows.push_str(&format!(
                "{:.3}\t{:?}\t{}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\t{}\t{}\t{}\n",
                row.at_s,
                row.phase,
                row.status.label(),
                row.inventory.get("electric-energy").copied().unwrap_or(0.0) / 3_600_000.0,
                row.energy_generated_j / 3_600_000.0,
                row.energy_load_j / 3_600_000.0,
                row.energy_repair_j / 3_600_000.0,
                row.energy_curtailed_j / 3_600_000.0,
                row.energy_resupply_j / 3_600_000.0,
                row.running_repair.is_some(),
                row.running_repair.as_ref().map(|repair| repair.label.as_str()).unwrap_or("NONE"),
                termination_label(&observed.outcome.termination),
            ));
        }
        std::fs::write(out.join(format!("{case}-observations.csv")), rows).unwrap();
        let mut events =
            String::from("at_s\tkind\tplans\tresources\tgenerated_kwh\tload_kwh\trepair_kwh\tenergy_after_kwh\n");
        for row in &observed.outcome.trace {
            events.push_str(&format!(
                "{:.3}\t{}\t{}\t{}\t{:.9}\t{:.9}\t{:.9}\t{:.9}\n",
                row.at_s,
                row.kind,
                row.plans.join("+"),
                row.resources.join("+"),
                row.energy_generated_j / 3_600_000.0,
                row.energy_load_j / 3_600_000.0,
                row.energy_repair_j / 3_600_000.0,
                row.inventory_after.get("electric-energy").copied().unwrap_or(0.0) / 3_600_000.0,
            ));
        }
        std::fs::write(out.join(format!("{case}-events.csv")), events).unwrap();
        let summary = format!(
            "{{\n  \"case\": \"{case}\",\n  \"termination\": \"{}\",\n  \"survived_s\": {:.6},\n  \"final_energy_kwh\": {:.9}\n}}\n",
            termination_label(&observed.outcome.termination),
            observed.outcome.survived_s,
            observed.outcome.final_inventory.get("electric-energy").copied().unwrap_or(0.0) / 3_600_000.0,
        );
        std::fs::write(out.join(format!("{case}-summary.json")), summary).unwrap();
        println!(
            "{case}: {} survived={:.3}s final_energy={:.6}kWh",
            termination_label(&observed.outcome.termination),
            observed.outcome.survived_s,
            observed.outcome.final_inventory.get("electric-energy").copied().unwrap_or(0.0) / 3_600_000.0,
        );
    }
}
