# Phase C2b 设计：What-if 计划提升为 Proposed Plan

- 状态：设计草案（待 C2b 门禁）
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

- **actions 在 propose 时构建并持久化**，不在 commit 时重建；`actionsSha256 = sha256("poie.proposed-action/0.1\n" + canonical actions[])`。
- `planSha256 = sha256("poie.plan/0.1\n" + canonical plan payload)`，只作为交叉校验；`Plan::digest` 不含 preconditions/allocations，不能单独作为审批对象。
- `proposalSha256 = sha256("poie.promotion/0.1\n" + canonical proposal without id/status/times/proposalSha256)`；包含 actions/actionsSha、scope、stateVersion、nowMs、SAP preview。状态变化不改 proposal digest。
- `scope` 必须等于 `promote:<snapshotId>:<resultId>`；approval token 的 `subject_digest` 用 B1 `poie-governance::subject_digest("whatif.promote", args, sessionId, policy, expires)` 计算，`args` 含 scope/resultId/proposalSha256；审批时比对。
- `stateVersion` 来源明确为 **host_assumption**：C2b MVP 用 `{"site-a":4}`；不是 live 真值，必须在 UI/proposal 中标注。propose 与 commit 都要求该假设未变；测试用 `4→5` 证明 stale 生效。

### 5.2 ProposedAction

```text
ProposedAction {
  schema:"poie.proposed-action/0.1", schemaVersion:"0.1",
  proposalId, actionId, idempotencyKey, target, command, planRef,
  policyVersion, precondition, validFrom, validUntil, budget, reservations,
  reversibility, verificationSpec,
  state:"proposed",
  authorization:null,
  promotionEvidence:{approvalTokenId, proposalSha256, basis:"approval:<tokenId>;simulation=whatif:<snapshotSha256>;model=<modelSha256>"}
}
```

- 由新增 `poie-plan` 公开 `plan_to_actions_at(plan, now_ms, state_version) -> Result<Vec<Action>, SapError>` 生成；**不带 Authorization**。
- 时间窗以 `now_ms` 为基准重定：`valid_from = now + start_s*1000`，`valid_until = now + end_s*1000 + 60_000`；propose 时即检查 `valid_until > now`。
- commit 时若 `now >= valid_until` → `approval_expired`，不得提交已过期动作。
- 只写 `promotion_proposed_actions` 表；不写 live ActionLedger/SAP。
- 未来 dispatch 阶段才创建 `Authorization`，basis 引用 promotion token。

## 6. 前置 API

### 6.1 `poie-plan::sap`

当前 `to_action` 私有；`validate_plan_through_sap` 会自建 planning-validation Authorization，C2b 不使用。

- 新增 `pub fn plan_to_actions_at(plan: &Plan, now_ms: i64, state_version: &BTreeMap<String,u64>) -> Result<Vec<Action>, SapError>`：
  - 空 state_version → `SapError::EmptyStateVersion`；
  - 不创建 Authorization；
  - 复用现有 `to_action` 的字段映射，但时间窗按 `now_ms` 重定。
- `preview_plan_through_sap(plan, AssumedContext)` 已提供 checks/assumed/rejections，且不创建 Authorization；**C2b 的 sapPreview 直接用它**，不新增 `validate_proposed_plan`。
- `review_dispatch` 在 Validated 状态会报 `missing_authorization`；C2b 不把该 checks 当“SAP 失败”，只把非 Authorization 的拒绝视为 `sap_rejected`。
- 现有 `validate_plan_through_sap` 仅保留测试/旧路径。

### 6.2 ApprovalToken

- 复用 `approval_tokens` 表；token 在 propose 时创建：
  - `tool = "whatif.promote"`；`tool_version="1.0.0"`
  - `run_id = proposalId`；`call_id = ""`
  - `subject_digest = subject_digest("whatif.promote", args, sessionId, POLICY_VERSION, expires)`
  - `args_json_redacted` 存 `{scope, resultId, proposalSha256}`
  - `expires_at_ms = now + 30min`（可配置，上限 24h）
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

- 状态迁移必须原子条件更新；重复 approve/commit 幂等或拒绝（明确选择）。
- commit 成功：写入 proposed actions、token `approved→executing→consumed`、proposal=committed，同一事务。
- 重启：pending/approved 保留；**不自动 approve/commit**；过期 token 标 expired。
- 进程在 commit 中断：proposal 保持 approved 或 committed（以事务为准）；不得出现“token consumed 但无 actions”或反之。

## 10. 持久化（v6）

```text
PromotionCommit {
  schema:"poie.promotion-commit/0.1", schemaVersion:"0.1",
  proposalId, planSha256, actionsSha256, committedAt, committedBy,
  tokenId, commitSha256
}

promotion_proposals(
  proposal_id PK, result_id, snapshot_id, variant_id, schema_version,
  proposal_json, proposal_sha256, status, approval_token_id, created_at, updated_at
)
promotion_proposed_actions(
  proposal_id, action_id, action_json, action_sha256, created_at,
  PRIMARY KEY(proposal_id, action_id)
)
```

- `PRAGMA user_version=6`；只加表，不改 C1/C2a 表；现有 `db.rs` 的 v5 迁移需先升到 6，并补 v5→v6 fixture。
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
- 当前 `append_audit` 不写 `prev_hash/hash`；C2b 不声称审计链完整，链迁移/外部锚由 B2/D2 单独门禁。C2b 只保证事件与业务写入同一 DB 连接顺序。

## 13. 测试与验收

- propose：plan result → pending token；timed result / 非 planned / 空 actions 拒绝；stale model/snapshot/result 拒绝。
- actions 在 propose 固化；approve 后改 result/snapshot 不影响已存 actions；commit 不重建。
- plan mismatch（改 result JSON/重新运行指纹不同）拒绝。
- SAP structural failure → `sap_rejected`，不建 token。
- approve/reject/cancel/expire；重复操作幂等/拒绝一致。
- commit：approved+unexpired → proposed actions；token consumed；无 dispatch/live 写。
- commit 时 stale/token expired/action 时间窗过期 → 拒绝，无部分写；stateVersion 4→5 拒绝。
- 重启：pending/approved 保留、不自动 commit；过期→expired。
- 审计链完整；UI 显示 scope/checks；timed 无 promote 按钮。
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
