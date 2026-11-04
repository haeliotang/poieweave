//! Governed agent runtime (B0): closed tools, citations, run persistence.
//!
//! A real OpenAI-compatible provider is called with the six-tool catalog. Read-
//! only tools execute now; `action.draft` creates a pending approval token and
//! stops the run (approval resume lands in B1).

use std::collections::{BTreeMap, BTreeSet};

use poie_governance::{
    subject_digest, tool, ApprovalToken, AuditRecord, Decision, EvidenceRef, Profile, RunState, RunStatus,
    ToolCallRecord, AUDIT_SCHEMA, POLICY_VERSION,
};
use poie_ontology::graph::{Goal, Query, DEFAULT_MAX_DEPTH};
use poie_ontology::model::Verification;
use poie_plan::plan::{plan, PlanningGoal, PlanningRequest};
use poie_plan::sap::{preview_plan_through_sap, AssumedContext};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::State;

use crate::auth;
use crate::inference;
use crate::AppState;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentResult {
    pub run_id: String,
    pub status: String,
    pub text: String,
    pub citations: Vec<EvidenceRef>,
    pub tools: Vec<String>,
    pub approval_token_id: Option<String>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
struct ProviderResponse {
    choices: Vec<ProviderChoice>,
}

#[derive(Deserialize)]
struct ProviderChoice {
    message: ProviderMessage,
}

#[derive(Deserialize, Default)]
struct ProviderMessage {
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    tool_calls: Vec<ProviderToolCall>,
}

#[derive(Deserialize)]
struct ProviderToolCall {
    id: String,
    function: ProviderFunction,
}

#[derive(Deserialize)]
struct ProviderFunction {
    name: String,
    #[serde(default)]
    arguments: String,
}

fn now_ms() -> i64 {
    auth::now_ms()
}

fn random_id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).map_err(|error| format!("random id: {error}"))?;
    Ok(hex::encode(bytes))
}

fn sha_json(value: &Value) -> String {
    poie_digest::sha256_of("poie.agent/0.1", value).unwrap_or_else(|_| "sha-error".to_string())
}

fn tools_json() -> Value {
    let schemas = [
        (
            "catalog.describe",
            "Return the Ontology Catalog",
            json!({"type":"object","additionalProperties":false,"properties":{}}),
        ),
        (
            "state.query",
            "Query objects, links and capability results",
            json!({"type":"object","additionalProperties":false,"required":["subject"],"properties":{"subject":{"type":"string"},"asOf":{"type":["string","null"]}}}),
        ),
        (
            "plan.propose",
            "Produce a proposed plan (no authorization)",
            json!({"type":"object","additionalProperties":false,"required":["goal","horizonS"],"properties":{"goal":{"type":"string"},"horizonS":{"type":"number"},"deadlineS":{"type":["number","null"]}}}),
        ),
        (
            "plan.preview_sap",
            "Read-only SAP dispatch preview",
            json!({"type":"object","additionalProperties":false,"required":["planFingerprint"],"properties":{"planFingerprint":{"type":"string"}}}),
        ),
        (
            "action.draft",
            "Draft an action from a plan; requires operator approval",
            json!({"type":"object","additionalProperties":false,"required":["planFingerprint","actionId"],"properties":{"planFingerprint":{"type":"string"},"actionId":{"type":"string"}}}),
        ),
        (
            "answer.emit",
            "Emit validated claims/citations as the final answer",
            json!({"type":"object","additionalProperties":false,"required":["claims"],"properties":{"claims":{"type":"array"},"suggestions":{"type":"array"}}}),
        ),
    ];
    Value::Array(
        schemas
            .iter()
            .map(|(name, description, parameters)| {
                json!({
                    "type":"function", "function": {"name": name, "description": description, "parameters": parameters}
                })
            })
            .collect(),
    )
}

fn profile_from(value: &str) -> Result<Profile, String> {
    match value {
        "analyst" => Ok(Profile::Analyst),
        "dispatcher" => Ok(Profile::Dispatcher),
        _ => Err(format!("unknown profile {value:?}")),
    }
}

fn object_ref(kind: &str, id: &str) -> EvidenceRef {
    EvidenceRef {
        kind: kind.to_string(),
        id: id.to_string(),
        fingerprint: Some(format!("{kind}:{id}")),
        sha256: None,
        as_of: None,
        source: "model".into(),
    }
}

fn protocol_error(code: &str, detail: impl Into<String>) -> String {
    format!("{code}: {}", detail.into())
}

pub async fn run(prompt: String, profile: String, state: State<'_, AppState>) -> Result<AgentResult, String> {
    let operator = auth::require(&state.session)?;
    let profile = profile_from(&profile)?;
    let prompt = prompt.trim();
    if prompt.is_empty() || prompt.len() > 4000 {
        return Err("invalid_parameter: prompt must be 1–4000 characters".into());
    }
    let (endpoint, model, key) = inference::provider_config(
        &*state.db.lock().map_err(|_| "database lock poisoned".to_string())?,
        state.secrets.as_ref(),
    )?
    .ok_or_else(|| "remote inference is not configured".to_string())?;
    let url = inference::completion_url(&endpoint);
    let run_id = random_id()?;
    let mut run = RunState::new(run_id.clone(), operator.session_id.clone(), profile, now_ms());
    run.status = RunStatus::Running;
    let mut messages: Vec<Value> = vec![
        json!({"role":"system","content":"You are Poieweave's governed industrial agent. Use only the provided tools. Do not claim facts without citations. End with answer.emit; suggestions are non-factual. Never dispatch or authorize actions."}),
        json!({"role":"user","content":prompt}),
    ];
    let mut citations: Vec<EvidenceRef> = Vec::new();
    let mut tools_used = Vec::new();
    let mut plan_opt: Option<poie_plan::plan::Plan> = None;
    let mut calls: BTreeMap<String, u32> = BTreeMap::new();
    let mut final_text = String::new();
    let mut error: Option<String> = None;

    for step in 0..8u32 {
        run.budget.steps_used = step + 1;
        let body = json!({
            "model": model,
            "messages": messages,
            "tools": tools_json(),
            "tool_choice": "auto",
            "temperature": 0,
            "max_tokens": 1024
        });
        let response = state.http.post_json(&url, &key, &body.to_string())?;
        if !(200..300).contains(&response.status) {
            error = Some(protocol_error("provider_protocol", format!("HTTP {}", response.status)));
            break;
        }
        if response.body.len() > 262_144 {
            error = Some(protocol_error("provider_response_too_large", "response over 256 KiB"));
            break;
        }
        let parsed: ProviderResponse = serde_json::from_str(&response.body)
            .map_err(|error| protocol_error("provider_protocol", format!("invalid JSON: {error}")))?;
        let message = parsed
            .choices
            .first()
            .map(|choice| &choice.message)
            .ok_or_else(|| protocol_error("provider_protocol", "missing choices"))?;
        if message.tool_calls.is_empty() {
            if let Some(content) = &message.content {
                final_text = content.clone();
            }
            error = Some(protocol_error("citation_missing", "answer.emit was not called"));
            break;
        }
        let mut assistant_tool_calls = Vec::new();
        for call in &message.tool_calls {
            let name = call.function.name.clone();
            let spec =
                tool(&name).ok_or_else(|| protocol_error("invalid_tool_arguments", format!("unknown tool {name}")))?;
            if !profile.allows(&name) {
                error = Some(protocol_error("policy_blocked", format!("profile does not allow {name}")));
                break;
            }
            let arguments: Value = serde_json::from_str(if call.function.arguments.trim().is_empty() {
                "{}"
            } else {
                &call.function.arguments
            })
            .map_err(|error| protocol_error("invalid_tool_arguments", error.to_string()))?;
            let count = calls.entry(name.clone()).or_insert(0);
            *count += 1;
            if *count > spec.max_calls_per_run {
                error = Some(protocol_error("budget_exceeded", format!("{name} called too often")));
                break;
            }
            assistant_tool_calls.push(json!({"id": call.id, "type":"function", "function": {"name": name, "arguments": call.function.arguments}}));
            let (outcome, call_citations) =
                execute_tool(&name, &arguments, &state, &mut plan_opt, &run_id, &operator.session_id, &mut run)?;
            citations.extend(call_citations.iter().cloned());
            tools_used.push(name.clone());
            let record = ToolCallRecord {
                id: call.id.clone(),
                tool: name.clone(),
                tool_version: spec.version.to_string(),
                args_sha256: sha_json(&arguments),
                args_json_redacted: arguments.to_string(),
                result_sha256: sha_json(&outcome),
                result_summary: outcome.to_string().chars().take(160).collect(),
                citations: call_citations.clone(),
                evidence_refs: call_citations,
                truncated: false,
                decision: Decision::Allowed,
            };
            run.turns.push(record);
            if name == "answer.emit" {
                final_text = outcome.pointer("/text").and_then(|value| value.as_str()).unwrap_or_default().to_string();
            }
            if name == "action.draft" {
                run.status = RunStatus::WaitingApproval;
                run.pending_approval =
                    outcome.pointer("/pendingApproval").cloned().and_then(|value| serde_json::from_value(value).ok());
                run.citations = citations.clone();
                run.updated_at_ms = now_ms();
                save_run(&state, &run)?;
                return Ok(AgentResult {
                    run_id,
                    status: "waiting_approval".into(),
                    text: final_text,
                    citations,
                    tools: tools_used,
                    approval_token_id: run.pending_approval.as_ref().map(|pending| pending.token_id.clone()),
                    error: None,
                });
            }
            messages.push(json!({"role":"tool","tool_call_id": call.id, "content": outcome.to_string()}));
        }
        if error.is_some() {
            break;
        }
        messages.push(json!({"role":"assistant","content": message.content, "tool_calls": assistant_tool_calls}));
        if final_text.is_empty() && message.content.is_some() && message.tool_calls.is_empty() {
            final_text = message.content.clone().unwrap_or_default();
        }
        if run.turns.last().map(|record| record.tool == "answer.emit").unwrap_or(false) {
            break;
        }
    }

    if error.is_none() && final_text.is_empty() {
        error = Some(protocol_error("citation_missing", "no final answer"));
    }
    run.status = if error.is_some() { RunStatus::Failed } else { RunStatus::Completed };
    run.citations = citations.clone();
    run.updated_at_ms = now_ms();
    run.error = error.clone();
    save_run(&state, &run)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    db.append_audit(now_ms(), &operator.display_name, "agent.run", &format!("{run_id} {:?} {:?}", run.status, error))?;
    append_agent_audit(
        &state,
        &run_id,
        &run.session_id,
        if error.is_some() { Decision::Failed } else { Decision::Completed },
        &format!("{:?}", run.status),
    )?;
    Ok(AgentResult {
        run_id,
        status: if error.is_some() { "failed".into() } else { "completed".into() },
        text: final_text,
        citations,
        tools: tools_used,
        approval_token_id: None,
        error,
    })
}

fn append_agent_audit(
    state: &State<'_, AppState>,
    run_id: &str,
    session_id: &str,
    decision: Decision,
    reason: &str,
) -> Result<(), String> {
    use std::io::{BufRead, Write};
    let path = state.data_dir.join("agent-audit.log");
    let prev = if path.exists() {
        std::io::BufReader::new(std::fs::File::open(&path).map_err(|error| format!("audit open: {error}"))?)
            .lines()
            .map_while(Result::ok)
            .filter(|line| !line.trim().is_empty())
            .last()
            .and_then(|line| serde_json::from_str::<AuditRecord>(&line).ok())
            .map(|record| record.hash)
    } else {
        None
    };
    let record = AuditRecord {
        schema: AUDIT_SCHEMA.into(),
        event_id: random_id()?,
        at_ms: now_ms(),
        actor: "agent".into(),
        session_id: session_id.into(),
        run_id: run_id.into(),
        turn_id: 0,
        call_id: None,
        tool: None,
        tool_version: None,
        decision,
        reason: reason.into(),
        args_sha256: None,
        result_sha256: None,
        sap_record_id: None,
        prev_hash: prev.clone(),
        hash: String::new(),
    }
    .seal(prev)
    .map_err(|error| error.to_string())?;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|error| format!("audit open: {error}"))?;
    writeln!(file, "{}", serde_json::to_string(&record).map_err(|error| error.to_string())?)
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("audit write: {error}"))?;
    Ok(())
}

fn save_run(state: &State<'_, AppState>, run: &RunState) -> Result<(), String> {
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let json = serde_json::to_string(run).map_err(|error| error.to_string())?;
    db.save_agent_run(
        &run.run_id,
        &run.schema_version,
        &run.session_id,
        &json,
        run.created_at_ms,
        run.updated_at_ms,
        run.created_at_ms + 3_600_000,
    )
}

fn execute_tool(
    name: &str,
    arguments: &Value,
    state: &State<'_, AppState>,
    plan_opt: &mut Option<poie_plan::plan::Plan>,
    run_id: &str,
    session_id: &str,
    _run: &mut RunState,
) -> Result<(Value, Vec<EvidenceRef>), String> {
    match name {
        "catalog.describe" => {
            Ok((serde_json::to_value(&state.catalog).map_err(|error| error.to_string())?, Vec::new()))
        }
        "state.query" => {
            let subject = arguments
                .get("subject")
                .and_then(Value::as_str)
                .ok_or_else(|| "invalid_tool_arguments: subject".to_string())?;
            let (kind, id) =
                subject.split_once(':').ok_or_else(|| "invalid_tool_arguments: subject must be type:id".to_string())?;
            let objects = match kind {
                "asset" => state
                    .domain
                    .assets
                    .iter()
                    .find(|item| item.id == id)
                    .map(|item| json!({"id": item.id, "kind": item.kind, "status": item.status})),
                "resource" => state
                    .domain
                    .resources
                    .iter()
                    .find(|item| item.id == id)
                    .map(|item| json!({"id": item.id, "status": item.status})),
                "process" => state
                    .domain
                    .processes
                    .iter()
                    .find(|item| item.id == id)
                    .map(|item| json!({"id": item.id, "status": item.status})),
                "service" => state
                    .domain
                    .services
                    .iter()
                    .find(|item| item.id == id)
                    .map(|item| json!({"id": item.id, "verdict": item.verdict})),
                _ => None,
            };
            if objects.is_none() {
                return Err(format!("unknown_object: {subject}"));
            }
            Ok((json!({"objects":[objects], "links":[], "capabilities":[]}), vec![object_ref(kind, id)]))
        }
        "plan.propose" => {
            let goal = arguments
                .get("goal")
                .and_then(Value::as_str)
                .ok_or_else(|| "invalid_tool_arguments: goal".to_string())?;
            let horizon = arguments
                .get("horizonS")
                .and_then(Value::as_f64)
                .ok_or_else(|| "invalid_tool_arguments: horizonS".to_string())?;
            let deadline = arguments.get("deadlineS").and_then(Value::as_f64);
            let query = Query {
                goal: Goal::Service(goal.to_string()),
                min_verification: Verification::Demonstrated,
                max_depth: DEFAULT_MAX_DEPTH,
            };
            let _ = poie_ontology::graph::solve(&state.model, &query);
            let planned = plan(&PlanningRequest {
                model: &state.model,
                now_s: 0.0,
                horizon_s: horizon,
                goals: vec![PlanningGoal { goal: Goal::Service(goal.to_string()), deadline_s: deadline, priority: 10 }],
                min_verification: Verification::Demonstrated,
                max_actions: 12,
            })
            .map_err(|error| format!("plan_failed: {error}"))?;
            let fingerprint = planned.digest();
            *plan_opt = Some(planned.clone());
            Ok((
                json!({"planFingerprint": fingerprint, "actions": planned.actions.iter().map(|action| action.action_id.clone()).collect::<Vec<_>>()}),
                vec![EvidenceRef {
                    kind: "plan".into(),
                    id: fingerprint.clone(),
                    fingerprint: Some(fingerprint),
                    sha256: None,
                    as_of: None,
                    source: "model".into(),
                }],
            ))
        }
        "plan.preview_sap" => {
            let plan = plan_opt.as_ref().ok_or_else(|| "preview_unavailable: plan.propose first".to_string())?;
            let assumed = AssumedContext {
                now_ms: 0,
                holds_command_ownership: true,
                reservations_held: true,
                predicates_holding: plan
                    .actions
                    .iter()
                    .flat_map(|action| action.preconditions.clone())
                    .collect::<BTreeSet<_>>(),
                state_version: BTreeMap::from([("site-a".to_string(), 4)]),
            };
            let preview =
                preview_plan_through_sap(plan, &assumed).map_err(|error| format!("preview_failed: {error:?}"))?;
            Ok((serde_json::to_value(preview).map_err(|error| error.to_string())?, vec![]))
        }
        "action.draft" => {
            let plan = plan_opt.as_ref().ok_or_else(|| "preview_unavailable: plan.propose first".to_string())?;
            let action_id = arguments
                .get("actionId")
                .and_then(Value::as_str)
                .ok_or_else(|| "invalid_tool_arguments: actionId".to_string())?;
            let action = plan
                .actions
                .iter()
                .find(|action| action.action_id == action_id)
                .ok_or_else(|| "unknown_object: action".to_string())?;
            let token_id = random_id()?;
            let expires = now_ms() + 300_000;
            let digest = subject_digest(name, arguments, session_id, POLICY_VERSION, expires)
                .map_err(|error| error.to_string())?;
            let token = ApprovalToken {
                id: token_id.clone(),
                session_id: session_id.to_string(),
                run_id: run_id.to_string(),
                call_id: String::new(),
                tool: name.to_string(),
                tool_version: "1.0.0".into(),
                subject_digest: digest.clone(),
                policy_version: POLICY_VERSION.into(),
                args_json_redacted: arguments.to_string(),
                impact_json: json!({"action": action.action_id, "process": action.process, "target": action.target})
                    .to_string(),
                citations_json: "[]".into(),
                state: "pending".into(),
                created_at_ms: now_ms(),
                decided_at_ms: None,
                started_at_ms: None,
                expires_at_ms: expires,
            };
            let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
            db.save_approval_token(&token)?;
            let pending = json!({"tokenId": token_id, "tool": name, "subjectDigest": digest.clone(), "argsJsonRedacted": arguments.to_string(), "impact": token.impact_json, "citations": [], "expiresAt": expires});
            Ok((json!({"pendingApproval": pending}), vec![]))
        }
        "answer.emit" => {
            let claims = arguments
                .get("claims")
                .and_then(Value::as_array)
                .ok_or_else(|| "invalid_tool_arguments: claims".to_string())?;
            let mut text = Vec::new();
            for claim in claims {
                let claim_text = claim.get("text").and_then(Value::as_str).unwrap_or_default();
                let cites = claim.get("citations").and_then(Value::as_array).cloned().unwrap_or_default();
                if cites.is_empty() {
                    return Err("citation_missing: claim without citation".into());
                }
                text.push(claim_text.to_string());
            }
            Ok((json!({"accepted": true, "text": text.join("\n")}), vec![]))
        }
        _ => Err(format!("not_implemented: {name}")),
    }
}

#[tauri::command]
pub async fn agent_run(prompt: String, profile: String, state: State<'_, AppState>) -> Result<AgentResult, String> {
    run(prompt, profile, state).await
}

#[tauri::command]
pub fn agent_status(run_id: String, state: State<'_, AppState>) -> Result<RunState, String> {
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let json = db.agent_run(&run_id)?.ok_or_else(|| format!("run_not_found: {run_id}"))?;
    serde_json::from_str(&json).map_err(|error| format!("run state corrupt: {error}"))
}

#[tauri::command]
pub fn agent_approve(run_id: String, token_id: String, state: State<'_, AppState>) -> Result<RunState, String> {
    let operator = auth::require(&state.session)?;
    let token = {
        let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
        let token = db.approval_token(&token_id)?.ok_or_else(|| format!("approval_required: {token_id}"))?;
        if token.session_id != operator.session_id || token.run_id != run_id {
            return Err("approval_mismatch: token does not belong to this session/run".into());
        }
        if !db.transition_approval(&token_id, "pending", "approved", now_ms())? {
            return Err("approval_mismatch: token is not pending".into());
        }
        token
    };
    execute_approved(&state, &run_id, &token)
}

#[tauri::command]
pub fn agent_resume(run_id: String, token_id: String, state: State<'_, AppState>) -> Result<RunState, String> {
    let operator = auth::require(&state.session)?;
    let token = {
        let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
        db.approval_token(&token_id)?.ok_or_else(|| format!("approval_required: {token_id}"))?
    };
    if token.session_id != operator.session_id || token.run_id != run_id {
        return Err("approval_mismatch".into());
    }
    execute_approved(&state, &run_id, &token)
}

#[tauri::command]
pub fn agent_reject(run_id: String, token_id: String, state: State<'_, AppState>) -> Result<RunState, String> {
    let operator = auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let token = db.approval_token(&token_id)?.ok_or_else(|| format!("approval_required: {token_id}"))?;
    if token.session_id != operator.session_id || token.run_id != run_id {
        return Err("approval_mismatch".into());
    }
    if !db.transition_approval(&token_id, "pending", "rejected", now_ms())? {
        return Err("approval_mismatch: token is not pending".into());
    }
    let mut run = load_run(&db, &run_id)?;
    run.status = RunStatus::Failed;
    run.pending_approval = None;
    run.error = Some("approval_rejected".into());
    run.updated_at_ms = now_ms();
    let json = serde_json::to_string(&run).map_err(|error| error.to_string())?;
    db.save_agent_run(
        &run.run_id,
        &run.schema_version,
        &run.session_id,
        &json,
        run.created_at_ms,
        run.updated_at_ms,
        run.created_at_ms + 3_600_000,
    )?;
    db.append_audit(now_ms(), &operator.display_name, "agent.rejected", &format!("{run_id} {token_id}"))?;
    append_agent_audit(&state, &run_id, &run.session_id, Decision::Rejected, &token_id)?;
    Ok(run)
}

#[tauri::command]
pub fn agent_cancel(run_id: String, state: State<'_, AppState>) -> Result<RunState, String> {
    let operator = auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    let mut run = load_run(&db, &run_id)?;
    if let Some(pending) = &run.pending_approval {
        let _ = db.transition_approval(&pending.token_id, "pending", "cancelled", now_ms());
    }
    run.status = RunStatus::Cancelled;
    run.pending_approval = None;
    run.error = Some("cancelled".into());
    run.updated_at_ms = now_ms();
    let json = serde_json::to_string(&run).map_err(|error| error.to_string())?;
    db.save_agent_run(
        &run.run_id,
        &run.schema_version,
        &run.session_id,
        &json,
        run.created_at_ms,
        run.updated_at_ms,
        run.created_at_ms + 3_600_000,
    )?;
    db.append_audit(now_ms(), &operator.display_name, "agent.cancel", &run_id)?;
    append_agent_audit(&state, &run_id, &run.session_id, Decision::Cancelled, &run_id)?;
    Ok(run)
}

fn load_run(db: &crate::db::Db, run_id: &str) -> Result<RunState, String> {
    let json = db.agent_run(run_id)?.ok_or_else(|| format!("run_not_found: {run_id}"))?;
    serde_json::from_str(&json).map_err(|error| format!("run state corrupt: {error}"))
}

fn execute_approved(state: &State<'_, AppState>, run_id: &str, token: &ApprovalToken) -> Result<RunState, String> {
    let operator = auth::require(&state.session)?;
    let db = state.db.lock().map_err(|_| "database lock poisoned".to_string())?;
    if token.session_id != operator.session_id || token.run_id != run_id {
        return Err("approval_mismatch".into());
    }
    if !db.transition_approval(&token.id, "approved", "executing", now_ms())? {
        return Err("approval_mismatch: token is not approved".into());
    }
    let mut run = load_run(&db, run_id)?;
    let impact: Value = serde_json::from_str(&token.impact_json).unwrap_or_else(|_| json!({"action": "draft"}));
    let text = format!(
        "Approved action draft: {} → {} (no dispatch)",
        impact.get("action").and_then(Value::as_str).unwrap_or("draft"),
        impact.get("target").and_then(Value::as_str).unwrap_or("target")
    );
    let citations: Vec<EvidenceRef> = serde_json::from_str(&token.citations_json).unwrap_or_default();
    run.turns.push(ToolCallRecord {
        id: token.call_id.clone(),
        tool: token.tool.clone(),
        tool_version: token.tool_version.clone(),
        args_sha256: poie_digest::sha256_of("poie.agent/0.1", &json!({"args": token.args_json_redacted}))
            .unwrap_or_default(),
        args_json_redacted: token.args_json_redacted.clone(),
        result_sha256: poie_digest::sha256_of("poie.agent/0.1", &json!({"text": text})).unwrap_or_default(),
        result_summary: text.clone(),
        citations: citations.clone(),
        evidence_refs: citations.clone(),
        truncated: false,
        decision: Decision::Approved,
    });
    run.status = RunStatus::Completed;
    run.pending_approval = None;
    run.citations.extend(citations);
    run.updated_at_ms = now_ms();
    db.transition_approval(&token.id, "executing", "consumed", now_ms())?;
    let json = serde_json::to_string(&run).map_err(|error| error.to_string())?;
    db.save_agent_run(
        &run.run_id,
        &run.schema_version,
        &run.session_id,
        &json,
        run.created_at_ms,
        run.updated_at_ms,
        run.created_at_ms + 3_600_000,
    )?;
    db.append_audit(
        now_ms(),
        &operator.display_name,
        "agent.approved",
        &format!("{} {} {}", run.run_id, token.tool, token.id),
    )?;
    Ok(run)
}
