# Phase C2b 设计门禁（独立）

- 校验对象：[phase-c2b-promote](phase-c2b-promote.zh-CN.md)，以及 `poie-plan::sap`/`plan`、`poie-state::review`、B1 `approval_tokens`、host `whatif.rs`/`db.rs`、[PLATFORM](PLATFORM.zh-CN.md)/[DIGEST](DIGEST.md)/[SCHEMAS](SCHEMAS.md)、[traceweave 输入清单](traceweave-adoption-inputs.zh-CN.md)。
- 快照：branch `design/c2b`，HEAD **`64a8974`**；写本文件前工作树无改动；只读校验，只写本文件；未运行 Tauri/Playwright，未做真实 app/UI 验收。
- 方法：逐条复核 §5/§6/§8/§9/§10/§12/§13，并对照 `crates/poie-plan/src/sap.rs`/`plan.rs`、`crates/poie-whatif/src/lib.rs`、`crates/poie-state/src/review.rs`、`apps/desktop/src-tauri/src/db.rs`、`crates/poie-governance/src/lib.rs`；运行 docs/schema 检查与 `poie-plan` lib 测试。
- 结论：**CHANGES_REQUIRED**（5 个 C2b 阻断；未达到“零阻断 PASS”）。

## 已确认闭合

- actions 在 propose 构建/持久化、commit 不重建；`plan_to_actions_at` 返回 `Result` 且 `authorization:null`；C2b 不再新增 `validate_proposed_plan`，改用公开 `preview_plan_through_sap`（`sap.rs:194-277`）。
- `stateVersion` 明确 `host_assumption`；UI/proposal 标注；4→5 stale 测试写入 §13。
- C1 `mode=plan`/`outcome.plan=="planned"`/非空 actions → `not_promotable`；v6 两表、`PromotionCommit`/`ProposedAction` schemaVersion、basis 的 simulation/model、traceweave scope exception 表均已写入。

## 判定

1. **BLOCKER：重复 approve/commit 仍是“待选择”，不是已决定语义。** §9:171 原样保留“幂等或拒绝（明确选择）”，§13:223 只要求“幂等/拒绝一致”；§6.2:129 只决定了重复 propose。approve 重试、commit 成功后重试、已 consumed token 再 commit、reject/cancel 重试均无确定结果；approve/reject/cancel 也未声明 token+proposal 同事务，失败可留下 token 已变、proposal 未变。修正：明确选择“拒绝（`invalid_status`）”或“返回已存 commit”，用条件更新/事务覆盖 token+proposal，并写重试测试。

2. **BLOCKER：proposal 没有批准时的 snapshot/variant/result digest，commit 的 stale 检查无 baseline。** §5.1:65-72 只有 `planFingerprint/planSha256/modelSha256/stateVersion`；§8:155 要求 snapshot/variant/result 的 digest 与当前行一致。若行内 label/settings/variant JSON 改动但重跑 plan 指纹不变，commit 无法发现；`planSha256` 也不覆盖 C1 result settings。修正：把 `snapshotSha256`/variant digest/`resultSha256` 纳入 proposal 与 `proposalSha256`，commit 逐项比对并加“改行仍同指纹”测试。

3. **BLOCKER：动作时间窗基准与 C1 `nowS` 语义冲突，preview 无法复用同一时间窗。** §5.2:97 用 `valid_from=now+start_s*1000`；C1 `run_plan` 传 `now_s=snapshot.now_s`（`whatif lib:411`），`plan()` 以 `request.now_s` 为 cursor（`plan.rs:124/332`），故 `PlannedAction.start_s` 已含 `snapshot.now_s`；非零 nowS 会重复计时，wall-clock 下窗口错误。`preview_plan_through_sap` 仍用 `to_action` 的 `start_s*1000`（`sap.rs:90-101/211`），当 `AssumedContext.now_ms=当前时间` 时 time_window 直接判过期；设计未定义 AssumedContext 的 holds/reservations/predicates/now。修正：`now + (start_s - plan.start_s)*1000` 或强制 `nowS=0`；让 preview 使用同一批 now 重定动作，并写清 assumed 上下文。

4. **BLOCKER：proposal/action digest 可能循环，且 `ProposedActionDraft` 未定义。** §5.1:70 的 `actions:[ProposedActionDraft]` 没有 schema/字段；§5.2:92 的 `ProposedAction` 含 `promotionEvidence.approvalTokenId` 与 `proposalSha256`。按字面：`actionsSha256` 哈希 actions，`proposalSha256` 又包含 actions/actionsSha，token 的 `subject_digest` 再含 `proposalSha256`，而 action 里的 `proposalSha256`/tokenId 又需先有 proposal/token；无法同时定稿。修正：定义 draft（不含 evidence/token）并只对 draft 求 actionsSha；commit 物化完整 ProposedAction 时把 evidence 排除在 action/proposal digest 外，或明确“commit actionsSha 指 draft”。

5. **BLOCKER：审计/部分写未闭合，§12 与 §13 自相矛盾。** §12:215 声明“不声称审计链完整，只保证同一 DB 连接顺序”；§13:227 仍要求“审计链完整”。§9:172 的 commit 事务只含 token/actions/proposal，未含 `append_audit`；`append_audit` 是普通 INSERT（`db.rs:227`），提交后崩溃会留下 committed 但无审计事件。修正：把审计 INSERT 放入同一事务或明确 outbox，删除/改写 §13 的链断言，并加崩溃注入测试。

## REQUIRED-BEFORE（不单独算阻断，但施工前必须收口）

- `SapError`/`SapError::EmptyStateVersion` 未在设计中定义（源码只有 `PlanValidationError`/`PreviewError`）；`planSha256` 的 “canonical plan payload” 未钉到 C1 `{fingerprint, actions}`；`commitSha256` 无 domain/payload，[DIGEST](DIGEST.md):30 仍是 `poie.promotion-commit/0.1 | C2b`；`impact_json`/`citations_json` 及 token `session_id`/`policy` 取值仍未定义（表列 NOT NULL，`db.rs:106-107`）。
- `to_action` 把 `reversibility` 固定 `Idempotent`、`reservations` 置空（`sap.rs:108-109`），设计只说“复用字段映射”，未声明这是占位限制；SAP preview 的 `predicates_holding` 也未说明是假设，可能把 assumed 前置条件显示为 observed。
- v6：`db.rs:160` 仍 `user_version=5`、`db.rs:688` 仍断言 5；§10:195 只说“需先升到 6 + v5→v6 fixture”，未写替换旧断言、迁移事务/失败注入。
- 平台收口：仍无 `CommandError` 包络、`whatif_*` 的 window/command/session/policy 授权、具体 no-live 断言（`authorization=null`、不调用 `execution::snapshot`、命令注册表、仅两张新表）；C2b-7 仍只有 scope exception，未列 promote 最小能力/scope。

## 已执行检查

```text
python3 scripts/check-docs.py                     OK, exit 0
python3 scripts/check-schema-registry.py          35 schemas, exit 0
cargo test -p poie-plan --lib                     19 passed, 0 failed
git status --porcelain（写本文件前）              clean
```

边界：只读源码/设计；除本文件外未改任何文件；未跑 Tauri/Playwright/真实 app，因此 UI、迁移与 no-live 验收仍属设计审查而非运行证据。

VERDICT: CHANGES_REQUIRED
