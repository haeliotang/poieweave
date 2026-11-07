//! M9 Modbus TCP rehearsal: local virtual PLC, FC16 register contract, FC03 reads.
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use poie_exec::gate::{DispatchOutcome, ExecutionGate, GateLease};
use poie_exec::writable::{Command, ExecError, Receipt, WritableAdapter, WritableDeclaration};
use poie_state::action::{Action, Precondition, Reversibility, VerificationSpec};
use poie_state::record::Timestamp;

const WATCHDOG: Duration = Duration::from_millis(2_000);

struct PlcState {
    last_seq: u16,
    executions: u32,
    valve_open: bool,
    pump_on: bool,
    fault: bool,
    last_heartbeat: Instant,
    exception: Option<u8>,
}
impl PlcState {
    fn new() -> Self {
        Self {
            last_seq: 0,
            executions: 0,
            valve_open: false,
            pump_on: false,
            fault: false,
            last_heartbeat: Instant::now(),
            exception: None,
        }
    }
    fn flow_x10(&self) -> u16 {
        if self.valve_open && self.pump_on && !self.fault {
            80
        } else {
            0
        }
    }
    fn registers(&self) -> [u16; 12] {
        [
            1,
            self.last_seq,
            u16::from(self.valve_open),
            5_000,
            0,
            u16::from(self.fault),
            self.flow_x10(),
            u16::from(self.valve_open),
            u16::from(self.pump_on),
            0,
            0,
            self.executions.min(u32::from(u16::MAX)) as u16,
        ]
    }
}

fn apply_write(state: &mut PlcState, start: u16, values: &[u16]) -> Result<(), u8> {
    if let Some(code) = state.exception.take() {
        return Err(code);
    }
    match start {
        0 if values.len() >= 4 => {
            let seq = values[1];
            if seq <= state.last_seq {
                return Ok(());
            } // stale sequence: ignore, no physical action
            state.last_seq = seq;
            state.executions += 1;
            state.valve_open = values[2] == 1;
            state.pump_on = values[2] == 1;
            Ok(())
        }
        4 => {
            state.last_heartbeat = Instant::now();
            Ok(())
        }
        9 => {
            state.exception = Some(values[0] as u8);
            Ok(())
        }
        10 => {
            if values[0] == 1 && !state.valve_open && state.flow_x10() == 0 {
                state.fault = false;
                Ok(())
            } else {
                Err(4)
            }
        }
        _ => Err(2),
    }
}

fn handle(mut stream: TcpStream, state: Arc<Mutex<PlcState>>) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    loop {
        let mut header = [0u8; 7];
        if stream.read_exact(&mut header).is_err() {
            return;
        }
        let tid = [header[0], header[1]];
        let length = u16::from_be_bytes([header[4], header[5]]) as usize;
        if !(2..=260).contains(&length) {
            return;
        }
        let mut pdu = vec![0u8; length - 1];
        if stream.read_exact(&mut pdu).is_err() {
            return;
        }
        let fc = pdu[0];
        let response = match fc {
            3 => {
                let start = u16::from_be_bytes([pdu[1], pdu[2]]);
                let qty = u16::from_be_bytes([pdu[3], pdu[4]]) as usize;
                let regs = state.lock().unwrap().registers();
                if start as usize + qty > regs.len() {
                    vec![0x83, 0x02]
                } else {
                    let mut out = vec![0x03, (qty * 2) as u8];
                    for value in &regs[start as usize..start as usize + qty] {
                        out.extend_from_slice(&value.to_be_bytes());
                    }
                    out
                }
            }
            16 => {
                let start = u16::from_be_bytes([pdu[1], pdu[2]]);
                let qty = u16::from_be_bytes([pdu[3], pdu[4]]) as usize;
                let byte_count = pdu[5] as usize;
                if byte_count != qty * 2 || pdu.len() < 6 + byte_count {
                    vec![0x90, 0x03]
                } else {
                    let values: Vec<u16> = pdu[6..6 + byte_count]
                        .chunks(2)
                        .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                        .collect();
                    match apply_write(&mut state.lock().unwrap(), start, &values) {
                        Ok(()) => {
                            let mut out = vec![0x10];
                            out.extend_from_slice(&start.to_be_bytes());
                            out.extend_from_slice(&(qty as u16).to_be_bytes());
                            out
                        }
                        Err(code) => vec![0x90, code],
                    }
                }
            }
            _ => vec![fc | 0x80, 0x01],
        };
        let len = (response.len() + 1) as u16;
        let mut frame = Vec::with_capacity(7 + response.len());
        frame.extend_from_slice(&tid);
        frame.extend_from_slice(&[0, 0]);
        frame.extend_from_slice(&len.to_be_bytes());
        frame.push(1);
        frame.extend_from_slice(&response);
        if stream.write_all(&frame).is_err() {
            return;
        }
    }
}

struct ModbusAdapter {
    addr: SocketAddr,
    declaration: WritableDeclaration,
    seq: u16,
    keys: BTreeMap<String, String>,
    heartbeat: u16,
}
impl ModbusAdapter {
    fn new(addr: SocketAddr) -> Self {
        Self {
            addr,
            declaration: WritableDeclaration {
                adapter_id: "modbus-tcp-plc".into(),
                version: "0.1.0".into(),
                source: format!("modbus://{addr}"),
                supports_idempotency: true,
                max_command_validity_ms: 10_000,
                dedup_window_ms: 60_000,
                supports_lease_epoch: true,
                verification_sources: vec!["plc-flow-register".into()],
                compatibility: Some("virtual Modbus slave; no hardware".into()),
            },
            seq: 0,
            keys: BTreeMap::new(),
            heartbeat: 0,
        }
    }
    fn request(&self, pdu: &[u8]) -> Result<Vec<u8>, ExecError> {
        let mut stream = TcpStream::connect_timeout(&self.addr, Duration::from_millis(500))
            .map_err(|error| ExecError::Disconnected(error.to_string()))?;
        stream.set_read_timeout(Some(Duration::from_millis(500))).ok();
        let len = (pdu.len() + 1) as u16;
        let mut frame = vec![0, 1, 0, 0];
        frame.extend_from_slice(&len.to_be_bytes());
        frame.push(1);
        frame.extend_from_slice(pdu);
        stream.write_all(&frame).map_err(|error| ExecError::Disconnected(error.to_string()))?;
        let mut header = [0u8; 7];
        stream.read_exact(&mut header).map_err(|error| ExecError::Disconnected(error.to_string()))?;
        let length = u16::from_be_bytes([header[4], header[5]]) as usize;
        let mut pdu = vec![0u8; length - 1];
        stream.read_exact(&mut pdu).map_err(|error| ExecError::Disconnected(error.to_string()))?;
        if pdu[0] & 0x80 != 0 {
            return Err(match pdu.get(1).copied().unwrap_or(4) {
                6 => ExecError::Rejected("slave device busy (0x06)".into()),
                4 => ExecError::Rejected("slave device failure (0x04)".into()),
                other => ExecError::Rejected(format!("modbus exception 0x{other:02x}")),
            });
        }
        Ok(pdu)
    }
    fn heartbeat(&mut self) -> Result<(), ExecError> {
        self.heartbeat = self.heartbeat.wrapping_add(1);
        let mut pdu = vec![0x10];
        pdu.extend_from_slice(&4u16.to_be_bytes());
        pdu.extend_from_slice(&1u16.to_be_bytes());
        pdu.push(2);
        pdu.extend_from_slice(&self.heartbeat.to_be_bytes());
        self.request(&pdu).map(|_| ())
    }
    fn read_registers(&self, start: u16, qty: u16) -> Result<Vec<u16>, ExecError> {
        let mut pdu = vec![0x03];
        pdu.extend_from_slice(&start.to_be_bytes());
        pdu.extend_from_slice(&qty.to_be_bytes());
        let response = self.request(&pdu)?;
        Ok(response[2..].chunks(2).map(|c| u16::from_be_bytes([c[0], c[1]])).collect())
    }
    fn set_exception(&self, code: u16) -> Result<(), ExecError> {
        let mut pdu = vec![0x10];
        pdu.extend_from_slice(&9u16.to_be_bytes());
        pdu.extend_from_slice(&1u16.to_be_bytes());
        pdu.push(2);
        pdu.extend_from_slice(&code.to_be_bytes());
        self.request(&pdu).map(|_| ())
    }
    fn write_sequence(&self, seq: u16) -> Result<(), ExecError> {
        let mut pdu = vec![0x10];
        pdu.extend_from_slice(&0u16.to_be_bytes());
        pdu.extend_from_slice(&4u16.to_be_bytes());
        pdu.push(8);
        for value in [1u16, seq, 1, 5_000] {
            pdu.extend_from_slice(&value.to_be_bytes());
        }
        self.request(&pdu).map(|_| ())
    }
    fn execution_count(&self) -> u32 {
        self.read_registers(11, 1).map(|r| u32::from(r[0])).unwrap_or(0)
    }
}
impl WritableAdapter for ModbusAdapter {
    fn name(&self) -> &str {
        &self.declaration.adapter_id
    }
    fn declaration(&self) -> &WritableDeclaration {
        &self.declaration
    }
    fn dispatch(&mut self, command: &Command) -> Result<Receipt, ExecError> {
        if let Some(original) = self.keys.get(&command.idempotency_key) {
            return Err(ExecError::Duplicate(original.clone()));
        }
        self.seq = self.seq.wrapping_add(1).max(1);
        let mut pdu = vec![0x10];
        pdu.extend_from_slice(&0u16.to_be_bytes());
        pdu.extend_from_slice(&4u16.to_be_bytes());
        pdu.push(8);
        for value in [1u16, self.seq, 1, 5_000] {
            pdu.extend_from_slice(&value.to_be_bytes());
        }
        self.request(&pdu)?;
        self.keys.insert(command.idempotency_key.clone(), command.action_id.clone());
        Ok(Receipt {
            action_id: command.action_id.clone(),
            accepted: true,
            executed_at_ms: command.valid_from_ms,
            detail: "FC16 accepted".into(),
        })
    }
}

fn action(id: &str, key: &str) -> Action {
    Action {
        action_id: id.into(),
        idempotency_key: key.into(),
        target: "plc".into(),
        command: "pump_on".into(),
        plan_ref: Some("shadow".into()),
        policy_version: "modbus/0.1".into(),
        precondition: Precondition { state_version: BTreeMap::from([("site-a".into(), 1)]), predicates: vec![] },
        valid_from: Timestamp(0),
        valid_until: Timestamp(10_000),
        budget: vec![],
        reservations: vec![],
        reversibility: Reversibility::Idempotent,
        verification_spec: VerificationSpec {
            observe: "plc-flow".into(),
            condition: "flow > 5".into(),
            window_ms: 5_000,
        },
        authorization: None,
        redispatch_of: None,
    }
}
fn gate_with_lease() -> ExecutionGate {
    let mut gate = ExecutionGate::new();
    gate.set_lease(Some(GateLease { epoch: 1, dispatch_deadline_ms: 20_000 }));
    gate
}

fn main() {
    let state = Arc::new(Mutex::new(PlcState::new()));
    let watchdog = Arc::clone(&state);
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(100));
        let mut plc = watchdog.lock().unwrap();
        if !plc.fault && plc.last_heartbeat.elapsed() > WATCHDOG {
            plc.valve_open = false;
            plc.pump_on = false;
            plc.fault = true;
        }
    });
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server = Arc::clone(&state);
    std::thread::spawn(move || {
        for stream in listener.incoming().flatten() {
            let state = Arc::clone(&server);
            std::thread::spawn(move || handle(stream, state));
        }
    });

    let mut adapter = ModbusAdapter::new(addr);
    let mut checks: Vec<(&str, bool, String)> = Vec::new();

    // M1: FC16 command, FC03 flow readback through gate.
    adapter.heartbeat().ok();
    let mut gate = gate_with_lease();
    let a1 = action("a1", "key-1");
    gate.register_authorized(&a1).unwrap();
    let outcome = gate.dispatch(&a1, &mut adapter, 100).unwrap();
    let flow = adapter.read_registers(6, 1).map(|r| r[0]).unwrap_or(0);
    checks.push((
        "M1",
        matches!(outcome, DispatchOutcome::Dispatched { .. }) && flow == 80,
        format!("outcome Dispatched={}; flow_x10={flow}", outcome.dispatched()),
    ));

    // M2: duplicate idempotency key and stale sequence ignored.
    let before = adapter.execution_count();
    let duplicate = adapter.dispatch(&Command {
        action_id: "a2".into(),
        idempotency_key: "key-1".into(),
        target: "plc".into(),
        command: "pump_on".into(),
        valid_from_ms: 0,
        valid_until_ms: 10_000,
        lease_epoch: Some(1),
    });
    let after_duplicate = adapter.execution_count();
    let stale = adapter.write_sequence(adapter.seq);
    let after_stale = adapter.execution_count();
    checks.push((
        "M2",
        matches!(duplicate, Err(ExecError::Duplicate(_)))
            && before == after_duplicate
            && stale.is_ok()
            && after_stale == before,
        format!(
            "duplicate={duplicate:?}; exec {}->{}; stale_seq_write={stale:?}; exec_after_stale={after_stale}",
            before, after_duplicate
        ),
    ));

    // M3: exception mapping 0x06 and 0x04.
    adapter.set_exception(6).unwrap();
    let mut gate3 = gate_with_lease();
    let a3 = action("a3", "key-3");
    gate3.register_authorized(&a3).unwrap();
    let busy = gate3.dispatch(&a3, &mut adapter, 100);
    let mut adapter4 = ModbusAdapter::new(addr);
    adapter4.heartbeat().ok();
    adapter4.set_exception(4).unwrap();
    let mut gate4 = gate_with_lease();
    let a4 = action("a4", "key-4");
    gate4.register_authorized(&a4).unwrap();
    let failure = gate4.dispatch(&a4, &mut adapter4, 100);
    let busy_reason = match &busy {
        Ok(DispatchOutcome::Rejected { reason }) => reason.clone(),
        _ => String::new(),
    };
    let failure_reason = match &failure {
        Ok(DispatchOutcome::Rejected { reason }) => reason.clone(),
        _ => String::new(),
    };
    checks.push((
        "M3",
        busy_reason.contains("busy") && failure_reason.contains("failure"),
        format!("busy_reason={busy_reason:?}; failure_reason={failure_reason:?}"),
    ));

    // M4: unreachable TCP -> Fallback.
    let dead: SocketAddr = "127.0.0.1:1".parse().unwrap();
    let mut dead_adapter = ModbusAdapter::new(dead);
    let mut gate5 = gate_with_lease();
    let a5 = action("a5", "key-5");
    gate5.register_authorized(&a5).unwrap();
    let fallback = gate5.dispatch(&a5, &mut dead_adapter, 100);
    checks.push((
        "M4",
        matches!(fallback, Ok(DispatchOutcome::Fallback { .. })),
        format!("fallback={:?}", fallback.map(|o| o.dispatched())),
    ));

    // M5: stop heartbeat -> watchdog closes; clear only after safe.
    let mut fault = 0;
    for _ in 0..50 {
        fault = adapter.read_registers(5, 1).map(|r| r[0]).unwrap_or(0);
        if fault == 1 {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let valve = adapter.read_registers(7, 1).map(|r| r[0]).unwrap_or(1);
    let flow = adapter.read_registers(6, 1).map(|r| r[0]).unwrap_or(1);
    let mut clear = vec![0x10];
    clear.extend_from_slice(&10u16.to_be_bytes());
    clear.extend_from_slice(&1u16.to_be_bytes());
    clear.push(2);
    clear.extend_from_slice(&1u16.to_be_bytes());
    let cleared = adapter.request(&clear).is_ok();
    let fault_after = adapter.read_registers(5, 1).map(|r| r[0]).unwrap_or(1);
    checks.push((
        "M5",
        fault == 1 && valve == 0 && flow == 0 && cleared && fault_after == 0,
        format!("fault={fault}; valve={valve}; flow={flow}; cleared={cleared}; fault_after={fault_after}"),
    ));

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
