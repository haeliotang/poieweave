# Phase C（C1）设计门禁终验（独立）

- 校验对象：[Phase C](phase-c-what-if.zh-CN.md)（下称 C）、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)、[README](README.zh-CN.md)、[Phase B](phase-b-agent-governance.zh-CN.md)；代码 `crates/poie-{ontology,plan,sim,digest,objects}`、`apps/desktop/src-tauri`、`apps/desktop/src`。
- 快照：branch `design/phase-c-c1`，HEAD **`a857486`**；写本文件前工作树干净。只读校验，未改源码/设计/git，只写本文件。
- 方法：审阅 `69987ca..a857486` 全量 diff；逐项核对 C 与 PLATFORM/DIGEST/SCHEMAS/README；核对 `Patch` 13 字段、`graph::Verdict`、`poie-plan`、`poie-digest`、host 命令/DB；运行 `check-docs.py`、`check-schema-registry.py`。未跑全量 cargo/Tauri/Playwright。
- 结论：**CHANGES_REQUIRED**（4 BLOCKER + 2 REQUIRED-BEFORE）。Rescope 确实把 Perturbation/timed/promote 移出 C1，旧报告的 B1/B3/B4 不再作为 C1 阻断；但 C1 自身的 digest、跨文档 scope、compare/registry、plan outcome 仍未收口。

## 五项判定

1. **BLOCKER — Snapshot/result digest 既未冻结也不符合 DIGEST。** C:60 写成 `sha256("poie.snapshot/0.1" + canonical_json(snapshot_without_sha))`，缺少 DIGEST:13 强制的 `"\n"`；`poie-digest::sha256_of` 实际是 `domain + "\n" + canonical_json`（lib.rs:77-83），按 C:60 手写与调用现有函数会得到不同字节。C:96 的 result digest 是 `"poie.variant/0.1" + snapshotSha256 + canonical variant + canonical settings` 的字符串拼接，不是单一 canonical payload，`settings` 未定义；DIGEST:24 的 `poie.variant/0.1` 仍写 `perturbation`，`poie.snapshot/0.1` 只登记在 SCHEMAS:34，DIGEST 域表没有这一行。Snapshot 结构 C:47-55 没有 `snapshotSha256` 字段，C:60/153 却使用 `snapshot_without_sha` 和 DB `snapshot_sha256`，排除项只列 id/createdAt/createdBy/seed；Variant C:66-72 的 id/createdAt/createdBy 未声明排除，但 C:96 要 hash "canonical variant"，违反 DIGEST:60-61（不把时间戳/随机 ID 放入 canonical payload，可选字段显式 null）。结果 digest 只覆盖输入，不覆盖 outcome/actions/planSha256；在冻结 payload、排除项、换行和 domain 之前，"同输入同 digest"（C:183）不可判定。seed 被排除且无消费者，也不应留作语义输入。

2. **BLOCKER — 共享平台文档仍把 C1 定义成旧 Phase C，跨文档 scope 冲突。** PLATFORM:194 仍写 `C1 snapshot/perturbation/outcome -> C2 compare/promote/checkpoint`；C 的 C1 已明确无 perturbation，却在 C:142/150-155 提供 compare/result_get 与 whatif 表。PLATFORM:32 仍要求 C 定义 `PerturbationDto`，README:11 仍写 C = "快照、扰动、模拟、比较、提升为 proposed plan"、依赖 "plan/sim"，DIGEST:24 仍以 perturbation 为 variant payload；这四处与 C:1-5,76,142 直接矛盾。按 PLATFORM §8 实现会漏掉 compare（C 验收 #1），按 C 实现又会让 PLATFORM/DIGEST 的 digest 契约无对应。另：C 的 Snapshot/Variant/VariantResult 只写 `schemaVersion`（C:48/67/88），没有 PLATFORM:9-12 与 SCHEMAS:53 要求的 `schema` 字符串；C1 的 `CompareResult` 未在 SCHEMAS:18-20 登记。HEAD 提交的 VALIDATION-C 仍是针对 `69987ca` 的旧报告（C:83-89/204），不是 C1 门禁证据；本次已替换。

3. **BLOCKER — Compare/result registry 契约不足以实现。** C:142 的 `whatif_compare` 返回 `CompareResult`，但 C 未给字段，SCHEMAS:18-20 未登记 compare schema/domain；C:124 只写按 outcome/完成时间/resultId 排序，未定义 structure 与 plan 的跨模式比较、空/重复/跨 snapshot 的 resultIds、baseline 缺失，以及“全部差异”的类型（delta/元素 schema 未定义）。C:87-93 的 VariantResult 没有 `mode`/`settings` 字段，C:96 却要求 "canonical settings"；DB C:150-155 也不存 mode/settings，`whatif_result_get` 无法重算 digest，`compare` 无法区分同一 variant 的 structure/plan 结果。C:143 有 `result_get`，但没有 result list 命令；持久化结果在重启后只能靠前端记住 resultId，C:185 的“持久化恢复”没有 API 路径。C:158 的 v3 迁移只说“只加表/失败回滚”，未给 v2→v3 幂等步骤；实际 `db.rs:122` 无条件写 `PRAGMA user_version=2`，C0 必须先改。`actions/rootCauses/externalDependencies` 的元素类型也未定义（C:91）。

4. **BLOCKER — plan outcome 会把有界/未知误标成 infeasible，且 API 与文档不符。** C:84 有 infeasible/unknown/search_limit 四值，C:184 却要求 "plan failure 映射 infeasible"；`Plan` 的失败原因包含 `action budget exhausted`、`horizon exceeded`、`unknown state`、`search limit`（plan.rs:151-166、306-310），全部映射为 infeasible 会与 C:28 的 caveat（不声称物理不可行）冲突，并让 unknown/search_limit 永远不可达。C:122 写 "plan with horizon/now/deadline/maxActions"，但 `PlanningRequest` 只有 now/horizon/goals/min_verification/max_actions（plan.rs:21-28），deadline 在 `PlanningGoal.deadline_s`（plan.rs:10-14），cursor 从 `now_s` 起（plan.rs:124）；C1 没有 `startS` 消费者。Snapshot `deadlineS?` 与 Variant `deadlineS|null`、goal/minVerification 的覆盖优先级未定义（C:51-53 对 69-70），variant run 的 mode 也未持久化。需要 plan 失败→outcome 的映射表、deadline 归属/优先级，以及 PlanError→稳定错误码。

5. **REQUIRED-BEFORE — Playwright/auth 与 C2 前置仍有不实或缺口。** Playwright：C:168 说浏览器只显示 host-only 占位、不调命令，C:191 却要测 "variant 表单校验（浏览器不执行 host）"；占位页没有表单，二选一。Auth/scope：C:146/187 承诺 unauthorized/scope_denied，PLATFORM:88 要求 whatif_* 会话+policy，PLATFORM:94 要求 `authorize(window, command, session, policy)`，但 host 现只有 `auth::require`、命令返回 `Result<_, String>`，没有 HostPolicy/CommandError（lib.rs:217-223），C:176 的 C0 顺序未列此项。C2：把 timed/storage/promote 延后并声明重跑门禁，方向是诚实的；但 C:204 说需要 "TimedScenario 公共构造"，实际 `from_parts` 已公开（timed.rs:349）且字段 pub；真正缺的是 `AssetTimeline` shift/公开 field（time.rs:72-78）、startMode、storage capacity/initial/maxCharge 与 `assetDelayS` 只移修复/不移故障的语义；C2 还漏了 `TimedRunner` case 的必填 `expect` 和 evidence validity。补正 C2 清单，避免把已有 API 当缺口、把真实语义坑留到实现。

## 已核对正向

- `Patch` 13 字段与 C:68/75 完全对应（model.rs:813-847）；`Model::patched` 存在（model.rs:726），C1 的结构求解与 plan 均可基于独立 Model 副本实现。
- `OutcomeKind.structure` 四值 = `graph::Verdict` 四值（graph.rs:40-46）；C1 API 无 timed/promote/approve 命令，C1 不依赖 poie-sim/state/viability。
- “不写 live/SAP/federation”的实现基础成立：host 命令表无 `sap_*`，`AppState.model` 为只读 `Arc`，C1 只新增 whatif 表；C2 对 timed/promote 的重跑门禁方向正确。

## 已执行检查

```text
python3 scripts/check-docs.py              OK, exit 0
python3 scripts/check-schema-registry.py   OK (25 schemas), exit 0
git diff --name-only 69987ca..a857486      4 docs files
git status --porcelain                     only this report after write
```

边界：设计门禁；`poie-whatif` 尚不存在，未跑全量 cargo/Tauri/Playwright；不以未实现代码作为失败理由。

## 判定

Rescope 的方向正确，并确实消除了旧的 Perturbation/timed/promote C1 阻断；但 C1 仍有 4 个阻断（digest、跨文档 scope、compare/registry、plan outcome），不满足“零 C1 阻断”。

VERDICT: CHANGES_REQUIRED
