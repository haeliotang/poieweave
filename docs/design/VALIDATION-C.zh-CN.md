# Phase C（C1）设计门禁终验（独立）

- 校验对象：[Phase C](phase-c-what-if.zh-CN.md)（下称 C）、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)、[README](README.zh-CN.md)；代码 `crates/poie-{ontology,plan,digest,sim}`、`apps/desktop/src-tauri`。
- 快照：branch `design/phase-c-c1`，HEAD **`d530fb6`**；写本文件前工作树干净。只读校验，未改源码/设计/git，只写本文件。
- 方法：审阅 `a857486..d530fb6` 全量 diff（6 文件）；逐项核对 C 与 PLATFORM/DIGEST/SCHEMAS/README；核对 `Plan`/`PlanError`/`GoalFailure`、`poie-digest`、`Patch`、`graph::Verdict`、`TimedScenario`/`AssetTimeline`、host 命令；运行 `check-docs.py`、`check-schema-registry.py`。未跑全量 cargo/Tauri/Playwright。
- 结论：**CHANGES_REQUIRED**（3 BLOCKER + 2 REQUIRED-BEFORE）。digest 换行/canonical、compare 契约、Playwright、deadline 归属确已修复；但 plan outcome、PLATFORM:32 与 DIGEST 域表、result digest 覆盖仍未收口。

## 五项判定

1. **BLOCKER — plan outcome 仍不可实现，且文档自相矛盾。** C:132 的映射写 `PlanError` 的 `NoCandidate`/`ActionBudget`/`Horizon`，但 `PlanError` 只有 `InvalidRequest`/`Model`/`Schedule`（plan.rs:80-84），全仓库无这三个变体。`action budget exhausted`/`horizon exceeded`/`unknown state`/`search limit` 是 `GoalFailure.reason`（plan.rs:151-166、306-310），由 `finish()` 以 `Ok(Plan)` 返回（plan.rs:333、344）；因此“成功 → planned”会把预算耗尽/超 horizon/未知的结果标成 planned，而错误映射永不触发。`InvalidRequest` 还包含 `max_actions==0`、`horizon<=now`（plan.rs:112-119），映射为 `infeasible` 会把调用方错误伪装成不可行；`Model`/`Schedule` 未映射；单 goal 也可能 actions 非空但 `goals_unmet` 非空（plan.rs:167、217），partial 语义未定义。C:214 仍写“plan failure 映射 `infeasible`”，与 C:132 直接冲突。必须改为按 `Plan.goals_unmet` 的 reason 派生 outcome，并给出 `PlanError` 三变体的处理。

2. **BLOCKER — 跨文档 scope 仍未统一。** PLATFORM:32 仍写“C 定义 `PatchDto`、`PerturbationDto`、...”，而 C:79 明确 C1 不含 Perturbation；该行在 `d530fb6` 未改（diff 只改 PLATFORM:88、194），按 PLATFORM 实现会为 C1 引入已移除的 timed 扰动 DTO。另：C:62 指向 DIGEST，但 DIGEST §1 域表仍无 `poie.snapshot/0.1`（只有 SCHEMAS:35 有）；`poie.variant/0.1` 行已改，`poie.snapshot` 行仍缺。PLATFORM:194、README:11、DIGEST:24 已修正，属实；但“C/PLATFORM 一致”不成立。

3. **BLOCKER — `resultSha256` 仍未绑定 outcome/actions，与声明不符。** C:104 的 payload 是 `{snapshotSha256, variant:{patch,goal,minVerification,deadlineS}, mode, settings}`；没有 outcome、actions、completionS、planFingerprint、planSha256。DIGEST:24 同样只列输入。因此它是“输入 digest”，不是结果 digest：同一输入下 outcome/计划内容变化（求解器修复、数据漂移、存储被改）不会改变 `result_sha256`。上轮阻断项未关闭；要么把 outcome/actions/planSha256 纳入 payload，要么改名（如 `inputSha256`）并声明“结果内容由 resultId/runId 标识”。另 `settings.deadlineS?`（C:94）与 Snapshot `deadlineS?`（C:54）未说明未设置时省略还是 null，DIGEST:60 要求显式 null，需在 C:62/104 写明，否则同一输入可有两个 digest。

4. **REQUIRED-BEFORE — compare 契约可再收紧。** CompareResult/schema/`mode`/`settings`/DB/`result_list`/v2→v3 迁移均已补齐（C:159-169、93-94、184、156、187；SCHEMAS:21），这是实质修复。但 C:171 把“跨 snapshot/重复 resultId”映射为 `invalid_patch`（与 patch 无关），应新增/改用 `invalid_request` 或 `cross_snapshot`；`baselineResultId` 不在 `resultIds` 内、`completionS` 为 null 时的排序、C:188“被 compare 引用”均未定义（compare 不持久化）。不阻断，但实现前需定稿。

5. **REQUIRED-BEFORE — auth/错误包络与 C2 清单仍需校准。** C:175 说复用“B0 CommandError 字符串前缀”，但 host 只有 `auth::require`，无 `CommandError`/`HostPolicy`/`authorize`（auth.rs:156、lib.rs:217-228）；PLATFORM:88 又从“会话 + policy”改成“会话 + auth”，与 PLATFORM §4 的 `authorize(..., policy)` 及 C:216 的 `scope_denied` 不一致。C2 清单：`TimedScenario::from_parts` 确实已公开（poie-sim/src/timed.rs:349）；但 `AssetTimeline` 仅 `asset` 为 pub、无 `shift`（poie-ontology/src/time.rs:72-78），`startMode`、`assetDelayS` 在代码中不存在（crates/apps 全无匹配），而 `CaseDoc.expect` 已是必填并参与比较（timed.rs:137-146、609-628）、evidence validity 也已支持（timed.rs:233-263），把它列为缺口不准确。

## 已核对正向

- Snapshot 结构已含 `snapshotSha256`（C:50），digest 公式含 `\n` 且用 canonical JSON（C:62/104），与 `sha256_of` 的 `domain + "\n" + canonical` 一致（poie-digest lib.rs:77-83）；排除 id/createdAt/createdBy/seed 已写明。
- `Patch` 13 字段与 C:78 完全一致（model.rs:813-847），`Model::patched` 存在（model.rs:726）。
- `OutcomeKind.structure` 四值 = `graph::Verdict` 四值（graph.rs:40-45）；C1 无 promote/timed 命令，`poie-whatif` 尚不存在（设计门禁，不作为失败理由）。
- `deadlineS` 只进 `PlanningGoal.deadline_s`（plan.rs:12-14），`PlanningRequest` 无 deadline（plan.rs:21-28）；C:131/133 与代码一致，`startS` 未被传入也已声明。
- Playwright 只测占位/禁用（C:197、220），不再要求表单校验；与“浏览器不执行 host”一致。

## 已执行检查

```text
python3 scripts/check-docs.py              OK, exit 0
python3 scripts/check-schema-registry.py   OK (26 schemas), exit 0
git diff --stat a857486..d530fb6           6 docs files
git status --porcelain                     clean before this report
```

边界：设计门禁；未跑全量 cargo/Tauri/Playwright；不以未实现代码作为失败理由，但以“设计与现有 API 不符”为阻断。

## 判定

四项声明中，digest 新行/排除项、compare/registry、Playwright、deadline 归属已实质修复；但 plan outcome 的变体名与机制仍错、PLATFORM:32 仍要求 PerturbationDto、DIGEST 仍缺 `poie.snapshot` 域且 result digest 不覆盖 outcome，故不满足“零 C1 阻断”。

VERDICT: CHANGES_REQUIRED
