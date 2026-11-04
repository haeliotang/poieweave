# Phase C（C1）设计门禁终验（独立）

- 校验对象：[Phase C](phase-c-what-if.zh-CN.md)（下称 C）、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)、[README](README.zh-CN.md)；代码 `crates/poie-{ontology,plan,digest,sim}`、`apps/desktop/src-tauri`。
- 快照：branch `design/phase-c-c1`，HEAD **`3f7181c`**；写本文件前工作树干净。只读校验，未改源码/设计/git，只写本文件。
- 方法：审阅 `a857486..3f7181c` 全量 diff（6 文件）；逐项核对 C 与 PLATFORM/DIGEST/SCHEMAS/README；核对 `Plan`/`PlanError`/`GoalFailure`、`graph::Verdict`、`poie-digest`、`Patch`；运行 `check-docs.py`、`check-schema-registry.py` 与 `poie-plan` 两个单测。未跑全量 cargo/Tauri/Playwright。
- 结论：**CHANGES_REQUIRED**（1 BLOCKER + 4 非阻断）。scope 与 result digest 两项已实质修复；plan outcome 的 reason 覆盖仍有洞，未满足“零 C1 阻断”。

## 判定

1. **BLOCKER — plan outcome 的 reason 映射仍不闭合（claim 1 只部分成立）。** C:133-138 只列出 6 个 `GoalFailure.reason`；`plan.rs` 实际还会产生 `"deadline missed"`（plan.rs:212）与动态 `"blocked: <root cause>"`（plan.rs:229-232），二者既不在三条映射里，也没有 else/default 或“其余 → …”规则。两者都可达并被 C1 自身暴露：C:37 的“截止提前”场景只改 `deadlineS`（C:131），base-01 上 `a_deadline_miss_is_reported` 已断言该 reason（plan.rs:413-420，实测通过）；`restore:lathe-1` 的 blocked 路径由 plan.rs:422-435 断言 reason 含 "blocked"（实测通过）。因此 `whatif_variant_run(mode=plan)` 在上述输入上没有定义 outcome；而 C:104 已把 `outcome` 纳入 `resultSha256`，outcome 未定义会导致 digest 不可定义。→ 至少补 `"deadline missed"`/`"blocked:*"` → `infeasible`（或显式 fallback），并加对应映射测试。已确认其余部分已修正：不再虚构 `NoCandidate/ActionBudget/Horizon`，`Ok(Plan)` 且 unmet 非空不再算 planned，优先级存在，C:220 与 C:132 不再冲突。

2. **REQUIRED-BEFORE（非阻断）— `InvalidRequest` → `invalid_patch` 语义错位。** `InvalidRequest` 的三种来源是 `horizon<=now`、空 goals、`max_actions==0`（plan.rs:112-119），来自 Snapshot/请求参数而非 patch；PLATFORM:56 已同时提供 `invalid_parameter`，而 C:125-126 的 snapshot 创建并未校验这三项。建议映射为 `invalid_parameter`（或在 snapshot 创建时拒绝），否则坏快照会以 patch 错误报出。

3. **REQUIRED-BEFORE（非阻断）— 可选字段 null/省略仍未写死。** C:54、C:94 用 `deadlineS?`，C:96/104 用 `planSha256?`/`completionS?`，但未说明“未设置时写 null 还是省略”；DIGEST §2.6（DIGEST.md:60）要求可选字段缺省为 `null`、不得时而省略。同一输入因此仍可能有两个 canonical/digest。要么去掉 `?` 明确 null，要么在 C:62/104 引用 DIGEST §2.6。

4. **REQUIRED-BEFORE（非阻断）— `planSha256` 无可执行定义。** C:96/104 把 `planSha256` 纳入结果 digest，但 C 未定义其域与公式（DIGEST:22 只有 `poie.plan/0.1` 一行），也未写 `planFingerprint` 与 `planSha256` 的关系（DIGEST §1.1 要求 fingerprint 作为字段再 hash）。实现前需定稿，但不影响 C1 结构/计划主路径。

5. **REQUIRED-BEFORE（非阻断）— compare/auth 残留未收口。** C:178 仍把跨 snapshot/重复 resultId 映射 `invalid_patch`（与 patch 无关）；PLATFORM:88 的 `whatif_*` 仍是“会话 + auth”，与 PLATFORM §4 的 `authorize(window,command,session,policy)`（:94）及 `HostPolicy`（:97）不一致；C:182 错误码未列 `invalid_parameter`/`stale_snapshot`。均不阻断 C1 主流程。

## 已核对正向

- `PlanError` 三变体与 plan.rs:80-84 一致；C:132 的三变体处理可实现；`deadlineS` 只进 `PlanningGoal.deadline_s`（plan.rs:12-14，`PlanningRequest` 无 deadline），C:131/140 正确。
- 跨文档 scope 已统一：PLATFORM:32 移除 `PerturbationDto` 并归 C2；C:79/C:16 与之一致；DIGEST:24 新增 `poie.snapshot/0.1`，与 C:62 的排除项（snapshotSha256/id/createdAt/createdBy/seed）一致；SCHEMAS:35-36 同步。
- `resultSha256` 已真正绑定结果：C:104 payload 含 `outcome/actions/rootCauses/externalDependencies/planSha256?/completionS?`，DIGEST:25 同步；digest 公式 `domain "\n" canonical_json` 与 `poie-digest::sha256_of` 一致（lib.rs:77-83）；C:219“两次运行同 digest”在确定性下成立。
- `Patch` 13 字段与 model.rs:813-847 一致；`structure` 的 `blocked/candidate/unknown/search_limit` 对应 `graph::Verdict` 四值；Playwright 只测占位/禁用（C:196、C:227），与 host-only 一致。

## 已执行检查

```text
python3 scripts/check-docs.py              OK, exit 0
python3 scripts/check-schema-registry.py   OK (26 schemas), exit 0
cargo test -p poie-plan --lib a_deadline_miss_is_reported   ok（reason = "deadline missed"）
cargo test -p poie-plan --lib the_planner_restores_a_failed_asset_before_re_solving  ok（"blocked: …"）
git status --porcelain                     clean before this report
```

边界：设计门禁；不以“C1 代码未实现”为失败理由，只以设计与现有 `poie-plan` API 不符/不可实现为阻断。

VERDICT: CHANGES_REQUIRED
