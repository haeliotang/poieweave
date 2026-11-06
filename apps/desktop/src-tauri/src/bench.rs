//! Desktop M9 bench scenarios: gate, duplicate/expired/fallback/outcome_unknown.
//! Simulated only; no physical adapter and no numeric-observation change.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use poie_exec::gate::{DispatchOutcome, ExecutionGate, GateLease};
use poie_exec::verify::{verify_action, VerificationSignal};
use poie_exec::writable::{Command, ExecError, Receipt, WritableAdapter, WritableDeclaration};
use poie_state::action::{Action, ActionState, Precondition, Reversibility, VerificationSpec};
use poie_state::record::Timestamp;
use serde::Serialize;

#[derive(Default)]
pub struct BenchUiState {
    pub unknown: bool,
    pub fallback: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchCheck {
    pub id: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchReport {
    pub scenario: String,
    pub passed: bool,
    pub checks: Vec<BenchCheck>,
    pub physical_executions: u32,
    pub unknown: bool,
    pub fallback: bool,
}

struct Bench {
    valve_open: bool,
    stuck: bool,
    unreachable: bool,
    executions: u32,
}

struct Adapter {
    declaration: WritableDeclaration,
    bench: Rc<RefCell<Bench>>,
    keys: BTreeMap<String, String>,
}
impl Adapter {
    fn new(bench: Rc<RefCell<Bench>>) -> Self {
        Self {
            declaration: WritableDeclaration {
                adapter_id: "desktop-bench".into(),
                version: "0.1.0".into(),
                source: "desktop-sim".into(),
                supports_idempotency: true,
                max_command_validity_ms: 10_000,
                dedup_window_ms: 60_000,
                supports_lease_epoch: true,
                verification_sources: vec!["flow-meter".into()],
                compatibility: Some("desktop simulation; no hardware".into()),
            },
            bench,
            keys: BTreeMap::new(),
        }
    }
}
impl WritableAdapter for Adapter {
    fn name(&self) -> &str {
        &self.declaration.adapter_id
    }
    fn declaration(&self) -> &WritableDeclaration {
        &self.declaration
    }
    fn dispatch(&mut self, command: &Command) -> Result<Receipt, ExecError> {
        if self.bench.borrow().unreachable {
            return Err(ExecError::Disconnected("simulated PLC unreachable".into()));
        }
        if let Some(original) = self.keys.get(&command.idempotency_key) {
            return Err(ExecError::Duplicate(original.clone()));
        }
        self.keys.insert(command.idempotency_key.clone(), command.action_id.clone());
        let mut bench = self.bench.borrow_mut();
        bench.valve_open = true;
        bench.executions += 1;
        Ok(Receipt {
            action_id: command.action_id.clone(),
            accepted: true,
            executed_at_ms: command.valid_from_ms,
            detail: "simulated ACK".into(),
        })
    }
}

struct FlowSensor {
    bench: Rc<RefCell<Bench>>,
}
impl VerificationSignal for FlowSensor {
    fn source(&self) -> &str {
        "flow-meter"
    }
    fn observe(&mut self, _action_id: &str) -> Option<bool> {
        let bench = self.bench.borrow();
        Some(bench.valve_open && !bench.stuck)
    }
}

fn action(id: &str, key: &str, valid_until: i64) -> Action {
    Action {
        action_id: id.into(),
        idempotency_key: key.into(),
        target: "desktop-bench".into(),
        command: "pump_on".into(),
        plan_ref: Some("bench".into()),
        policy_version: "bench/0.1".into(),
        precondition: Precondition { state_version: BTreeMap::from([("site-a".into(), 1)]), predicates: vec![] },
        valid_from: Timestamp(0),
        valid_until: Timestamp(valid_until),
        budget: vec![],
        reservations: vec![],
        reversibility: Reversibility::Idempotent,
        verification_spec: VerificationSpec {
            observe: "flow-meter".into(),
            condition: "flow".into(),
            window_ms: 5_000,
        },
        authorization: None,
        redispatch_of: None,
    }
}

fn setup(stuck: bool, unreachable: bool) -> (ExecutionGate, Adapter, Rc<RefCell<Bench>>) {
    let bench = Rc::new(RefCell::new(Bench { valve_open: false, stuck, unreachable, executions: 0 }));
    let mut gate = ExecutionGate::new();
    gate.set_lease(Some(GateLease { epoch: 1, dispatch_deadline_ms: 20_000 }));
    (gate, Adapter::new(Rc::clone(&bench)), bench)
}

pub fn run(state: &mut BenchUiState, scenario: &str) -> Result<BenchReport, String> {
    let mut checks = Vec::new();
    let physical;
    match scenario {
        "B3" => {
            let (mut gate, mut adapter, bench) = setup(false, false);
            let a = action("a3", "key-b3", 10_000);
            gate.register_authorized(&a).map_err(|e| e.to_string())?;
            gate.dispatch(&a, &mut adapter, 100).map_err(|e| e.to_string())?;
            let before = bench.borrow().executions;
            let duplicate = adapter.dispatch(&Command {
                action_id: "a3b".into(),
                idempotency_key: "key-b3".into(),
                target: "desktop-bench".into(),
                command: "pump_on".into(),
                valid_from_ms: 0,
                valid_until_ms: 10_000,
                lease_epoch: Some(1),
            });
            let after = bench.borrow().executions;
            checks.push(BenchCheck {
                id: "B3".into(),
                passed: matches!(duplicate, Err(ExecError::Duplicate(_))) && before == after,
                detail: format!("duplicate={}; executions {before}->{after}", duplicate.is_err()),
            });
            physical = after;
        }
        "B4" => {
            let (mut gate, mut adapter, bench) = setup(false, false);
            let a = action("a4", "key-b4", 500);
            gate.register_authorized(&a).map_err(|e| e.to_string())?;
            let outcome = gate.dispatch(&a, &mut adapter, 1_000).map_err(|e| e.to_string())?;
            checks.push(BenchCheck {
                id: "B4".into(),
                passed: matches!(outcome, DispatchOutcome::Rejected { .. }),
                detail: format!("rejected={}", !outcome.dispatched()),
            });
            physical = bench.borrow().executions;
        }
        "B5" => {
            let (mut gate, mut adapter, _bench) = setup(false, true);
            let a = action("a5", "key-b5", 10_000);
            gate.register_authorized(&a).map_err(|e| e.to_string())?;
            let outcome = gate.dispatch(&a, &mut adapter, 100).map_err(|e| e.to_string())?;
            let fallback = matches!(outcome, DispatchOutcome::Fallback { .. });
            state.fallback = fallback;
            checks.push(BenchCheck { id: "B5".into(), passed: fallback, detail: format!("fallback={fallback}") });
            physical = 0;
        }
        "B6" => {
            let (mut gate, mut adapter, bench) = setup(true, false);
            let a = action("a6", "key-b6", 10_000);
            gate.register_authorized(&a).map_err(|e| e.to_string())?;
            gate.dispatch(&a, &mut adapter, 100).map_err(|e| e.to_string())?;
            let mut signals: Vec<Box<dyn VerificationSignal>> = vec![Box::new(FlowSensor { bench: Rc::clone(&bench) })];
            let verified = verify_action("a6", "desktop-bench", &mut signals).map_err(|e| e.to_string())?;
            let unknown = gate.record_outcome("a6", ActionState::OutcomeUnknown).is_ok();
            state.unknown = unknown;
            checks.push(BenchCheck {
                id: "B6".into(),
                passed: !verified.confirmed && unknown,
                detail: format!("flow_confirmed={}; outcome_unknown={unknown}", verified.confirmed),
            });
            physical = bench.borrow().executions;
        }
        other => return Err(format!("unknown scenario {other}")),
    }
    Ok(BenchReport {
        scenario: scenario.into(),
        passed: checks.iter().all(|c| c.passed),
        checks,
        physical_executions: physical,
        unknown: state.unknown,
        fallback: state.fallback,
    })
}

pub fn reset(state: &mut BenchUiState, operator_confirmed: bool) -> Result<(bool, bool), String> {
    if !operator_confirmed {
        return Err("operator_confirmation_required: confirm the physical bench is safe".into());
    }
    state.unknown = false;
    state.fallback = false;
    Ok((state.unknown, state.fallback))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_bench_scenarios_pass() {
        let mut state = BenchUiState::default();
        for scenario in ["B3", "B4", "B5", "B6"] {
            let report = run(&mut state, scenario).unwrap();
            assert!(report.passed, "{scenario}: {:?}", report.checks);
        }
        assert!(state.unknown);
        assert!(state.fallback);
    }

    #[test]
    fn reset_requires_operator_confirmation() {
        let mut state = BenchUiState { unknown: true, fallback: true };
        assert!(reset(&mut state, false).is_err());
        assert!(!reset(&mut state, true).unwrap().0);
    }
}
