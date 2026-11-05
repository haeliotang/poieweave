//! Phase C2b promotion DTOs: approved C1 plan results become proposed actions.
//! This module does not create Authorization, dispatch, or write live state.

use std::collections::BTreeMap;

use poie_digest::sha256_of;
use poie_plan::plan::Plan;
use poie_plan::sap::SapPreview;
use poie_state::action::Action;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const PROMOTION_SCHEMA: &str = "poie.promotion/0.1";
pub const PROPOSED_ACTION_SCHEMA: &str = "poie.proposed-action/0.1";
pub const PROMOTION_COMMIT_SCHEMA: &str = "poie.promotion-commit/0.1";
pub const PROMOTE_POLICY_VERSION: &str = "poie-promote/0.1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionProposal {
    pub schema: String,
    pub schema_version: String,
    pub proposal_id: String,
    pub result_id: String,
    pub snapshot_id: String,
    pub variant_id: String,
    pub result_sha256: String,
    pub snapshot_sha256: String,
    pub variant_json_sha256: String,
    pub inputs_sha256: String,
    pub plan_fingerprint: String,
    pub plan_sha256: String,
    pub model_sha256: String,
    pub state_version: BTreeMap<String, u64>,
    pub state_version_source: String,
    pub scope: String,
    pub status: String,
    pub approval_token_id: String,
    pub requested_at: i64,
    pub requested_by: String,
    pub decided_at: Option<i64>,
    pub committed_at: Option<i64>,
    pub now_ms: i64,
    pub actions: Vec<Action>,
    pub actions_sha256: String,
    pub sap_preview: SapPreview,
    pub proposal_sha256: String,
}

impl PromotionProposal {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        result_id: String,
        snapshot_id: String,
        variant_id: String,
        result_sha256: String,
        snapshot_sha256: String,
        variant_json_sha256: String,
        plan: &Plan,
        model_sha256: String,
        state_version: BTreeMap<String, u64>,
        actions: Vec<Action>,
        sap_preview: SapPreview,
        proposal_id: String,
        token_id: String,
        now_ms: i64,
        requested_by: String,
    ) -> Result<Self, String> {
        let plan_sha256 = promotion_plan_sha256(plan)?;
        let actions_sha256 = proposed_actions_sha256(&actions)?;
        let inputs_sha256 = promotion_inputs_sha256(&snapshot_sha256, &variant_json_sha256, &result_sha256)?;
        let scope = format!("promote:{snapshot_id}:{result_id}");
        let mut proposal = PromotionProposal {
            schema: PROMOTION_SCHEMA.into(),
            schema_version: "0.1".into(),
            proposal_id,
            result_id,
            snapshot_id,
            variant_id,
            result_sha256,
            snapshot_sha256,
            variant_json_sha256,
            inputs_sha256,
            plan_fingerprint: plan.digest(),
            plan_sha256,
            model_sha256,
            state_version,
            state_version_source: "host_assumption".into(),
            scope,
            status: "pending".into(),
            approval_token_id: token_id,
            requested_at: now_ms,
            requested_by,
            decided_at: None,
            committed_at: None,
            now_ms,
            actions,
            actions_sha256,
            sap_preview,
            proposal_sha256: String::new(),
        };
        proposal.proposal_sha256 = promotion_proposal_digest(&proposal)?;
        Ok(proposal)
    }
}

pub fn promotion_plan_sha256(plan: &Plan) -> Result<String, String> {
    let actions: Vec<Value> = plan
        .actions
        .iter()
        .map(|action| {
            json!({
                "actionId": action.action_id,
                "process": action.process,
                "target": action.target,
                "startS": action.start_s,
                "endS": action.end_s,
                "preconditions": action.preconditions,
                "allocations": action.allocations,
                "occupies": action.occupies,
                "restores": action.restores,
                "verification": action.verification.to_string(),
            })
        })
        .collect();
    let goals_unmet: Vec<Value> =
        plan.goals_unmet.iter().map(|failure| json!({"goal": failure.goal, "reason": failure.reason})).collect();
    sha256_of(
        "poie.promotion-plan/0.1",
        &json!({
            "planId": plan.plan_id,
            "startS": plan.start_s,
            "horizonS": plan.horizon_s,
            "actions": actions,
            "goalsMet": plan.goals_met,
            "goalsUnmet": goals_unmet,
        }),
    )
    .map_err(|error| error.to_string())
}

pub fn proposed_action_sha256(action: &Action) -> Result<String, String> {
    sha256_of(PROPOSED_ACTION_SCHEMA, action).map_err(|error| error.to_string())
}

pub fn proposed_actions_sha256(actions: &[Action]) -> Result<String, String> {
    sha256_of("poie.proposed-action-draft/0.1", &actions).map_err(|error| error.to_string())
}

pub fn promotion_inputs_sha256(
    snapshot_sha256: &str,
    variant_json_sha256: &str,
    result_sha256: &str,
) -> Result<String, String> {
    sha256_of(
        "poie.promotion-inputs/0.1",
        &json!({
            "snapshotSha256": snapshot_sha256,
            "variantJsonSha256": variant_json_sha256,
            "resultSha256": result_sha256,
        }),
    )
    .map_err(|error| error.to_string())
}

pub fn promotion_proposal_digest(proposal: &PromotionProposal) -> Result<String, String> {
    let mut value = serde_json::to_value(proposal).map_err(|error| error.to_string())?;
    if let Some(object) = value.as_object_mut() {
        for key in
            ["proposalSha256", "proposalId", "approvalTokenId", "status", "requestedAt", "decidedAt", "committedAt"]
        {
            object.remove(key);
        }
    }
    sha256_of(PROMOTION_SCHEMA, &value).map_err(|error| error.to_string())
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposedAction {
    pub schema: String,
    pub schema_version: String,
    pub proposal_id: String,
    pub action_id: String,
    pub action_sha256: String,
    pub action: Action,
    pub state: String,
    pub authorization: Option<Value>,
    pub promotion_evidence: PromotionEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionEvidence {
    pub approval_token_id: String,
    pub proposal_sha256: String,
    pub basis: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromotionCommit {
    pub schema: String,
    pub schema_version: String,
    pub proposal_id: String,
    pub plan_sha256: String,
    pub actions_sha256: String,
    pub inputs_sha256: String,
    pub token_id: String,
    pub committed_at: i64,
    pub committed_by: String,
    pub commit_sha256: String,
}

impl PromotionCommit {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        proposal: &PromotionProposal,
        actions_sha256: String,
        committed_at: i64,
        committed_by: String,
    ) -> Result<Self, String> {
        let mut commit = PromotionCommit {
            schema: PROMOTION_COMMIT_SCHEMA.into(),
            schema_version: "0.1".into(),
            proposal_id: proposal.proposal_id.clone(),
            plan_sha256: proposal.plan_sha256.clone(),
            actions_sha256,
            inputs_sha256: proposal.inputs_sha256.clone(),
            token_id: proposal.approval_token_id.clone(),
            committed_at,
            committed_by,
            commit_sha256: String::new(),
        };
        commit.commit_sha256 = promotion_commit_digest(&commit)?;
        Ok(commit)
    }
}

pub fn promotion_commit_digest(commit: &PromotionCommit) -> Result<String, String> {
    let mut value = serde_json::to_value(commit).map_err(|error| error.to_string())?;
    if let Some(object) = value.as_object_mut() {
        object.remove("commitSha256");
    }
    sha256_of(PROMOTION_COMMIT_SCHEMA, &value).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use poie_plan::plan::{Plan, PlannedAction};
    use poie_state::action::{Precondition, Reversibility, VerificationSpec};
    use poie_state::record::Timestamp;

    fn plan() -> Plan {
        Plan {
            plan_id: "plan-0".into(),
            start_s: 0.0,
            horizon_s: 86_400.0,
            actions: vec![PlannedAction {
                action_id: "action-1".into(),
                process: "replace-sleeve-p1".into(),
                target: "site".into(),
                runs: 1,
                start_s: 0.0,
                end_s: 10_800.0,
                preconditions: vec!["pump-p1 available".into()],
                allocations: vec![("electric-energy".into(), 1_000.0)],
                occupies: vec!["robot-m1".into()],
                restores: vec!["pump-p1".into()],
                verification: poie_ontology::model::Verification::Verified,
            }],
            goals_met: vec!["restore:pump-p1".into()],
            goals_unmet: Vec::new(),
            completion_s: 10_800.0,
        }
    }

    fn action() -> Action {
        Action {
            action_id: "plan-0-action-1".into(),
            idempotency_key: "plan-0:action-1".into(),
            target: "site".into(),
            command: "replace-sleeve-p1".into(),
            plan_ref: Some("plan-0".into()),
            policy_version: "plan-policy/0.1".into(),
            precondition: Precondition {
                state_version: [("site-a".into(), 4)].into_iter().collect(),
                predicates: vec![],
            },
            valid_from: Timestamp(1_000),
            valid_until: Timestamp(20_000),
            budget: vec![],
            reservations: vec![],
            reversibility: Reversibility::Idempotent,
            verification_spec: VerificationSpec { observe: "x".into(), condition: "y".into(), window_ms: 1 },
            authorization: None,
            redispatch_of: None,
        }
    }

    #[test]
    fn proposal_digest_ignores_identity_and_status() {
        let action = action();
        let preview = SapPreview {
            checks: vec![],
            assumed: vec![],
            rejections: vec![],
            authorized_actions: 0,
            authorization_created: false,
        };
        let mut proposal = PromotionProposal::create(
            "res-1".into(),
            "snap-1".into(),
            "var-1".into(),
            "r".repeat(64),
            "s".repeat(64),
            "v".repeat(64),
            &plan(),
            "m".repeat(64),
            [("site-a".into(), 4)].into_iter().collect(),
            vec![action],
            preview,
            "prom-1".into(),
            "tok-1".into(),
            1_000,
            "tester".into(),
        )
        .unwrap();
        let before = proposal.proposal_sha256.clone();
        proposal.status = "approved".into();
        proposal.proposal_id = "prom-2".into();
        proposal.decided_at = Some(2_000);
        assert_eq!(before, promotion_proposal_digest(&proposal).unwrap());
        assert_eq!(promotion_plan_sha256(&plan()).unwrap().len(), 64);
        assert_eq!(proposed_action_sha256(&proposal.actions[0]).unwrap().len(), 64);
    }
}
