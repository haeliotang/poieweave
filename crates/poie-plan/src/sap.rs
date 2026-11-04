//! SAP validation path for plans (M8.1, M8 exit requirement).
//!
//! Every planned action is turned into a SAP `Action`, added to an action ledger
//! as `proposed`, advanced through `validated` and `authorized` using the
//! normative transition table, and then checked by the six dispatch checks. The
//! planner never dispatches; actual dispatch still needs a real authorizer and
//! the executor, so no plan bypasses the authorization path.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use poie_state::action::{
    Action, ActionLedger, ActionState, Authorization, BudgetItem, Precondition, Reversibility, VerificationSpec,
};
use poie_state::record::Timestamp;
use poie_state::review::{review_dispatch, DispatchContext, ReviewCheck};

use crate::plan::{Plan, PlannedAction};

#[derive(Clone, Debug, PartialEq)]
pub struct SapValidation {
    pub authorized_actions: usize,
    pub review_checks: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AssumedContext {
    pub now_ms: i64,
    pub holds_command_ownership: bool,
    pub reservations_held: bool,
    pub predicates_holding: BTreeSet<String>,
    pub state_version: BTreeMap<String, u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CheckPreview {
    pub check: String,
    pub status: String, // observed|assumed|failed|unknown
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SapPreview {
    pub checks: Vec<CheckPreview>,
    pub assumed: Vec<String>,
    pub rejections: Vec<String>,
    pub authorized_actions: usize,
    pub authorization_created: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PreviewError {
    EmptyStateVersion,
    InvalidPlan(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum PlanValidationError {
    EmptyStateVersion,
    Structural(String),
    Transition(String),
    Review(String),
    Overlap(String),
}

impl fmt::Display for PlanValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlanValidationError::EmptyStateVersion => {
                f.write_str("plan validation needs a non-empty state-version vector")
            }
            PlanValidationError::Structural(detail) => write!(f, "structural action error: {detail}"),
            PlanValidationError::Transition(detail) => write!(f, "lifecycle transition rejected: {detail}"),
            PlanValidationError::Review(detail) => write!(f, "dispatch review rejected the planned action: {detail}"),
            PlanValidationError::Overlap(detail) => write!(f, "planned actions overlap: {detail}"),
        }
    }
}

impl std::error::Error for PlanValidationError {}

fn to_action(
    plan: &Plan,
    planned: &PlannedAction,
    state_version: &BTreeMap<String, u64>,
    authorization: Option<Authorization>,
) -> Action {
    let start_ms = (planned.start_s * 1000.0) as i64;
    let end_ms = (planned.end_s * 1000.0) as i64;
    Action {
        action_id: format!("{}-{}", plan.plan_id, planned.action_id),
        idempotency_key: format!("{}:{}", plan.plan_id, planned.action_id),
        target: planned.target.clone(),
        command: planned.process.clone(),
        plan_ref: Some(plan.plan_id.clone()),
        policy_version: "plan-policy/0.1".to_string(),
        precondition: Precondition { state_version: state_version.clone(), predicates: planned.preconditions.clone() },
        valid_from: Timestamp(start_ms),
        valid_until: Timestamp(end_ms + 60_000),
        budget: planned
            .allocations
            .iter()
            .filter(|(_, amount)| *amount > 0.0)
            .map(|(resource, amount)| BudgetItem { resource: resource.clone(), amount_si: *amount })
            .collect(),
        reservations: Vec::new(),
        reversibility: Reversibility::Idempotent,
        verification_spec: VerificationSpec {
            observe: format!("{} outcome", planned.process),
            condition: "process reports completed and the effect is observed".to_string(),
            window_ms: 60_000,
        },
        authorization,
        redispatch_of: None,
    }
}

/// Runs the plan through the SAP lifecycle and the six dispatch checks.
pub fn validate_plan_through_sap(
    plan: &Plan,
    _site: &str,
    state_version: &BTreeMap<String, u64>,
) -> Result<SapValidation, PlanValidationError> {
    if state_version.is_empty() {
        return Err(PlanValidationError::EmptyStateVersion);
    }
    // Plans are sequential: non-decreasing starts and no overlap on one asset.
    for pair in plan.actions.windows(2) {
        if pair[1].start_s + 1e-9 < pair[0].start_s {
            return Err(PlanValidationError::Structural("actions are not ordered by start time".into()));
        }
    }
    for (index, first) in plan.actions.iter().enumerate() {
        for second in plan.actions.iter().skip(index + 1) {
            if first.start_s < second.end_s && second.start_s < first.end_s {
                for asset in &first.occupies {
                    if second.occupies.contains(asset) {
                        return Err(PlanValidationError::Overlap(format!(
                            "{} and {} both occupy {asset}",
                            first.action_id, second.action_id
                        )));
                    }
                }
            }
        }
    }

    let mut ledger = ActionLedger::new();
    let mut review_checks = 0usize;
    for planned in &plan.actions {
        let action = to_action(
            plan,
            planned,
            state_version,
            Some(Authorization {
                by: "planning-validation".to_string(),
                basis: "validation-only entry; dispatch still needs the authorizer".to_string(),
                scope: "dispatch".to_string(),
                valid_until: Timestamp((planned.end_s * 1000.0) as i64 + 3_600_000),
            }),
        );
        action.validate().map_err(|error| PlanValidationError::Structural(error.to_string()))?;
        ledger.add(&action).map_err(|error| PlanValidationError::Structural(error.to_string()))?;
        ledger
            .apply(&action.action_id, ActionState::Validated)
            .map_err(|error| PlanValidationError::Transition(error.to_string()))?;
        ledger
            .apply(&action.action_id, ActionState::Authorized)
            .map_err(|error| PlanValidationError::Transition(error.to_string()))?;
        let context = DispatchContext {
            now_ms: action.valid_from.0,
            current_state_version: state_version.clone(),
            holds_command_ownership: true,
            reservations_held: true,
            predicates_holding: action.precondition.predicates.iter().cloned().collect::<BTreeSet<String>>(),
            authorization_revoked: false,
            ownership_dispatch_deadline_ms: None,
            executor_dedup_window_ms: None,
        };
        review_dispatch(&action, &ledger, &context)
            .map_err(|rejections| PlanValidationError::Review(format!("{rejections:?}")))?;
        review_checks += 1;
        if ledger.state(&action.action_id) != Some(ActionState::Authorized) {
            return Err(PlanValidationError::Transition("action did not reach authorized".into()));
        }
    }
    Ok(SapValidation { authorized_actions: plan.actions.len(), review_checks })
}

/// Read-only SAP preview. It never creates an Authorization, never advances an
/// action to Authorized and never dispatches. `assumed` context is explicit.
pub fn preview_plan_through_sap(plan: &Plan, assumed: &AssumedContext) -> Result<SapPreview, PreviewError> {
    if assumed.state_version.is_empty() {
        return Err(PreviewError::EmptyStateVersion);
    }
    if plan.actions.is_empty() {
        return Ok(SapPreview {
            checks: Vec::new(),
            assumed: Vec::new(),
            rejections: Vec::new(),
            authorized_actions: 0,
            authorization_created: false,
        });
    }
    let mut ledger = ActionLedger::new();
    let mut rejections = Vec::new();
    let mut checks = Vec::new();
    for planned in &plan.actions {
        let action = to_action(plan, planned, &assumed.state_version, None);
        action.validate().map_err(|error| PreviewError::InvalidPlan(error.to_string()))?;
        ledger.add(&action).map_err(|error| PreviewError::InvalidPlan(error.to_string()))?;
        ledger
            .apply(&action.action_id, ActionState::Validated)
            .map_err(|error| PreviewError::InvalidPlan(error.to_string()))?;
        let context = DispatchContext {
            now_ms: assumed.now_ms,
            current_state_version: assumed.state_version.clone(),
            holds_command_ownership: assumed.holds_command_ownership,
            reservations_held: assumed.reservations_held,
            predicates_holding: assumed.predicates_holding.clone(),
            authorization_revoked: false,
            ownership_dispatch_deadline_ms: None,
            executor_dedup_window_ms: None,
        };
        let result = review_dispatch(&action, &ledger, &context);
        for check in [
            ReviewCheck::TimeWindow,
            ReviewCheck::Preconditions,
            ReviewCheck::CommandOwnership,
            ReviewCheck::Reservations,
            ReviewCheck::IdempotencyKey,
            ReviewCheck::Authorization,
            ReviewCheck::DedupWindow,
        ] {
            let rejection = result.as_ref().err().and_then(|items| items.iter().find(|item| item.check == check));
            let status = if check == ReviewCheck::Authorization {
                "assumed"
            } else if rejection.is_some() {
                "failed"
            } else {
                "observed"
            };
            let detail = rejection.map(|item| item.detail.clone()).unwrap_or_else(|| {
                if check == ReviewCheck::Authorization {
                    "authorization not created in preview".to_string()
                } else {
                    "ok".to_string()
                }
            });
            if status == "assumed" && !checks.iter().any(|entry: &CheckPreview| entry.check == "authorization") {
                // keep one assumed entry per preview; actions share the same assumption
            }
            if !checks.iter().any(|entry: &CheckPreview| entry.check == check.as_str()) {
                checks.push(CheckPreview { check: check.as_str().to_string(), status: status.to_string(), detail });
            }
            if check != ReviewCheck::Authorization {
                if let Some(item) = rejection {
                    if !rejections.iter().any(|existing| existing == &item.detail) {
                        rejections.push(item.detail.clone());
                    }
                }
            }
        }
    }
    let assumed_checks = checks
        .iter()
        .filter(|check| check.status == "assumed")
        .map(|check| format!("{}: {}", check.check, check.detail))
        .collect();
    Ok(SapPreview { checks, assumed: assumed_checks, rejections, authorized_actions: 0, authorization_created: false })
}

#[cfg(test)]
mod tests {
    use super::*;
    use poie_ontology::graph::Goal;
    use poie_ontology::model::{Model, Verification};
    use std::path::PathBuf;

    use crate::plan::{plan, PlanningGoal, PlanningRequest};

    fn base01() -> Model {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../scenarios/base-01/model.toml");
        Model::load(&path).unwrap()
    }

    fn service_plan() -> Plan {
        let model = base01();
        let request = PlanningRequest {
            model: &model,
            now_s: 0.0,
            horizon_s: 86_400.0,
            goals: vec![PlanningGoal { goal: Goal::Service("potable-water".into()), deadline_s: None, priority: 10 }],
            min_verification: Verification::Demonstrated,
            max_actions: 20,
        };
        plan(&request).unwrap()
    }

    fn state_version() -> BTreeMap<String, u64> {
        BTreeMap::from([("site-a".to_string(), 42)])
    }

    #[test]
    fn a_planned_route_passes_the_sap_lifecycle_and_review() {
        let plan = service_plan();
        let validation = validate_plan_through_sap(&plan, "site-a", &state_version()).unwrap();
        assert_eq!(validation.authorized_actions, plan.actions.len());
        assert_eq!(validation.review_checks, plan.actions.len());
    }

    #[test]
    fn validation_without_a_state_version_fails_closed() {
        let plan = service_plan();
        assert_eq!(
            validate_plan_through_sap(&plan, "site-a", &BTreeMap::new()),
            Err(PlanValidationError::EmptyStateVersion)
        );
    }

    #[test]
    fn an_empty_plan_validates_without_checks() {
        let mut plan = service_plan();
        plan.actions.clear();
        let validation = validate_plan_through_sap(&plan, "site-a", &state_version()).unwrap();
        assert_eq!(validation.authorized_actions, 0);
        assert_eq!(validation.review_checks, 0);
    }

    #[test]
    fn overlapping_actions_are_rejected() {
        let mut plan = service_plan();
        plan.actions[0].occupies = vec!["lathe-1".into()];
        let mut second = plan.actions[0].clone();
        second.action_id = "duplicate".into();
        plan.actions.push(second);
        let error = validate_plan_through_sap(&plan, "site-a", &state_version()).unwrap_err();
        assert!(matches!(error, PlanValidationError::Overlap(_)), "{error}");
    }

    #[test]
    fn structurally_invalid_actions_are_rejected() {
        let mut plan = service_plan();
        plan.actions[0].process.clear();
        let error = validate_plan_through_sap(&plan, "site-a", &state_version()).unwrap_err();
        assert!(matches!(error, PlanValidationError::Structural(_)), "{error}");
    }

    #[test]
    fn preview_is_read_only_and_marks_authorization_assumed() {
        let plan = service_plan();
        let assumed = AssumedContext {
            now_ms: 0,
            holds_command_ownership: true,
            reservations_held: true,
            predicates_holding: plan.actions.iter().flat_map(|action| action.preconditions.clone()).collect(),
            state_version: state_version(),
        };
        let preview = preview_plan_through_sap(&plan, &assumed).unwrap();
        assert_eq!(preview.checks.len(), 7);
        assert!(preview.checks.iter().any(|check| check.check == "authorization" && check.status == "assumed"));
        assert!(!preview.assumed.is_empty());
        assert!(!preview.rejections.iter().any(|reason| reason.contains("authorization")));
        assert_eq!(preview.authorized_actions, 0);
        assert!(!preview.authorization_created);
    }
}
