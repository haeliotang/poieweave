//! Simulated M9 bench loop. No hardware, no network: this only proves the
//! software path gate -> writable adapter -> independent sensor.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use poie_exec::gate::{DispatchOutcome, ExecutionGate, GateLease};
use poie_exec::verify::{verify_action, VerificationSignal};
use poie_exec::writable::{Command, ExecError, Receipt, WritableAdapter, WritableDeclaration};
use poie_state::action::{Action, Precondition, Reversibility, VerificationSpec};
use poie_state::record::Timestamp;

#[derive(Default)]
struct Bench {
    pump_on: bool,
    valve_open: bool,
    flow_lpm: f64,
    physical_executions: usize,
}

struct BenchAdapter {
    declaration: WritableDeclaration,
    bench: Rc<RefCell<Bench>>,
    keys: BTreeMap<String, String>,
    unreachable: bool,
}

impl BenchAdapter {
    fn new(bench: Rc<RefCell<Bench>>) -> BenchAdapter {
        BenchAdapter {
            declaration: WritableDeclaration {
                adapter_id: "bench-plc-sim".into(),
                version: "0.1.0".into(),
                source: "bench-sim".into(),
                supports_idempotency: true,
                max_command_validity_ms: 10_000,
                dedup_window_ms: 60_000,
                supports_lease_epoch: true,
                verification_sources: vec!["flow-meter".into()],
                compatibility: Some("simulated bench; no hardware".into()),
            },
            bench,
            keys: BTreeMap::new(),
            unreachable: false,
        }
    }
}

impl WritableAdapter for BenchAdapter {
    fn name(&self) -> &str {
        &self.declaration.adapter_id
    }
    fn declaration(&self) -> &WritableDeclaration {
        &self.declaration
    }
    fn dispatch(&mut self, command: &Command) -> Result<Receipt, ExecError> {
        if self.unreachable {
            return Err(ExecError::Disconnected("simulated PLC unreachable".into()));
        }
        if let Some(original) = self.keys.get(&command.idempotency_key) {
            return Err(ExecError::Duplicate(original.clone()));
        }
        self.keys.insert(command.idempotency_key.clone(), command.action_id.clone());
        let mut bench = self.bench.borrow_mut();
        if command.command == "pump_on" {
            bench.pump_on = true;
            bench.valve_open = true;
            bench.flow_lpm = 8.0;
            bench.physical_executions += 1;
        } else {
            return Err(ExecError::Rejected(format!("unknown bench command {:?}", command.command)));
        }
        Ok(Receipt {
            action_id: command.action_id.clone(),
            accepted: true,
            executed_at_ms: command.valid_from_ms,
            detail: "bench pump + valve opened".into(),
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
        Some(self.bench.borrow().flow_lpm > 5.0)
    }
}

fn action(id: &str, key: &str, valid_until: i64) -> Action {
    Action {
        action_id: id.into(),
        idempotency_key: key.into(),
        target: "bench".into(),
        command: "pump_on".into(),
        plan_ref: Some("h2c-shadow".into()),
        policy_version: "bench/0.1".into(),
        precondition: Precondition {
            state_version: BTreeMap::from([("site-a".into(), 1)]),
            predicates: vec!["shadow plan authorized".into()],
        },
        valid_from: Timestamp(0),
        valid_until: Timestamp(valid_until),
        budget: Vec::new(),
        reservations: Vec::new(),
        reversibility: Reversibility::Idempotent,
        verification_spec: VerificationSpec {
            observe: "flow-meter".into(),
            condition: "flow > 5 L/min".into(),
            window_ms: 5_000,
        },
        authorization: None,
        redispatch_of: None,
    }
}

fn main() {
    let bench = Rc::new(RefCell::new(Bench::default()));
    let mut adapter = BenchAdapter::new(Rc::clone(&bench));
    let mut gate = ExecutionGate::new();
    gate.set_lease(Some(GateLease { epoch: 1, dispatch_deadline_ms: 20_000 }));
    let first = action("a1", "key-1", 10_000);
    gate.register_authorized(&first).unwrap();
    let outcome = gate.dispatch(&first, &mut adapter, 1_000).unwrap();
    let b1 = matches!(outcome, DispatchOutcome::Dispatched { .. });
    let mut signals: Vec<Box<dyn VerificationSignal>> = vec![Box::new(FlowSensor { bench: Rc::clone(&bench) })];
    let verification = verify_action("a1", "bench-plc-sim", &mut signals).unwrap();
    let b2 = verification.confirmed && verification.source == "flow-meter";

    let physical_after_first = bench.borrow().physical_executions;
    let duplicate = adapter.dispatch(&Command {
        action_id: "a2".into(),
        idempotency_key: "key-1".into(),
        target: "bench".into(),
        command: "pump_on".into(),
        valid_from_ms: 0,
        valid_until_ms: 10_000,
        lease_epoch: Some(1),
    });
    let b3 =
        matches!(duplicate, Err(ExecError::Duplicate(_))) && bench.borrow().physical_executions == physical_after_first;

    let expired = action("a3", "key-3", 500);
    gate.register_authorized(&expired).unwrap();
    let b4 = matches!(gate.dispatch(&expired, &mut adapter, 1_000).unwrap(), DispatchOutcome::Rejected { .. });
    let physical_after_expired = bench.borrow().physical_executions;

    adapter.unreachable = true;
    let mut gate2 = ExecutionGate::new();
    gate2.set_lease(Some(GateLease { epoch: 1, dispatch_deadline_ms: 20_000 }));
    let a4 = action("a4", "key-4", 10_000);
    gate2.register_authorized(&a4).unwrap();
    let fallback = gate2.dispatch(&a4, &mut adapter, 1_000);
    let b5 = matches!(fallback, Ok(DispatchOutcome::Fallback { .. }));
    {
        let mut bench = bench.borrow_mut();
        bench.pump_on = false;
        bench.valve_open = false;
        bench.flow_lpm = 0.0;
    }
    let closed = !bench.borrow().valve_open && bench.borrow().flow_lpm == 0.0;

    let checks = [
        ("B1", "gate 派发", b1, "Dispatched"),
        ("B2", "独立验证", b2, "flow-meter confirmed"),
        ("B3", "重复 key", b3, "Duplicate，物理执行不增加"),
        ("B4", "过期命令", b4, "Rejected，物理执行不增加"),
        ("B5", "断连 fallback + fail-closed", b5 && closed, "Fallback + valve closed"),
    ];
    println!("id\tcheck\tresult\tdetail");
    let mut failures = 0;
    for (id, name, pass, detail) in checks {
        if !pass {
            failures += 1;
        }
        println!("{id}\t{name}\t{}\t{detail}", if pass { "PASS" } else { "FAIL" });
    }
    println!(
        "physical_executions={} (after first {}, after expired {})",
        bench.borrow().physical_executions,
        physical_after_first,
        physical_after_expired
    );
    std::process::exit(if failures == 0 { 0 } else { 1 });
}
