# Phase C2b 设计：What-if 计划提升为 Proposed Plan

- 状态：**C2b 门禁 PASS**（2026-11-06；见 VALIDATION-C2B）
- 日期：2026-11-06
- 范围：把 C1 的 **plan 结果**提升为 proposed actions；审批 + 重新校验 + 持久化
- 明确不做：dispatch、live 写、SAP 执行、federation、timed promote、长期 capability grant
- 上游：C1 `poie-whatif`、B1 `approval_tokens`、`poie-plan::sap`、`poie-state` action 状态
- 下游：D2 远程/策略变更；未来执行阶段

## 1. 背景

C1 能生成 plan 结果，但结果只留在 what-if registry。C2b 要回答：

> 这个模拟计划能否在**重新校验**后，变成 SAP 的 proposed actions，并经过操作员审批？

C2a timed 结果只有 verdict/completion，没有完整 actions，因此 **C2b 只从 C1 `mode=plan` 结果提升**。

## 2. 目标

1. 新增 `plan_to_proposed`：stale 检查 → 重新跑 plan → SAP 结构校验 → 生成 proposed actions。
2. 审批对象是**完整计划/动作集合**，不是单次工具调用。
3. 审批 token 与 proposal 绑定；pending → approved → committed。
4. 审批后 commit 仍要重新校验；不允许“批准时有效，提交时已过期”。
5. 只写 proposed，不创建 Authorization、不 dispatch、不写 live/SAP 记录。
6. 重启后 pending/approved 可恢复；不自动 commit、不自动重放。
7. 审计可回答：谁、何时、批准了哪个 plan/scope。

## 3. 非目标

- 不 dispatch、不调用 SAP、不创建 live ActionLedger 记录。
- 不做 timed 结果 promote（C2b 无 action 数据）。
- 不做长期 capability grant/角色模型；C2b 是一次性 promotion token。
- 不做远程/多节点审批。
- 不做自动重试/自动 commit。
- 不声称 proposed 等于 authorized。

## 4. 核心流程

```text
C1 result(mode=plan)
  → promote.propose
      stale checks
      re-run plan
      plan fingerprint match
      SAP structural validation
      create PromotionProposal + pending ApprovalToken
  → approve / reject
  → commit
      stale checks again
      use actions built at propose (no silent rebuild)
      check action validity window
      consume token
      persist proposed actions
  → (future) authorize/dispatch is NOT C2b
```

## 5. 数据模型

### 5.1 PromotionProposal

```text
PromotionProposal {
  schema:"poie.promotion/0.1", schemaVersion:"0.1",
  proposalId,resultId,snapshotId,variantId,
  resultSha256,snapshotSha256,variantJsonSha256,inputsSha256,
  planFingerprint,planSha256,modelSha256,stateVersion:{site:version},
  stateVersionSource:"host_assumption",
  scope:"promote:<snapshotId>:<resultId>",
  status:"pending"|"approved"|"rejected"|"committed"|"expired"|"cancelled",
  approvalTokenId,requestedAt,requestedBy,decidedAt?,committedAt?,
  nowMs, actions:[ProposedActionDraft], actionsSha256,
  sapPreview:{checks,rejections,assumed},
  proposalSha256
}
```

- **actions 在 propose 时构建并持久化**，不在 commit 时重建；`actionsSha256 = sha256("poie.proposed-action-draft/0.1\n" + canonical ProposedActionDraft[])`。
- `planSha256 = sha256("poie.promotion-plan/0.1\n" + canonical {planId,startS,horizonS,actions[{actionId,process,target,startS,endS,preconditions,allocations,verification}],goalsMet,goalsUnmet})`（独立 domain，避免与 C1 `poie.plan/0.1` payload 混用），覆盖 preconditions/allocations；`Plan::digest` 只作交叉校验，不能单独作为审批对象。
- `proposalSha256 = sha256("poie.promotion/0.1\n" + canonical proposal without id/approvalTokenId/status/requestedAt/decidedAt/committedAt/proposalSha256)`；包含 actions/actionsSha、scope、stateVersion、**nowMs**、SAP preview。状态变化不改 proposal digest。
- `scope` 必须等于 `promote:<snapshotId>:<resultId>`；approval token 的 `subject_digest` 用 B1 `poie-governance::subject_digest("whatif.promote", args, sessionId, policy, expires)` 计算，`args` 含 scope/resultId/proposalSha256；审批时比对。
- `variantJsonSha256 = sha256("poie.whatif-variant/0.1\n" + canonical variant JSON)`
- `inputsSha256 = sha256("poie.promotion-inputs/0.1\n" + canonical {snapshotSha256,variantJsonSha256,resultSha256})`；commit 必须重读这三行并比对。
- `stateVersion` 来源明确为 **host_assumption**：C2b MVP 用 `{"site-a":4}`；不是 live 真值，必须在 UI/proposal 中标注。propose 与 commit 都要求该假设未变；测试用 `4→5` 证明 stale 生效。

### 5.2 ProposedAction

```text
ProposedActionDraft {
  schema:"poie.proposed-action-draft/0.1", schemaVersion:"0.1",
  actionId,idempotencyKey,target,command,planRef,policyVersion,
  precondition,validFrom,validUntil,budget,reservations,reversibility,verificationSpec
}

ProposedAction {
  schema:"poie.proposed-action/0.1", schemaVersion:"0.1",
  proposalId, actionId, actionSha256, action:ProposedActionDraft,
  state:"proposed", authorization:null,
  promotionEvidence:{approvalTokenId, proposalSha256, basis:"approval:<tokenId>;simulation=whatif:<snapshotSha256>;model=<modelSha256>"}
}
```

- 先由 `plan_to_actions_at(plan, origin_now_s, proposal_now_ms, state_version) -> Result<Vec<Action>, SapError>` 生成；**不带 Authorization**。
- `actionSha256 = sha256("poie.proposed-action/0.1\n" + canonical ProposedActionDraft)`；`promotion_proposed_actions.action_json` 存 **draft**，`action_sha256` 是同一公式；读取时重算。
- C1 plan 的 `start_s` 基于 `snapshot.now_s`（通常 0）。重定公式：
  - `valid_from = proposal_now_ms + (start_s - origin_now_s) * 1000`
  - `valid_until = proposal_now_ms + (end_s - origin_now_s) * 1000 + 60_000`
  - 不在 `start_s` 已含 now 的情况下再次加 now。
- propose 时检查 `valid_until > proposal_now_ms`；commit 用同一 actions，`proposal_now_ms` 只用于过期判断，不重建。
- commit 时若 `now >= valid_until` → `approval_expired`，不得提交已过期动作。
- 只写 `promotion_proposed_actions` 表；不写 live ActionLedger/SAP。
- 未来 dispatch 阶段才创建 `Authorization`，basis 引用 promotion token。

## 6. 前置 API

### 6.1 `poie-plan::sap`

当前 `to_action` 私有；`validate_plan_through_sap` 会自建 planning-validation Authorization，C2b 不使用。

- 新增 `pub fn plan_to_actions_at(plan: &Plan, origin_now_s: f64, proposal_now_ms: i64, state_version: &BTreeMap<String,u64>) -> Result<Vec<Action>, SapError>`；`origin_now_s = snapshot.nowS`（C1 计划起点），不得由 callers 另传：
  - 空 state_version → `SapError::EmptyStateVersion`；
  - 不创建 Authorization；
  - 复用现有 `to_action` 的字段映射，但时间窗按 `now_ms` 重定。
- 新增 `SapError { EmptyStateVersion, InvalidPlan(String), NonFiniteTime }`。
- 新增 `pub fn preview_actions(actions: &[Action], assumed: &AssumedContext) -> Result<SapPreview, SapError>`：
  - 对已重定 actions 做 ActionLedger add→Validated；
  - 用 `AssumedContext {now_ms: proposal_now_ms, state_version, holds_command_ownership:true, reservations_held:true, predicates_holding: actions 的 preconditions 并集}`；这些是显式假设，UI 必须显示 `assumed`；
  - 收集 checks/rejections；`missing_authorization` 与 `time_window=not_yet_valid` 是预期项（计划动作在未来），只有 `time_window=expired` 才是真实拒绝；其余拒绝 → `sap_rejected`；
  - 至少一个多动作计划的 propose 测试，证明第二个动作不会被 not_yet_valid 误判；
  - 不创建 Authorization，不进入 Authorized。
- C2b 不使用 `preview_plan_through_sap`（其时间基是 plan 绝对时间，无法直接用于重定 actions），也不新增 `validate_proposed_plan`。
- 现有 `validate_plan_through_sap` 仅保留测试/旧路径。
- 映射沿用 `to_action` 的默认：`reversibility=Idempotent`、`reservations=[]`、`policy_version="plan-policy/0.1"`；C2b 明确声明这些是当前默认，不声称已优化。

### 6.2 ApprovalToken

- propose 必须在**单一 DB 事务**内完成：写 proposal、写 approval token、写 `promotion.proposed` audit；任一步失败全部回滚，禁止 dangling pending proposal。
- 表加部分唯一索引：`CREATE UNIQUE INDEX ... ON promotion_proposals(result_id) WHERE status IN ('pending','approved')`；`duplicate_proposal` 由约束冲突返回，不依赖 check-then-insert。
- 复用 `approval_tokens` 表；token 在 propose 时创建：
  - `tool = "whatif.promote"`；`tool_version="1.0.0"`
  - `run_id = proposalId`；`call_id = ""`
  - `subject_digest = subject_digest("whatif.promote", args, sessionId, "poie-promote/0.1", expires)`（显式 policy；不用 `POLICY_VERSION=poie-agent/0.1`）
  - `args_json_redacted` 存 `{scope, resultId, proposalSha256}`
  - `expires_at_ms = now + 30min`（可配置，上限 24h）
  - `impact_json` = `{actions:N, targets:[...], budgets:[...]}`
  - `citations_json` = `[{kind:"plan",id:planFingerprint},{kind:"snapshot",id:snapshotId}]`
  - `policy_version = "poie-promote/0.1"`
  - `state=pending`
- approve：校验 token 属于当前 session、`tool`、`run_id`、`args.scope`、`proposalSha256`，原子 pending→approved；proposal 同步 approved。
- reject：pending→rejected；proposal rejected。
- cancel：pending/approved→cancelled；proposal cancelled。
- commit：要求 proposal approved、token approved、未过期；校验 token scope/session；事务内 token approved→executing→consumed + 写 actions + proposal committed。
- 重复 propose：同一 result 若已有 pending/approved proposal → `duplicate_proposal`（拒绝，不新建）；committed/rejected/expired 后可重新 propose。
- 不实现 `interrupted` 状态；commit 失败回滚，proposal 仍 approved、token 仍 approved。
- C2b 的“能力范围”是一次性 token 的 scope；长期 grant/revoke/inventory 属 D2 审批扩展（scope exception，owner: D2 design）。

## 7. 命令

| 命令 | 输入 | 输出 |
| --- | --- | --- |
| `whatif_promote_propose` | resultId | PromotionProposal（pending + token） |
| `whatif_promote_get` | proposalId | PromotionProposal |
| `whatif_promote_list` | limit | PromotionProposal[] |
| `whatif_promote_approve` | proposalId | PromotionProposal（approved） |
| `whatif_promote_reject` | proposalId, reason | PromotionProposal（rejected） |
| `whatif_promote_commit` | proposalId | PromotionCommit（proposed actions） |
| `whatif_promote_cancel` | proposalId | PromotionProposal（cancelled） |

错误码：`unknown_result`、`unknown_proposal`、`not_promotable`、`duplicate_proposal`、`invalid_status`、`stale_snapshot`、`stale_result`、`plan_mismatch`、`sap_rejected`、`approval_required`、`approval_mismatch`、`approval_expired`、`scope_denied`、`budget_exceeded`、`unauthorized`。

C2b 不新增 `dispatch`/`authorize` 命令。

所有命令使用统一 `CommandError { code, message, details? }` envelope（C2b 新命令；现有 C1/C2a 命令不改签名）。scope 校验由 host `authorize(window="poie-view-plan", command="whatif_promote_propose", session, policy)` 执行（PLATFORM window=main/poie-view-plan，command 是实际命令名）；approve/commit 同理分别传命令名。token 的 `tool="whatif.promote"`、`policy_version="poie-promote/0.1"`，`subject_digest` 显式使用该 policy version，不用 `poie-agent/0.1`。`no_live_write` 断言：proposed action `authorization==None`、不调用 `execution::snapshot`、不写 registry/SAP/federation；命令注册表/写表白名单允许列表只含 `whatif_timed_*`/`whatif_promote_*`/`promotion_*`，测试断言不在白名单的写函数未被调用。

## 8. Stale 检查

propose 和 commit 都必须检查：

1. result 存在、`mode=plan`、`outcome.plan=="planned"`、actions 非空；否则 `not_promotable`；
2. snapshot 存在；`model_sha256 == state.model digest`；
3. snapshot/variant/result 的 digest 与当前行一致；
4. 用 snapshot+variant **重新运行** `run_plan`，`plan_fingerprint` 与 result 记录一致；
5. `state_version` 与 proposal 中记录一致（commit 时不得漂移）；
6. commit 时 token 未过期、proposal status=approved。

任一失败：`stale_snapshot`/`stale_result`/`plan_mismatch`，不写 proposed。

## 9. 状态机与重启

```text
pending --approve--> approved --commit--> committed
pending --reject--> rejected
pending/approved --expire--> expired
pending/approved --cancel--> cancelled
```

重复操作语义（明确选择）：

| 操作 | 允许起点 | 重复调用 | 其他状态 |
| --- | --- | --- | --- |
| approve | pending | 已 approved 且 token/proposal 匹配 → 返回同一 proposal（幂等） | `invalid_status` |
| reject | pending | 已 rejected 且同一 proposal → 返回（幂等） | `invalid_status` |
| cancel | pending/approved | 已 cancelled → 返回（幂等） | `invalid_status` |
| commit | approved | 已 committed 且同一 proposal/token → 返回已存 `PromotionCommit`（幂等） | `invalid_status` |

- token 与 proposal 的 approve/reject/cancel 必须在**同一 DB 事务**中更新；token 条件更新失败则回滚 proposal。
- commit 成功事务：写 `promotion_proposed_actions`、写 `promotion_commits`、token `approved→executing→consumed`、proposal=committed、**append_audit**，全在同一事务。
- 重启：pending/approved 保留；**不自动 approve/commit**；propose/read/approve/commit 时在同一事务内把过期 proposal 与 token 标 expired（无后台定时器）。已 committed 的重复 commit 返回存储的 commit，不重复写。
- 进程在 commit 中断：事务回滚到 approved；不得出现“token consumed 但无 actions/commit”或反之。
- 测试：propose 插入 proposal 后强制 token 插入失败 → 无 proposal；并发 duplicate propose → 一个成功、一个 `duplicate_proposal`；过期后 approve/commit → `approval_expired` 且 proposal 同步 expired。
- consumed token 不允许被另一个 proposal 复用；`approval_mismatch`。

## 10. 持久化（v6）

```text
PromotionCommit {
  schema:"poie.promotion-commit/0.1", schemaVersion:"0.1",
  proposalId, planSha256, actionsSha256, committedAt, committedBy,
  tokenId, inputsSha256, commitSha256
}

promotion_proposals(
  proposal_id PK, result_id, snapshot_id, variant_id, schema_version,
  proposal_json, proposal_sha256, status, approval_token_id, created_at, updated_at
)
promotion_proposed_actions(
  proposal_id, action_id, action_json, action_sha256, created_at,
  PRIMARY KEY(proposal_id, action_id)
)
promotion_commits(
  proposal_id PK, schema_version, commit_json, commit_sha256,
  inputs_sha256, token_id, committed_at, committed_by
)
```

- `commitSha256 = sha256("poie.promotion-commit/0.1\n" + canonical commit without commitSha)`；`planSha256` 同上。
- `PRAGMA user_version=6`；只加表，不改 C1/C2a 表；现有 `db.rs` 的 v5 迁移需先升到 6，并补 v5→v6 fixture（含已有 C1/C2a 行）。
- 迁移 5→6 幂等；失败回滚。
- proposal/action JSON 写入前重算 digest；读取时校验。

## 11. UI

- C1 plan 结果行增加“提升为 Proposed”动作；timed 结果不显示。
- 预览弹窗显示：
  - plan fingerprint、动作数量、时间窗、预算
  - SAP checks/rejections
  - scope、过期时间
- 操作：批准 / 拒绝 / 提交（approved 后）；不显示 Dispatch/执行。
- 状态徽标：pending/approved/committed/rejected/expired；不把 proposed 标成 live。
- 浏览器 host-only 占位。

## 12. 隔离与审计

- 不调用 SAP/dispatch/live/federation；测试断言无 live 写入路径。
- 审计事件：`promotion.proposed`、`promotion.approved`、`promotion.rejected`、`promotion.committed`、`promotion.expired`、`promotion.cancelled`。
- 审计 detail 含 proposalId、resultId、scope、tokenId、planSha 前 12 位；不记录密钥/完整敏感 payload。
- commit 的 audit 行与业务写入在同一事务内；崩溃不会留下“committed 但无 audit”。
- 当前 `append_audit` 不写 `prev_hash/hash`；C2b **不声称审计链完整**；链迁移/外部锚由 B2/D2 单独门禁。

## 13. 测试与验收

- propose：plan result → pending token；timed result / 非 planned / 空 actions 拒绝；stale model/snapshot/result 拒绝。
- actions 在 propose 固化；approve 后改 result/snapshot 不影响已存 actions；commit 不重建。
- plan mismatch（改 result JSON/重新运行指纹不同）拒绝。
- SAP structural failure → `sap_rejected`，不建 token。
- approve/reject/cancel/expire；重复操作幂等/拒绝一致。
- commit：approved+unexpired → proposed actions；token consumed；无 dispatch/live 写。
- commit 时 stale/token expired/action 时间窗过期 → 拒绝，无部分写；stateVersion 4→5 拒绝。
- 重启：pending/approved 保留、不自动 commit；过期→expired。
- commit 后存在 `promotion.committed` audit 事件；不宣称 hash 链完整。
- UI 显示 scope/checks；timed 无 promote 按钮。
- 验收：真实 app 对 C1 plan result 完成 propose→approve→commit，DB 出现 proposed actions，live/SAP 无写入。

## 14. C2b 门禁清单

- [ ] plan_to_proposed 定义与 stale 规则闭合
- [ ] 审批 token scope/subject 绑定
- [ ] SAP 参数化校验，不创建 Authorization
- [ ] 状态机/重启/幂等/部分写
- [ ] 无 dispatch/live 声明与测试
- [ ] UI/审计/错误码
- [ ] timed 结果不可 promote 的边界

## 15. Traceweave 输入的 scope 例外

C2b 实现 C2b-1/2/3 的**一次性 promotion token**版本（scope、operator/session、revoke/expiry/重启）。  
以下明确延后，owner 与阶段如下：

| 输入 | 处理 | Owner |
| --- | --- | --- |
| C2b-2 签名指纹 | host 无签名身份表面，暂用 operator/session；签名绑定随 D2 签名身份实现 | D2 |
| C2b-7 capability inventory/最小权限 | C2b 只列 promote 能力；完整 inventory 随 D2 审批扩展 | D2 |
| 长期 grant/revoke | C2b 不实现；一次性 token 可 cancel/expire | D2 |
| C2b-8 授权 UI | C2b 只做 proposal 弹窗；长期授权 UI 随 D2 | D2 |

这不是静默丢弃：C2b 门禁要求上述例外在 D2 设计中有对应项。

## 16. 参考

- [Phase C1](phase-c-what-if.zh-CN.md)、[C2a](phase-c2-timed-what-if.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)
- [traceweave 借鉴清单](traceweave-adoption-inputs.zh-CN.md)
- `poie-plan::sap`、`poie-state::action`、B1 `approval_tokens`
