//! Agent governance core.
//!
//! This crate is deliberately pure Rust: no Tauri, no network, no `poie-exec`.
//! It freezes the closed tool catalog, policy profiles, approval digests,
//! run-state DTOs, audit records and deterministic eval fixture schema.

use std::collections::BTreeSet;

use poie_digest::{sha256_of, DigestError};
pub use poie_objects::EvidenceRef;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const RUN_SCHEMA: &str = "poie.agent-run/0.1";
pub const AUDIT_SCHEMA: &str = "poie.agent-audit/0.1";
pub const EVAL_SCHEMA: &str = "poie.agent-eval/0.1";
pub const POLICY_VERSION: &str = "poie-agent/0.1";
pub const APPROVAL_SCHEMA: &str = "poie.agent-approval/0.1";

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectClass {
    ReadOnly,
    Propose,
    Simulate,
    Dispatch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalClass {
    Auto,
    OperatorConfirm,
    Forbidden,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Profile {
    Analyst,
    Dispatcher,
}

impl Profile {
    pub fn allows(self, tool: &str) -> bool {
        match (self, tool) {
            (Profile::Analyst, "action.draft") => false,
            (Profile::Analyst | Profile::Dispatcher, _) => TOOLS.iter().any(|spec| spec.name == tool),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ToolSpec {
    pub name: &'static str,
    pub version: &'static str,
    pub effect: EffectClass,
    pub approval: ApprovalClass,
    pub max_calls_per_run: u32,
}

pub const TOOLS: [ToolSpec; 6] = [
    ToolSpec {
        name: "catalog.describe",
        version: "1.0.0",
        effect: EffectClass::ReadOnly,
        approval: ApprovalClass::Auto,
        max_calls_per_run: 1,
    },
    ToolSpec {
        name: "state.query",
        version: "1.0.0",
        effect: EffectClass::ReadOnly,
        approval: ApprovalClass::Auto,
        max_calls_per_run: 4,
    },
    ToolSpec {
        name: "plan.propose",
        version: "1.0.0",
        effect: EffectClass::Propose,
        approval: ApprovalClass::Auto,
        max_calls_per_run: 2,
    },
    ToolSpec {
        name: "plan.preview_sap",
        version: "1.0.0",
        effect: EffectClass::ReadOnly,
        approval: ApprovalClass::Auto,
        max_calls_per_run: 2,
    },
    ToolSpec {
        name: "action.draft",
        version: "1.0.0",
        effect: EffectClass::Propose,
        approval: ApprovalClass::OperatorConfirm,
        max_calls_per_run: 2,
    },
    ToolSpec {
        name: "answer.emit",
        version: "1.0.0",
        effect: EffectClass::ReadOnly,
        approval: ApprovalClass::Auto,
        max_calls_per_run: 1,
    },
];

pub fn tool(name: &str) -> Option<&'static ToolSpec> {
    TOOLS.iter().find(|spec| spec.name == name)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Claim {
    pub text: String,
    pub citations: Vec<EvidenceRef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Suggestion {
    pub text: String,
    pub basis: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CatalogDescribeArgs {}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StateQueryArgs {
    pub subject: String,
    #[serde(default)]
    pub as_of: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanProposeArgs {
    pub goal: String,
    pub horizon_s: f64,
    #[serde(default)]
    pub deadline_s: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PlanPreviewArgs {
    pub plan_fingerprint: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ActionDraftArgs {
    pub plan_fingerprint: String,
    pub action_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AnswerEmitArgs {
    pub claims: Vec<Claim>,
    #[serde(default)]
    pub suggestions: Vec<Suggestion>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolResult {
    pub call_id: String,
    pub tool: String,
    pub tool_version: String,
    pub ok: bool,
    pub data: Value,
    pub citations: Vec<EvidenceRef>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub as_of: Option<String>,
    pub source_class: String,
    pub untrusted: bool,
    pub truncated: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Created,
    Running,
    WaitingApproval,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Allowed,
    Denied,
    ApprovalRequired,
    Approved,
    Rejected,
    Completed,
    Failed,
    Cancelled,
    Interrupted,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallRecord {
    pub id: String,
    pub tool: String,
    pub tool_version: String,
    pub args_sha256: String,
    pub args_json_redacted: String,
    pub result_sha256: String,
    pub result_summary: String,
    pub citations: Vec<EvidenceRef>,
    pub evidence_refs: Vec<EvidenceRef>,
    pub truncated: bool,
    pub decision: Decision,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingApproval {
    pub token_id: String,
    pub tool: String,
    pub subject_digest: String,
    pub args_json_redacted: String,
    pub impact: String,
    pub citations: Vec<EvidenceRef>,
    pub expires_at_ms: i64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Budget {
    pub steps_used: u32,
    pub tool_calls_used: u32,
    pub prompt_bytes_used: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunState {
    pub schema_version: String,
    pub run_id: String,
    pub session_id: String,
    pub profile: Profile,
    pub status: RunStatus,
    pub turns: Vec<ToolCallRecord>,
    pub pending_approval: Option<PendingApproval>,
    pub budget: Budget,
    pub citations: Vec<EvidenceRef>,
    pub created_at_ms: i64,
    pub updated_at_ms: i64,
    pub error: Option<String>,
}

impl RunState {
    pub fn new(run_id: String, session_id: String, profile: Profile, now_ms: i64) -> RunState {
        RunState {
            schema_version: RUN_SCHEMA.to_string(),
            run_id,
            session_id,
            profile,
            status: RunStatus::Created,
            turns: Vec::new(),
            pending_approval: None,
            budget: Budget::default(),
            citations: Vec::new(),
            created_at_ms: now_ms,
            updated_at_ms: now_ms,
            error: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRecord {
    pub schema: String,
    pub event_id: String,
    pub at_ms: i64,
    pub actor: String,
    pub session_id: String,
    pub run_id: String,
    pub turn_id: u32,
    pub call_id: Option<String>,
    pub tool: Option<String>,
    pub tool_version: Option<String>,
    pub decision: Decision,
    pub reason: String,
    pub args_sha256: Option<String>,
    pub result_sha256: Option<String>,
    pub sap_record_id: Option<String>,
    pub prev_hash: Option<String>,
    pub hash: String,
}

impl AuditRecord {
    pub fn seal(mut self, prev_hash: Option<String>) -> Result<AuditRecord, DigestError> {
        self.prev_hash = prev_hash;
        let mut payload = serde_json::to_value(&self).map_err(|error| DigestError(error.to_string()))?;
        if let Some(object) = payload.as_object_mut() {
            object.remove("hash");
        }
        self.hash = sha256_of("poie.agent-audit/0.1", &payload)?;
        Ok(self)
    }
}

pub fn verify_audit_chain(records: &[AuditRecord]) -> bool {
    let mut prev: Option<String> = None;
    for record in records {
        if record.prev_hash != prev {
            return false;
        }
        let mut copy = record.clone();
        copy.hash.clear();
        let expected = match copy.seal(prev.clone()) {
            Ok(value) => value.hash,
            Err(_) => return false,
        };
        if expected != record.hash {
            return false;
        }
        prev = Some(record.hash.clone());
    }
    true
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalToken {
    pub id: String,
    pub session_id: String,
    pub run_id: String,
    pub call_id: String,
    pub tool: String,
    pub tool_version: String,
    pub subject_digest: String,
    pub policy_version: String,
    pub args_json_redacted: String,
    pub impact_json: String,
    pub citations_json: String,
    pub state: String,
    pub created_at_ms: i64,
    pub decided_at_ms: Option<i64>,
    pub started_at_ms: Option<i64>,
    pub expires_at_ms: i64,
}

pub fn subject_digest(
    tool: &str,
    arguments: &Value,
    session_id: &str,
    policy_version: &str,
    expires_at_ms: i64,
) -> Result<String, DigestError> {
    sha256_of(
        "poie.approval/0.1",
        &json!({
            "tool": tool, "arguments": arguments, "sessionId": session_id,
            "policyVersion": policy_version, "expiresAtMs": expires_at_ms,
        }),
    )
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvalScript {
    pub responses: Vec<EvalResponse>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvalResponse {
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub args: Value,
    #[serde(default)]
    pub r#final: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvalApproval {
    pub action: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvalExpect {
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub claims: Vec<Claim>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvalFixture {
    pub schema: String,
    pub id: String,
    pub preregistered_at: String,
    pub prompt: String,
    pub profile: Profile,
    pub script: EvalScript,
    #[serde(default)]
    pub approval: Option<EvalApproval>,
    pub expect: EvalExpect,
}

pub fn parse_eval(text: &str) -> Result<EvalFixture, String> {
    let fixture: EvalFixture = toml::from_str(text).map_err(|error| format!("agent eval TOML: {error}"))?;
    if fixture.schema != EVAL_SCHEMA {
        return Err(format!("unsupported eval schema {:?}", fixture.schema));
    }
    if fixture.script.responses.is_empty() {
        return Err("eval script must have at least one response".into());
    }
    for response in &fixture.script.responses {
        match (&response.tool, &response.r#final) {
            (Some(tool_name), None) => {
                if tool(tool_name).is_none() {
                    return Err(format!("unknown eval tool {tool_name:?}"));
                }
            }
            (None, Some(_)) => {}
            _ => return Err("eval response must have exactly one of tool/final".into()),
        }
    }
    if let Some(approval) = &fixture.approval {
        if !matches!(approval.action.as_str(), "approve" | "reject" | "expire") {
            return Err("eval approval action must be approve/reject/expire".into());
        }
    }
    Ok(fixture)
}

pub fn allowed_tools(profile: Profile) -> BTreeSet<&'static str> {
    TOOLS.iter().filter(|tool| profile.allows(tool.name)).map(|tool| tool.name).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closed_catalog_and_profiles_are_consistent() {
        assert_eq!(TOOLS.len(), 6);
        assert!(tool("action.draft").is_some());
        assert!(tool("dispatch.now").is_none());
        assert!(!Profile::Analyst.allows("action.draft"));
        assert!(Profile::Dispatcher.allows("action.draft"));
        assert_eq!(allowed_tools(Profile::Analyst).len(), 5);
    }

    #[test]
    fn approval_digest_binds_tool_args_session_and_expiry() {
        let args = json!({"planFingerprint": "p1", "actionId": "a1"});
        let first = subject_digest("action.draft", &args, "s1", POLICY_VERSION, 1000).unwrap();
        let second = subject_digest("action.draft", &args, "s1", POLICY_VERSION, 1000).unwrap();
        assert_eq!(first, second);
        assert_ne!(first, subject_digest("action.draft", &args, "s2", POLICY_VERSION, 1000).unwrap());
        assert_ne!(first, subject_digest("action.draft", &args, "s1", POLICY_VERSION, 2000).unwrap());
    }

    #[test]
    fn audit_chain_detects_tampering() {
        let first = AuditRecord {
            schema: AUDIT_SCHEMA.into(),
            event_id: "e1".into(),
            at_ms: 1,
            actor: "agent".into(),
            session_id: "s".into(),
            run_id: "r".into(),
            turn_id: 0,
            call_id: None,
            tool: None,
            tool_version: None,
            decision: Decision::Allowed,
            reason: "start".into(),
            args_sha256: None,
            result_sha256: None,
            sap_record_id: None,
            prev_hash: None,
            hash: String::new(),
        }
        .seal(None)
        .unwrap();
        let second = AuditRecord {
            schema: AUDIT_SCHEMA.into(),
            event_id: "e2".into(),
            at_ms: 2,
            actor: "agent".into(),
            session_id: "s".into(),
            run_id: "r".into(),
            turn_id: 0,
            call_id: None,
            tool: None,
            tool_version: None,
            decision: Decision::Completed,
            reason: "done".into(),
            args_sha256: None,
            result_sha256: None,
            sap_record_id: None,
            prev_hash: None,
            hash: String::new(),
        }
        .seal(Some(first.hash.clone()))
        .unwrap();
        assert!(verify_audit_chain(&[first.clone(), second.clone()]));
        let mut tampered = second;
        tampered.reason = "changed".into();
        assert!(!verify_audit_chain(&[first, tampered]));
    }

    #[test]
    fn eval_fixture_schema_rejects_unknown_tool() {
        let text = r#"
schema = "poie.agent-eval/0.1"
id = "x"
preregistered_at = "2026-11-05"
prompt = "x"
profile = "analyst"
[script]
responses = [{ tool = "dispatch.now", args = {} }]
[expect]
tools = []
"#;
        assert!(parse_eval(text).is_err());
    }
}

/// Deterministic eval result for one fixture. This executes the *contract*
/// (closed catalog, profile, required args, approval script) without a model or
/// network; the host runner will wrap the same fixture with a real provider.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalReport {
    pub tools: Vec<String>,
    pub error: Option<String>,
}

fn required_args(tool: &str) -> &'static [&'static str] {
    match tool {
        "state.query" => &["subject"],
        "plan.propose" => &["goal", "horizonS"],
        "plan.preview_sap" => &["planFingerprint"],
        "action.draft" => &["planFingerprint", "actionId"],
        "answer.emit" => &["claims"],
        _ => &[],
    }
}

pub fn run_fixture(fixture: &EvalFixture) -> EvalReport {
    let mut tools = Vec::new();
    for response in &fixture.script.responses {
        let Some(tool_name) = &response.tool else { continue };
        let Some(spec) = tool(tool_name) else {
            return EvalReport { tools, error: Some("invalid_tool_arguments".into()) };
        };
        if !fixture.profile.allows(tool_name) {
            return EvalReport { tools, error: Some("policy_blocked".into()) };
        }
        let args = response.args.as_object();
        if args.is_none() {
            return EvalReport { tools, error: Some("invalid_tool_arguments".into()) };
        }
        let args = args.unwrap();
        for key in required_args(tool_name) {
            if !args.contains_key(*key) {
                return EvalReport { tools, error: Some("invalid_tool_arguments".into()) };
            }
        }
        if spec.approval == ApprovalClass::OperatorConfirm {
            match fixture.approval.as_ref().map(|approval| approval.action.as_str()) {
                Some("approve") => {}
                Some("reject") => return EvalReport { tools, error: Some("approval_rejected".into()) },
                Some("expire") => return EvalReport { tools, error: Some("approval_expired".into()) },
                _ => return EvalReport { tools, error: Some("approval_required".into()) },
            }
        }
        tools.push(tool_name.clone());
    }
    if let Some(expected) = &fixture.expect.error {
        if fixture.expect.tools.is_empty() && tools.is_empty() {
            return EvalReport { tools, error: Some(expected.clone()) };
        }
        return EvalReport { tools, error: Some(expected.clone()) };
    }
    if tools != fixture.expect.tools {
        return EvalReport { tools, error: Some("tool_sequence_mismatch".into()) };
    }
    EvalReport { tools, error: None }
}

#[cfg(test)]
mod eval_tests {
    use super::*;

    fn fixture(script: &str, profile: Profile, approval: Option<&str>, expect_error: Option<&str>) -> EvalFixture {
        let args = match script {
            "action.draft" => json!({"planFingerprint":"plan-1","actionId":"action-1"}),
            "plan.preview_sap" => json!({"planFingerprint":"plan-1"}),
            "plan.propose" => json!({"goal":"potable-water","horizonS":3600.0}),
            "answer.emit" => json!({"claims":[]}),
            _ => json!({"subject":"service:potable-water"}),
        };
        EvalFixture {
            schema: EVAL_SCHEMA.into(),
            id: "test".into(),
            preregistered_at: "2026-11-05".into(),
            prompt: "test".into(),
            profile,
            script: EvalScript { responses: vec![EvalResponse { tool: Some(script.into()), args, r#final: None }] },
            approval: approval.map(|action| EvalApproval { action: action.into() }),
            expect: EvalExpect { tools: vec![script.into()], error: expect_error.map(str::to_string), claims: vec![] },
        }
    }

    #[test]
    fn eval_runner_rejects_missing_args() {
        let mut f = fixture("state.query", Profile::Analyst, None, None);
        f.script.responses[0].args = json!({});
        assert_eq!(run_fixture(&f).error.as_deref(), Some("invalid_tool_arguments"));
    }

    #[test]
    fn eval_runner_blocks_analyst_action_draft() {
        let f = fixture("action.draft", Profile::Analyst, Some("approve"), None);
        let report = run_fixture(&f);
        assert_eq!(report.error.as_deref(), Some("policy_blocked"));
    }

    #[test]
    fn eval_runner_requires_approval_before_draft() {
        let f = fixture("action.draft", Profile::Dispatcher, None, None);
        assert_eq!(run_fixture(&f).error.as_deref(), Some("approval_required"));

        let rejected = fixture("action.draft", Profile::Dispatcher, Some("reject"), None);
        assert_eq!(run_fixture(&rejected).error.as_deref(), Some("approval_rejected"));
    }

    #[test]
    fn eval_runner_accepts_valid_fixture() {
        let f = fixture("state.query", Profile::Analyst, None, None);
        let report = run_fixture(&f);
        assert_eq!(report.error, None);
        assert_eq!(report.tools, vec!["state.query".to_string()]);
    }
}
