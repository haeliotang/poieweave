# Phase C2b 设计门禁（独立）

- 校验对象：[phase-c2b-promote](phase-c2b-promote.zh-CN.md)，以及 `poie-plan::sap`/`plan`、`poie-state::action`/`review`、B1 `approval_tokens`、host `whatif.rs`/`db.rs`、[PLATFORM](PLATFORM.zh-CN.md)/[DIGEST](DIGEST.md)/[SCHEMAS](SCHEMAS.md)、[traceweave 输入清单](traceweave-adoption-inputs.zh-CN.md)。
- 快照：branch `design/c2b`，HEAD **`8991fa2`**；写本文件前工作树无改动；只读校验，只写本文件；未运行 Tauri/Playwright，未做真实 app/UI 验收。
- 方法：逐条对照设计 §4/§5/§6/§8/§9/§10/§13 与源码；复核 C1 `mode=plan` 的 result 字段、`Plan::digest`/`planSha256`、SAP 七项 review、`ActionLedger` 状态表、token 表与迁移；运行 docs/schema 检查及 `poie-plan` lib 测试。
- 结论：**CHANGES_REQUIRED**（4 个 C2b 阻断、2 组施工前必改；未达到“零阻断 PASS”）。

## 判定

1. **BLOCKER：审批主体没有闭合到动作集合，commit 还可能提交已过期动作。** [设计](phase-c2b-promote.zh-CN.md):61-75 让 `proposalSha256` 绑定 `actionsSha256`，但 §4:46-51 把“build proposed actions”放在 commit；若动作在 commit 才生成，批准时 `proposalSha` 无法绑定动作，commit 实际写入的不是被批准的动作集；若在 propose 生成，设计必须明确写出，并让 commit 重算后与 proposal 的 `actionsSha256` 严格相等。仅比较 `planFingerprint` 不够：`Plan::digest()`（`crates/poie-plan/src/plan.rs`:64-76）不含 `preconditions`/`allocations`/`occupies`/`verification`，C1 `plan_sha256`（`crates/poie-whatif/src/lib.rs`:450-461）也沿用同一窄 `ActionView`；模型/patch 变更可保持 fingerprint 而改变预算或前置条件。另 `to_action`（`crates/poie-plan/src/sap.rs`:90-101）的 `validFrom=start_s*1000`、`validUntil=end_s*1000+60s` 来自固定 snapshot，commit 只检查 token 过期（§8:147），不检查动作时间窗，直接违反目标 4“批准时有效、提交时已过期不允许”。修正：propose 生成动作并计算 `actionsSha256`，proposal 绑定之；commit 重算并比对，且以 `now_ms` 检查每个动作 `validFrom/validUntil`；失败返回 `not_promotable`/`stale_result`。

2. **BLOCKER：`validate_proposed_plan` 的声明自相矛盾，无法产出 `sapPreview.checks/rejections/assumed`。** §6.1:103-108 说只做结构校验、`ActionLedger` add/apply `Validated`，不调用 `review_dispatch`；但 §5.1:68 和 §11:187 要求 checks/rejections/assumed，UI 要展示 SAP 检查。`review_dispatch` 并非“需要 Authorization 才能调用”：它在 `Validated` 动作上可运行，只是 `authorization=None` 会产生 `missing_authorization`（`crates/poie-state/src/review.rs`:183-188），而 `preview_plan_through_sap` 已经公开实现七项检查、`assumed` 与 `authorization_created=false`（`crates/poie-plan/src/sap.rs`:194-276）；host `execution.rs`:95-121 也已在 `Validated` 上跑 review。修正：要么复用/参数化 preview 并显式列出 assumed（把 authorization 标为 assumed），要么删除 `sapPreview` 的 checks/rejections 承诺；后者需同步 UI/验收。另 `plan_to_actions` 的 `Vec<Action>` 签名无法表达空 `state_version`/空动作错误，建议改为 fallible 或要求调用方先校验；并明确该映射把 reversibility 固定为 `Idempotent`、reservations 为空（`sap.rs`:93-117）是占位限制，不能当作真实 SAP 语义。

3. **BLOCKER：`stateVersion` 没有权威来源，C2b 的 stale 检查会退化为硬编码。** §5.1:76 只说“host 明确提供（MVP：`{"site-a":4}`）”，但 host 当前没有状态版本存储：`poie-state::projection` 需从记录投影，而 desktop host 没有这些记录；`execution.rs`:83/112 与 `agent.rs`:499 的 `site-a:4` 是演示/测试常量，不是可刷新状态。于是 propose/commit 的 `state_version` 可任意给值，§8:146 的“state_version 与 proposal 一致”只证明两次调用传了同一常量，不能证明站点状态未漂移，也无法验证动作 precondition。修正：定义唯一来源（host 状态存储/投影结果/显式导入），或把它降级为 `assumedStateVersion` 并与 observed 分栏；UI/审计必须标注 assumed，测试至少用两个值模拟漂移并期望拒绝。

4. **BLOCKER：token 绑定、幂等和部分写仍未定义到可实施。** (a) §6.2:112-120 把 `subject_digest = proposalSha256`，但 B1 的 `subject_digest` 是 `poie.approval/0.1` 对 tool/args/session/policy/expiry 的绑定（`crates/poie-governance/src`:364-375）；C2b 未定义 `session_id`、`policy_version`、`impact_json`、`citations_json` 的取值，也未规定 approve/commit 必须校验 `tool=="whatif.promote"`、`run_id==proposalId`、`call_id`、`subject_digest` 重算、scope 与 token 归属。跨 proposal 复用 token、重复 propose 为同一 result 建多个 proposal/token 均未被禁止；表上也没有 `approval_token_id` 唯一约束。(b) §9:160 明写“幂等或拒绝（明确选择）”，这是未决设计；重复 propose/approve/commit、commit 成功后的重试、已 consumed token 的再次 commit 都没有确定语义。(c) §9:161 “同一事务”没有覆盖 `append_audit`（`db.rs`:227-233 只是普通 INSERT，且不写 `prev_hash/hash`），也不存在事务级 `save/transition` API；崩溃在 audit 前会留下 committed 但无审计。(d) Platform §6 的 token 状态含 `interrupted`，C2b 只用 `approved→executing→consumed`，未定义 executing 中断/重启的 `outcome_unknown`；approve 后的过期、懒过期（get/list 时标 expired）、`decidedBy`/session 身份也未写。修正：选定幂等规则，给 propose/commit 稳定幂等键；在 approve/commit 重算并校验 subject/scope/tool/run/session；把状态、动作、token、审计纳入同一事务或明确 outbox；补齐 interrupted/expired 恢复测试。

5. **REQUIRED-BEFORE：v6 迁移、schema/digest 和审计链未闭合。** `db.rs`:160 仍在建表批次末尾写 `PRAGMA user_version = 5`，测试 `db.rs`:687 也断言 5；设计 §10:178 要求 6，却未说明替换版本写法、从已填充 v5（C1/C2a 行）迁移、幂等重跑与失败回滚的测试。`ProposedAction`（§5.2:81-89）缺 `schemaVersion`，`PromotionCommit` 只有 §7 表名、无 schema；`proposalSha`/`actionsSha` 未定义 DIGEST §1 的 domain+payload（`DIGEST` 已登记 `poie.promotion/0.1`/`poie.proposed-action/0.1`），`planSha` 的“canonical plan payload”也未按 C1 的 `{fingerprint, actions}` 写死。§5.1 没有 `resultSha256`/`snapshotSha256` 字段，但 §8:144 要求“digest 与当前行一致”，proposal 中没有可比较的批准时 baseline。审计事件依赖 `append_audit`，其行不写 `hash/prev_hash`，因此 §12:196 的“审计链完整”目前不可验证。修正：补 v5→v6 fixture、完整 payload 定义、schemaVersion/commit schema，以及审计链或明确的“非链式审计”边界。

6. **REQUIRED-BEFORE：C1 可提升性/错误码/隔离验收/延后项需要收口。** §8:142 只要求 `mode=plan`，未要求 `outcome.plan=="planned"` 且 `actions` 非空；C1 的 `plan_outcome`（`whatif` lib:480-496）会产出 `infeasible`/`unknown`/`search_limit`，空或不可行计划仍可 propose/approve。错误码未对齐 PLATFORM §3：缺 `not_promotable`、`approval_mismatch`，把 `scope_denied` 写成 `scope_mismatch`，也未采用 `CommandError` 包络；PLATFORM §4 要求 `whatif_*` 走 `authorize(window,command,session,policy)`，设计 §7 没有 window/scope/policy。§13 的“无 live 写”没有可观测断言：host 没有 live SAP 表，应至少断言 proposed action `authorization=None`、不调用 `execution::snapshot`/不产生 SAP 记录、未注册 dispatch/authorize 命令、DB 只新增两张表。延后项：timed 不可 promote 是诚实的（C2a 结果无完整 actions）；长期 grant/revoke 的“留到 D2”也诚实，但与 traceweave C2b-1/2/3/7 的 P0“必须进设计”冲突，需明确 gate scope exception 与 D2 owner，或补一次性 token 的 operator/session 绑定与 approved 后 revoke/cancel；`promotionEvidence.basis` 只有 `approval:<tokenId>`，未按 PLATFORM §5.2 编码 `simulation=<branch>;model=<modelSha256>`，未来 Authorization 仍不可追溯。**OPTIONAL**：仅 C2b-8 授权/撤销 UI 可按 traceweave 清单后置；C2b-7 capability inventory 属清单 C2b-1..7 的 P1“必须进设计”，不能以 OPTIONAL 跳过，至少要在设计中列出 promote 的 scope/最小能力边界并标注未实现。

## 已执行检查

```text
python3 scripts/check-docs.py                     OK, exit 0
python3 scripts/check-schema-registry.py          34 schemas, exit 0
cargo test -p poie-plan --lib                     19 passed, 0 failed
git status --porcelain（写本文件前）              clean
```

边界：只读源码/设计；除本文件外未改任何文件；未跑 Tauri/Playwright/真实 app，因此 UI、迁移与 no-live 验收仍属设计审查而非运行证据。六项门禁清单中仅“timed 结果不可 promote”边界基本成立，其余五项均需上述修订后才能 PASS。

VERDICT: CHANGES_REQUIRED
