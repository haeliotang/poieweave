# Phase C2a 设计：时间版/容量 What-if（独立 C2a 模型）

- 状态：设计草案（待 C2a 门禁）
- 日期：2026-11-05
- 范围：timed/storage 变体与运行；**独立 C2a DTO/DB/命令，不扩展 C1 类型**
- 明确延后 C2b：promote、assetDelay、checkpoint、patch-on-timed
- 上游：C1、`poie-sim::timed`、`poie-ontology::time`、B1 登录会话

## 1. 背景

C1 只做静态 structure/plan。C2a 增加时间版推演：储能放电限制、截止提前、startMode。为避免改变 C1 已持久化结果和 `poie.variant/0.1` digest，C2a **不复用 C1 的 Snapshot/Variant/Result 类型**，使用独立 `timed.*` 契约与表。

## 2. 目标

1. Timed Snapshot 绑定 model digest + scenario raw-file digest + start/horizon/goal。
2. Timed Variant 只允许 `startS`、`deadlineS`、`storageDischargeW` 覆盖；不应用 PatchDto（C2b）。
3. Timed Runner 返回 TimedVerdict/completion/power_limited/capacity_conflict/warnings；caveat 明确。
4. 独立 TimedResult/TimedCompareResult；不改变 C1 digest/类型。
5. DB v5 只加独立 `whatif_timed_*` 表，不动 C1 表。
6. 无 promote/SAP/live 写入。

## 3. 非目标

- 不做 promote/plan_to_proposed/Authorization（C2b）。
- 不做 assetDelay/checkpoint（C2b）。
- 不支持 timed 上的 PatchDto/assetStatus；不静默 no-op。
- 不声称物理可行性；不做网络/遥测。

## 4. 前置 API（必须先合并）

### 4.1 `poie-sim::timed`

- 保留现有 `TimedScenario.storage_discharge_w`；**不新增 StorageLimit**。C2a 不使用 capacity/initial SoC 作为新权威；只使用现有 StockTimeline 的 `initial_si/changes` 与 scenario 的 discharge 上限。
- 新增 `TimedScenario::from_source(text: &str, model: Model) -> Result<TimedScenario, TimedError>`：解析 TOML 但不读 `model` 路径；host 负责从 sandbox 读取 model。现有 `load(path)` 保留给仓库/测试。
- 新增 `TimedRunner::evaluate(&TimedCase) -> Result<TimedOutcome, TimedError>`，不要求 `ExpectDoc`；`run(&CaseDoc)` 保留用于 fixtures。
- `TimedCase { goal: Goal, start_s, deadline_s: Option<f64>, min_verification: Verification }`；deadline 为显式秒；不包含 maxDepth/maxActions。
- 现有 `TimedOutcome` 字段/warnings/verdict 语义不变。

### 4.2 Host sandbox 读取

`whatif_timed_scenario_import(path)` 与 run：

1. `path` 为 sandbox 相对目录（含 `scenario.toml`）；用 `sandbox::read` 读取 scenario 文本（no-follow/1 MiB）。
2. 从 scenario TOML 的 `model` 字段取相对路径；host 用 `sandbox::resolve` 解析到 **scenario 所在目录下的相对路径**，拒绝绝对路径/`..`/symlink/超限。
3. `sandbox::read` 读取 model.toml，`Model::load` 或 `Model::from_doc` 构建模型；**不调用** `TimedScenario::load` 的路径拼接。
4. `TimedScenario::from_source(scenario_text, model)`；返回 scenario/model digest。

这样 model 路径不能逃逸 sandbox；run 重读两个文件并重新计算 digest，不一致 → `stale_scenario`/`model_mismatch`。

### 4.3 场景 digest（raw-file profile）

`scenarioSha256 = sha256("poie.timed-scenario/0.2\n" + raw_scenario_bytes)`；这是 DIGEST 的 **raw-file 域**，不做 JSON canonicalization；DIGEST §2.1 增补该 profile，固定向量：

`base-01-timed/scenario.toml`（2026-11-05 内容）→ `5cbfc5a5711b316318fd244c89149b9ef58b6f5d8c417c4747638dc7c1c205f3`。

- 读取前校验 UTF-8/≤256 KiB；文件变化 digest 变。
- 结果 digest 使用 canonical JSON（见 §6）。

## 5. 数据模型（独立 0.1）

### 5.1 Timed Snapshot

```text
TimedSnapshot {
  schema:"poie.timed-snapshot/0.1", schemaVersion:"0.1",
  id,label,modelSha256,scenarioPath,scenarioSha256,
  goal,minVerification,startMode:"beginning"|"at",startS,
  horizonS,nowS,deadlineS?,seed,
  createdAt,createdBy,snapshotSha256
}
```

- `timedSnapshotSha256 = sha256("poie.timed-snapshot/0.1\n" + canonical_json(without id/createdAt/createdBy/seed/snapshotSha))`；`startMode=at` 时 startS ∈ [0,horizonS]；beginning 时 startS=0。
- goal 只接受 `service:<id>`/`restore:<asset>`；minVerification 解析为现有枚举。
- scenarioPath 为 sandbox 相对目录；scenarioSha/modelSha 必须与重读一致。

### 5.2 Timed Variant

```text
TimedVariant {
  schema:"poie.timed-variant/0.1", id,snapshotId,label,description,
  startS?, deadlineS?,
  storageDischargeW:{resource: watts}
}
```

- 无 patch/goal/minVerification 字段；goal/minVerification 来自 snapshot。
- storageDischargeW 只允许 **降低** 现有 `storage_discharge_w[resource]`；未知资源/负值/NaN/超上限拒绝。
- deadlineS 若设置必须 > startS 且 ≤ horizonS；不设置则用 snapshot.deadlineS。
- startS 覆盖 snapshot；`startMode` 由 snapshot 决定。

### 5.3 Timed Result

```text
TimedResult {
  schema:"poie.timed-result/0.1", resultId,variantId,snapshotId,runId,
  mode:"timed",
  settings:{startMode,startS,horizonS,deadlineS?},
  outcome:{timed:"feasible|blocked|unknown|search_limit|too_late|insufficient_resource|capacity_conflict|power_limited",caveat},
  completionS?, warnings[],
  resultSha256
}
```

- `resultSha256 = sha256("poie.timed-result/0.1\n" + canonical_json({snapshotSha256,scenarioSha256,modelSha256,variant:{startS,deadlineS,storageDischargeW},settings,outcome,completionS,warnings}))`；排除 id/createdAt；可选字段 null 不省略。
- caveat 固定：`"relative to this timed scenario and bounded model; not a physical feasibility proof"`。

### 5.4 Timed Compare

```text
TimedCompareResult {
  schema:"poie.timed-compare/0.1", snapshotId, baselineResultId?,
  rows:[{resultId,variantId,outcome,completionS,delta}],
  sortKey:"timedVerdict|completionS|resultId", warnings[], complete:true
}
```

- 只接受同 snapshot 的 `TimedResult`；跨 snapshot → `invalid_parameter`；重复去重并 warning；不混 C1 静态结果。

## 6. 命令

| 命令 | 输入 | 输出 |
| --- | --- | --- |
| `whatif_timed_scenario_import` | sandbox 相对 path | `{scenarioSha256,modelSha256,description}` |
| `whatif_timed_scenario_list` | — | imported scenarios |
| `whatif_timed_snapshot_create` | TimedSnapshotInput | TimedSnapshot |
| `whatif_timed_variant_create` | snapshotId,label,description,startS?,deadlineS?,storageDischargeW | TimedVariant |
| `whatif_timed_run` | variantId | TimedResult |
| `whatif_timed_result_get` | resultId | TimedResult |
| `whatif_timed_result_list` | snapshotId,variantId?,limit | TimedResult[] |
| `whatif_timed_compare` | snapshotId,baselineResultId?,resultIds | TimedCompareResult |

错误码：`invalid_parameter`、`unsupported_schema`、`unknown_snapshot`、`unknown_variant`、`unknown_result`、`model_mismatch`、`scenario_mismatch`、`stale_scenario`、`power_limit_exceeded`、`budget_exceeded`、`unauthorized`、`scope_denied`。

C2a 使用登录会话；无审批/promote。

## 7. 持久化（v5）

只加独立表，不改 C1 表：

```text
whatif_timed_scenarios(scenario_sha256 PK, scenario_path, model_sha256, schema_version, description, imported_at)
whatif_timed_snapshots(snapshot_id PK, snapshot_json, snapshot_sha256, created_at)
whatif_timed_variants(variant_id PK, snapshot_id, variant_json, created_at)
whatif_timed_results(result_id PK, variant_id, snapshot_id, mode, settings_json, result_json, result_sha256, created_at)
```

- `PRAGMA user_version=5`；4→5 建表，幂等；不改 C1 `whatif_*` 表/JSON。
- scenario 以 digest 为 PK；同内容不同 path 只保留首个 path，import 返回已有记录（warning），不覆盖 digest。
- run 重读 scenario/model，校验 digest；不信任缓存。

## 8. 隔离/安全

- 只操作 sandbox 文件与内存 TimedScenario；不写 SAP/ledger/federation，不联网。
- path/model 解析全部走 A0 sandbox；拒绝绝对路径/`..`/symlink/超限。
- run 前后无 live/SAP 写入；测试断言。
- caveat 固定；不显示“部署/在线”文案。

## 9. UI

- Plan 视图 What-if 增加独立 **Timed** 区域：场景导入/选择、startS/deadlineS、storage 放电上限、运行、比较。
- 结果展示 timed verdict/completion/warnings；`power_limited`/`capacity_conflict` 单独文案。
- 浏览器 host-only 占位；C2a 不显示 promote/assetDelay。

## 10. 测试与验收

- scenario raw digest 固定向量（§4.3）；model/scenario 变更拒绝。
- startS/deadline/storage 上限校验；负值/NaN/未知资源拒绝。
- timed 结果至少覆盖 feasible、power_limited、too_late、capacity_conflict；同输入同 resultSha。
- 跨 snapshot compare 拒绝；重复 resultId 去重 warning。
- 路径逃逸/符号链接/超限拒绝；run 重读后 digest 变化 → stale。
- 无 SAP/ledger/federation 写入。
- UI：导入 base-01-timed、3 个 timed 变体运行/比较；host-only 占位。
- 验收：3 个 timed 变体（储能限制/截止提前/startMode）可比较，caveat 显示，无 C1 结果 digest 变化。

## 11. C2b（延后，需重跑门禁）

- promote/plan_to_proposed：审批 token 通用化、parameterized SAP validation、Authorization.basis。
- assetDelay（延迟修复语义）、checkpoint。
- timed PatchDto/assetStatus 覆盖。

## 12. 参考

- [Phase C1](phase-c-what-if.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)
