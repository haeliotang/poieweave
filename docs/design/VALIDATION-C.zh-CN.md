# Phase C（C1）设计门禁终验（独立）

- 校验对象：[Phase C](phase-c-what-if.zh-CN.md)（下称 C）、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)、[README](README.zh-CN.md)；代码 `crates/poie-plan`、`crates/poie-ontology`、`crates/poie-digest`。
- 快照：branch `design/phase-c-c1`，HEAD **`8d96568`**；写本文件前工作树干净。只读校验，未改源码/设计/git，只写本文件。
- 方法：审阅 `3f7181c..8d96568` 全量 diff；逐条核对 C §5.1/§5.3/§6.1/§7/§8 与 `plan.rs` 的 `GoalFailure.reason`、`PlanError`、`Plan::digest()`；核对 DIGEST/SCHEMAS/PLATFORM；运行 `check-docs.py`、`check-schema-registry.py` 与 `poie-plan` 全量 lib 测试。
- 结论：**PASS（零 C1 阻断）**。上轮唯一阻断已闭合；4 项声明均已落实，剩余为不影响 C1 开工的措辞/施工细节。

## 判定

1. **上轮阻断已闭合：reason 映射覆盖 `plan.rs` 全部 8 种原因，且有默认分支。** `plan.rs` 只产生 `action budget exhausted`（151/155/252/257）、`horizon exceeded`（166）、`deadline missed`（212）、`blocked: {root_cause_text}`（229–231）、`no restorable blocker`（301）、`unknown state`（306）、`search limit`（310）、`no plan found`（316）；C:136–141 逐类覆盖（预算耗尽/搜索上限 → `search_limit`，horizon/no plan/no restorable/`deadline missed`/`blocked:` → `infeasible`，未识别 → `unknown`），声明 `Ok(Plan)` 且 unmet 非空不得记 `planned`（C:142），并给出 unknown > search_limit > infeasible 优先级。两个新原因均可达且实测通过：`a_deadline_miss_is_reported`（reason = `deadline missed`）、`the_planner_restores_a_failed_asset_before_re_solving`（reason 以 `blocked:` 开头）。C:223 的验收与 §6.1 一致。
2. **其余 3 项声明成立。** (a) `InvalidRequest` → `invalid_parameter`（C:134，与 PLATFORM:56 一致），不再误报 `invalid_patch`，并以“Snapshot 创建时先校验 horizon/max_actions/goals”避免把调用方错误当业务结论。(b) 可选字段缺省写 `null`、不得省略，并引用 DIGEST §2.6（C:104）；`planSha256` 公式（C:105）与 DIGEST:22 的 `poie.plan/0.1`、`Plan::digest()`（plan.rs:62–76）及 `poie-digest::sha256_of` 的 `domain+"\n"+canonical` 一致，fingerprint 与 sha256 分列保留；`resultSha256`（C:106）覆盖 outcome/actions/planSha256/completionS，与 DIGEST:25 同步。(c) compare 跨 snapshot → `invalid_parameter`、重复 resultId 去重并告警（C:181）；PLATFORM:88 的 `whatif_*` 已改为 `authorize(window,command,session,policy)`。
3. **非阻断残留（首个 C1 PR 一并收口，不改设计契约）。** ① C:134 声称快照创建校验 horizon/max_actions/goals，但 §5.1 与 C:127 第 1 步未列这三项；run 侧已兜底为 `invalid_parameter`，无行为歧义。② C:181 要求“记录 warning”，但 CompareResult（C:171–179）无 `warnings` 字段；跨 mode 时必填 `delta` 未定义取值。③ C:185 错误码清单未列 `invalid_parameter`（C:134/181 已明确使用）。④ `snapshot.maxDepth` 未写进结构求解 `Query.max_depth`（graph.rs:32–36）；单 goal 下 `PlanningGoal.priority` 取值未定义（无语义影响）；`seed` 未使用且被排除出 digest。
4. **核对与边界。** `Patch` 13 字段与 model.rs:813–847 一致；`Verdict` 四值与 structure outcome（C:87）一致；4 个 `poie.whatif-*` schema 已在 SCHEMAS 登记；DB 迁移 2→3 与 db.rs:122 的 `user_version=2` 衔接；Playwright 仅测占位/禁用（C:196、C:227）。设计门禁不以 C1 未实现为失败；未跑全量 cargo/Tauri/Playwright。

## 已执行检查

```text
python3 scripts/check-docs.py                     OK, exit 0
python3 scripts/check-schema-registry.py          26 schemas, exit 0
cargo test -p poie-plan --lib                     19 passed, 0 failed
git status --porcelain（写本文件前）              clean
```

边界：只读校验；除本文件外未改任何文件。

VERDICT: PASS
