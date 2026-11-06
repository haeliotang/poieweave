//! Simulated bench v2: physical settling, ACK-without-effect, PLC watchdog.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use poie_exec::gate::{DispatchOutcome, ExecutionGate, GateLease};
use poie_exec::verify::{verify_action, VerificationSignal};
use poie_exec::writable::{Command, ExecError, Receipt, WritableAdapter, WritableDeclaration};
use poie_state::action::{Action, ActionState, Precondition, Reversibility, VerificationSpec};
use poie_state::record::Timestamp;

const SETTLING_MS: i64 = 3_000;
const WATCHDOG_MS: i64 = 2_000;

#[derive(Clone, Copy, PartialEq)]
enum Fault {
    None,
    StuckValve,
    DryTank,
}

struct Bench {
    valve_open: bool,
    pump_on: bool,
    command_at_ms: i64,
    fault: Fault,
    last_heartbeat_ms: i64,
    watchdog_tripped: bool,
    flow_locked: bool,
    physical_executions: usize,
}
impl Bench {
    fn tick(&mut self, now_ms: i64) {
        if !self.watchdog_tripped && now_ms - self.last_heartbeat_ms > WATCHDOG_MS {
            self.valve_open = false;
            self.pump_on = false;
            self.watchdog_tripped = true;
        }
    }
    fn flow_at(&mut self, now_ms: i64) -> f64 {
        self.tick(now_ms);
        if self.fault != Fault::None || !self.valve_open || !self.pump_on {
            return 0.0;
        }
        if now_ms < self.command_at_ms + SETTLING_MS {
            return 0.0;
        }
        8.0
    }
    fn heartbeat(&mut self, now_ms: i64) {
        self.last_heartbeat_ms = now_ms;
        self.tick(now_ms);
    }
    fn safe_clear(&mut self, now_ms: i64) -> bool {
        self.heartbeat(now_ms);
        if self.valve_open || self.flow_at(now_ms) > 0.0 {
            return false;
        }
        self.watchdog_tripped = false;
        self.fault = Fault::None;
        true
    }
}

struct BenchAdapter {
    declaration: WritableDeclaration,
    bench: Rc<RefCell<Bench>>,
    keys: BTreeMap<String, String>,
    unreachable: bool,
}
impl BenchAdapter {
    fn new(bench: Rc<RefCell<Bench>>) -> Self {
        Self {
            declaration: WritableDeclaration {
                adapter_id: "bench-v2-plc".into(),
                version: "0.1.0".into(),
                source: "bench-v2".into(),
                supports_idempotency: true,
                max_command_validity_ms: 10_000,
                dedup_window_ms: 60_000,
                supports_lease_epoch: true,
                verification_sources: vec!["flow-meter".into()],
                compatibility: Some("simulated bench v2; no hardware".into()),
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
        if self.bench.borrow().watchdog_tripped {
            return Err(ExecError::Rejected("watchdog tripped; no dispatch".into()));
        }
        if self.bench.borrow().flow_locked {
            return Err(ExecError::Rejected("flow interlock; outcome_unknown".into()));
        }
        if let Some(original) = self.keys.get(&command.idempotency_key) {
            return Err(ExecError::Duplicate(original.clone()));
        }
        self.keys.insert(command.idempotency_key.clone(), command.action_id.clone());
        let mut bench = self.bench.borrow_mut();
        bench.command_at_ms = command.valid_from_ms.max(0);
        bench.valve_open = true;
        bench.pump_on = bench.fault != Fault::DryTank && bench.fault != Fault::StuckValve;
        bench.physical_executions += 1;
        Ok(Receipt {
            action_id: command.action_id.clone(),
            accepted: true,
            executed_at_ms: command.valid_from_ms,
            detail: "ACK".into(),
        })
    }
}

struct FlowSensor {
    bench: Rc<RefCell<Bench>>,
    now_ms: i64,
}
impl VerificationSignal for FlowSensor {
    fn source(&self) -> &str {
        "flow-meter"
    }
    fn observe(&mut self, _action_id: &str) -> Option<bool> {
        Some(self.bench.borrow_mut().flow_at(self.now_ms) > 5.0)
    }
}

fn action(id: &str, key: &str) -> Action {
    Action {
        action_id: id.into(),
        idempotency_key: key.into(),
        target: "bench".into(),
        command: "pump_on".into(),
        plan_ref: Some("shadow".into()),
        policy_version: "bench/0.2".into(),
        precondition: Precondition { state_version: BTreeMap::from([("site-a".into(), 1)]), predicates: vec![] },
        valid_from: Timestamp(0),
        valid_until: Timestamp(10_000),
        budget: vec![],
        reservations: vec![],
        reversibility: Reversibility::Idempotent,
        verification_spec: VerificationSpec {
            observe: "flow-meter".into(),
            condition: "flow > 5".into(),
            window_ms: 5_000,
        },
        authorization: None,
        redispatch_of: None,
    }
}

fn setup() -> (ExecutionGate, BenchAdapter, Rc<RefCell<Bench>>) {
    let bench = Rc::new(RefCell::new(Bench {
        valve_open: false,
        pump_on: false,
        command_at_ms: 0,
        fault: Fault::None,
        last_heartbeat_ms: 0,
        watchdog_tripped: false,
        flow_locked: false,
        physical_executions: 0,
    }));
    bench.borrow_mut().heartbeat(0);
    let mut gate = ExecutionGate::new();
    gate.set_lease(Some(GateLease { epoch: 1, dispatch_deadline_ms: 20_000 }));
    (gate, BenchAdapter::new(Rc::clone(&bench)), bench)
}

fn main() {
    let mut checks: Vec<(&str, bool, String)> = Vec::new();

    // B6: ACK but no physical effect.
    {
        let (mut gate, mut adapter, bench) = setup();
        bench.borrow_mut().fault = Fault::StuckValve;
        let a = action("a6", "key-6");
        gate.register_authorized(&a).unwrap();
        let dispatched = matches!(gate.dispatch(&a, &mut adapter, 100).unwrap(), DispatchOutcome::Dispatched { .. });
        bench.borrow_mut().heartbeat(1_100);
        let mut s1: Vec<Box<dyn VerificationSignal>> =
            vec![Box::new(FlowSensor { bench: Rc::clone(&bench), now_ms: 1_100 })];
        let v1 = verify_action("a6", "bench-v2-plc", &mut s1).unwrap();
        bench.borrow_mut().heartbeat(5_100);
        let mut s5: Vec<Box<dyn VerificationSignal>> =
            vec![Box::new(FlowSensor { bench: Rc::clone(&bench), now_ms: 5_100 })];
        let v5 = verify_action("a6", "bench-v2-plc", &mut s5).unwrap();
        let unknown = gate.record_outcome("a6", ActionState::OutcomeUnknown).is_ok();
        bench.borrow_mut().flow_locked = unknown;
        let before_second = bench.borrow().physical_executions;
        bench.borrow_mut().heartbeat(5_200);
        let a6b = action("a6b", "key-6b");
        gate.register_authorized(&a6b).unwrap();
        let second = gate.dispatch(&a6b, &mut adapter, 5_200);
        let locked = matches!(second, Ok(DispatchOutcome::Rejected { .. }))
            && bench.borrow().physical_executions == before_second;
        checks.push((
            "B6",
            dispatched && !v1.confirmed && !v5.confirmed && unknown && locked,
            format!(
                "dispatched={dispatched}; verify1={}; verify5={}; outcome_unknown={unknown}; flow_locked={locked}",
                v1.confirmed, v5.confirmed
            ),
        ));
    }

    // B7: settling window.
    {
        let (mut gate, mut adapter, bench) = setup();
        let a = action("a7", "key-7");
        gate.register_authorized(&a).unwrap();
        let dispatched = matches!(gate.dispatch(&a, &mut adapter, 100).unwrap(), DispatchOutcome::Dispatched { .. });
        bench.borrow_mut().heartbeat(1_100);
        let mut early: Vec<Box<dyn VerificationSignal>> =
            vec![Box::new(FlowSensor { bench: Rc::clone(&bench), now_ms: 1_100 })];
        let early = verify_action("a7", "bench-v2-plc", &mut early).unwrap();
        bench.borrow_mut().heartbeat(4_100);
        let mut settled: Vec<Box<dyn VerificationSignal>> =
            vec![Box::new(FlowSensor { bench: Rc::clone(&bench), now_ms: 4_100 })];
        let settled = verify_action("a7", "bench-v2-plc", &mut settled).unwrap();
        let state = gate.ledger().state("a7");
        checks.push((
            "B7",
            dispatched && !early.confirmed && settled.confirmed && state == Some(ActionState::Acknowledged),
            format!("early={}; settled={}; state={state:?}", early.confirmed, settled.confirmed),
        ));
    }

    // B5R: PLC watchdog closes valve; reconnect needs explicit clear.
    {
        let (mut gate, mut adapter, bench) = setup();
        bench.borrow_mut().heartbeat(100);
        let a = action("a5", "key-5");
        gate.register_authorized(&a).unwrap();
        let _ = gate.dispatch(&a, &mut adapter, 100);
        // Host stops heartbeating; PLC tick at 2.5 s trips.
        bench.borrow_mut().tick(2_600);
        let tripped = bench.borrow().watchdog_tripped;
        let closed = !bench.borrow().valve_open && bench.borrow_mut().flow_at(2_600) == 0.0;
        // Reconnect: must not dispatch before clear.
        let blocked_action = action("a5b", "key-5b");
        gate.register_authorized(&blocked_action).unwrap();
        let blocked =
            matches!(gate.dispatch(&blocked_action, &mut adapter, 2_650), Ok(DispatchOutcome::Rejected { .. }));
        let cleared = bench.borrow_mut().safe_clear(2_700);
        let rearm_action = action("a5c", "key-5c");
        gate.register_authorized(&rearm_action).unwrap();
        let rearmed = cleared
            && !bench.borrow().watchdog_tripped
            && bench.borrow_mut().flow_at(2_700) == 0.0
            && matches!(gate.dispatch(&rearm_action, &mut adapter, 2_700), Ok(DispatchOutcome::Dispatched { .. }));
        checks.push((
            "B5R",
            tripped && closed && blocked && rearmed,
            format!("tripped={tripped}; valve_closed={closed}; blocked_before_clear={blocked}; cleared={cleared}; rearmed={rearmed}"),
        ));
    }

    println!("id\tresult\tdetail");
    let mut failures = 0;
    for (id, pass, detail) in &checks {
        if !pass {
            failures += 1;
        }
        println!("{id}\t{}\t{}", if *pass { "PASS" } else { "FAIL" }, detail);
    }
    println!("failures={failures}");
    std::process::exit(if failures == 0 { 0 } else { 1 });
}
