# Phase C2b 设计门禁（独立复验）

- 校验对象：[phase-c2b-promote](phase-c2b-promote.zh-CN.md)，以及 `poie-plan::sap`/`plan`、`poie-state::review`/`action`、B1 `approval_tokens`、host `whatif.rs`/`db.rs`、[PLATFORM](PLATFORM.zh-CN.md)/[DIGEST](DIGEST.md)/[SCHEMAS](SCHEMAS.md)、[traceweave 输入清单](traceweave-adoption-inputs.zh-CN.md)。
- 快照：branch `design/c2b`，HEAD **`2e714e6`**；写本文件前 `git status --porcelain` 为空；只读校验，只写本文件；未运行 Tauri/Playwright，未做真实 app/UI 验收。
- 方法：逐条复核 64a8974 的 5 个阻断与 2e714e6 的修订，对照设计 §5–§13 与 `sap.rs`/`plan.rs`/`review.rs`/`whatif lib.rs`/`db.rs`/`governance lib.rs`/PLATFORM；运行 docs、schema 与两个 lib 测试。
- 结论：**CHANGES_REQUIRED**（5 个 C2b 阻断；未达到“零阻断 PASS”）。

## 前一轮 5 项状态

- 1（幂等）：§9:191–204 已给明确语义表、token+proposal 同事务、commit audit 入事务；但 commit 结果无表可存（B1）。
- 2（baseline）：§5.1:65/81 已纳入 result/snapshot/variant/inputs SHA 并要求 commit 比对；`variantJsonSha256` 的计算域仍未定义（B4）。
- 3（时间基）：§5.2:102–106 已用 `(start_s-origin_now_s)` 修正双重计时，但公开 API 与 `origin_now_s` 矛盾（B2）；preview 的 assumed 上下文会自拒（B3）。
- 4（digest 循环）：draft/ProposedAction 拆分已消除 proposal→token→action 的循环；`actionSha256` 仍无公式（B4）。
- 5（审计）：§9:201 与 §12:246 已一致，commit audit 同事务、§13 不再宣称链完整；仅缺崩溃注入测试。

## 阻断

1. **BLOCKER：`promotion_commits` 表未定义，commit 幂等与 commitSha 无法持久化。** §9:201 要求事务写 `promotion_commits`，§9:198 要求重复 commit “返回已存 PromotionCommit”；但 §10:215–222 的 v6 DDL 只有 `promotion_proposals` 和 `promotion_proposed_actions`。仓库中也只有这一处引用该表。`PromotionCommit` 的 proposalId/planSha/actionsSha/token/inputsSha/commitSha 无列可落；commit 重试、`commitSha256` 校验和 §13 的 commit 验收均不可实现。修正：补 `promotion_commits` DDL/索引（或在 proposal 行明确同事务存储 commit 的字段）、加入 v6 迁移与 fixture，并把 §10 的“只加表”数量改为三张。

2. **BLOCKER：`plan_to_actions_at` 的公开签名与 §5.2 的四参数调用矛盾，时间基仍不能唯一实现。** §5.2:101 写 `(plan, origin_now_s, proposal_now_ms, state_version)`，§6.1:117 却写 `(plan, now_ms, state_version)`；`origin_now_s` 未定义（`plan.start_s`？`snapshot.now_s`？），`now_ms` 也未被指明是 proposal 时间。若按 §6.1 实现，§5.2 的 `(start_s-origin_now_s)` 公式没有输入；若按 §5.2 实现，§6.1 的公共 API 不成立。C1 `run_plan` 以 `snapshot.now_s` 为 cursor（`whatif lib.rs:411`、`plan.rs:124/332`），因此必须钉死 `origin_now_s == plan.start_s == snapshot.now_s`，并让两处签名一致；否则 64a8974 的“双重 now”阻断会以另一种形式回来。

3. **BLOCKER：`preview_actions` 的 assumed 上下文会把所有正常计划判成 `sap_rejected`。** §6.1:124 固定 `holds_command_ownership:false`、`reservations_held:false`、`predicates_holding:{}`。`review_dispatch` 对 false/空集合分别产生 `no_command_ownership`、`reservations_not_held`（`review.rs:128–160`）和 `stale_precondition`（`review.rs:113–126`）；而 §6.1:125 只说 `missing_authorization` 是预期项、其余拒绝 → `sap_rejected`。C1 的计划动作带有 `preconditions`（`plan.rs:341–354`），所以“propose 合法 plan result”会失败，§13:251 的正常路径无法通过。修正：把 ownership/reservations/preconditions 明确列为 assumed（并展示为 assumed），或把 `predicates_holding` 设为计划动作的 preconditions；只有真正的结构/时间窗拒绝才映射 `sap_rejected`。

4. **BLOCKER：digest 仍未闭合：planSha 复用 C1 域但 payload 不同，`actionSha256` 无定义，部分域缺少 `\n`。** §5.1:78 的 `planSha256` 使用 `poie.plan/0.1`，但 C1 `run_plan` 已用同一域对 `{fingerprint, actions}` 计算并存入 `result.plan_sha256`（`whatif lib.rs:461`）；两个 payload 产生同名/同域的不同 digest，审批、commit 与 C1 result 无法互相校验。§5.2:95 的 `actionSha256` 没有 domain/payload；`variantJsonSha256`（§5.1:65）也未定义计算域；§5.1:77 用 `poie.proposed-action/0.1` 哈希 drafts（DIGEST 另有 `poie.proposed-action-draft/0.1`）；§5.1:81 与 §10:225 的 `promotion-inputs`/`promotion-commit` 公式还漏了 DIGEST §1 的 `"\n"`。修正：给 C2b 计划 digest 独立域（或改用 C1 的 plan_sha）、定义单动作 digest 与 draft 域、按 `domain + "\n" + canonical` 统一 inputs/commit，并补固定向量。

5. **BLOCKER：平台授权行把 `whatif.promote` 当 window，且 no-live 断言仍不可执行。** §7:167 写 `authorize(window="whatif.promote", command, session, policy)`；PLATFORM §4 的 window 标签是 `main`/`poie-view-plan`，`whatif.promote` 是 token 的 tool/scope（§6.2:134），不是 window。按字面实现会 scope_denied。`no_live_write` 仍只有“测试断言无 SAP/ledger/federation 写函数调用”一句，未落下 `authorization=null`、不调用 `execution::snapshot`、命令注册表和仅新增哪些表的可执行断言；`CommandError` 也未映射现有 `whatif_*` 的 `Result<_, String>`。修正：`authorize(window="main"|"poie-view-plan", command="whatif_promote_*", ...)`，另设 policy scope `whatif.promote`，并列出上述 no-live 断言。

## 已执行检查

```text
python3 scripts/check-docs.py                     OK, exit 0
python3 scripts/check-schema-registry.py          37 schemas, exit 0
cargo test -p poie-plan --lib                    19 passed, 0 failed
cargo test -p poie-whatif --lib                   7 passed, 0 failed
git status --porcelain（写本文件前）              clean
```

边界：只读源码/设计；除本文件外未改任何文件；未跑 Tauri/Playwright/真实 app，因此 UI、迁移与 no-live 验收仍属设计审查而非运行证据。

VERDICT: CHANGES_REQUIRED
