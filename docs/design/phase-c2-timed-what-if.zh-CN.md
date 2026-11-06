# Phase C2a 设计：时间版/容量 What-if（独立 C2a 模型）

- 状态：**C2a 门禁 PASS + 已实现 0.2.0**（2026-11-06）
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
- 新增 `TimedScenario::from_source(text: &str, model: Model) -> Result<TimedScenario, TimedError>`：解析 TOML 但不读 `model` 路径；host 负责从 sandbox 读取 model。现有 `load(path)` 仅用于仓库/测试，C2a host 不使用。
- 新增 `TimedRunner::evaluate(&TimedCase) -> Result<TimedOutcome, TimedError>`，不要求 `ExpectDoc`；`run(&CaseDoc)` 保留用于 fixtures。
- `TimedCase { goal, start_s, deadline_s, horizon_s, min_verification }`；C2a 固定 now=0，runner 校验 `start_s ∈ [0,horizon_s]`、deadline 在 (start,horizon]；completion > horizon → `too_late`。
- 现有 `TimedOutcome` 字段/warnings/verdict 语义不变。

### 4.2 Host sandbox 读取

`whatif_timed_scenario_import(path)` 与 run：

sandbox 布局固定为：`base-01-timed/scenario.toml` + `base-01/model.toml`（与 repo `scenarios/` 同构）。

1. `scenarioPath` 是 sandbox 相对 **文件** 路径（如 `base-01-timed/scenario.toml`）；用 `sandbox::read` 读取（no-follow/1 MiB）。
2. 从 scenario TOML 的 `model` 字段取相对路径（base-01-timed 为 `../base-01/model.toml`）；允许 `..` 仅当 `sandbox::resolve_inside(root, scenario_parent, relative)` 归一化后仍在 **sandbox root 内**；拒绝绝对路径/逃逸/symlink/超限，不能直接 `Path::join`。
3. `sandbox::read` 读取 model.toml；用 `toml::from_str::<ModelDoc>` + `Model::from_doc` 构建模型，**不调用** `Model::load`/`TimedScenario::load` 的路径读取。
4. `TimedScenario::from_source(scenario_text, model)`；返回 scenario/model digest。
5. 导入时要求 scenario model digest == host `state.model` digest（PLATFORM §5 单一 SourceModel）；不同 → `model_mismatch`。
6. 验收前必须把 repo 的 `scenarios/base-01-timed/` 与 `scenarios/base-01/` 复制到 app sandbox 保持相对布局；host 不读 repo 绝对路径。

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
  goal,minVerification,startS,
  horizonS,nowS,deadlineS?,
  createdAt,createdBy,snapshotSha256
}
```

- `TimedSnapshotInput {label, scenarioPath, scenarioSha256, goal,minVerification, startS, horizonS, deadlineS?}`；C2a 固定 `nowS=0`（timed scenario 从 0 开始），`horizonS > 0`、`startS ∈ [0,horizonS]`、`deadlineS ∈ (startS,horizonS]`；缺省 startS=0。
- `snapshotSha256 = sha256("poie.timed-snapshot/0.1\n" + canonical_json(without id/createdAt/createdBy/snapshotSha256))`；**scenarioPath 不进入 digest**（仅定位），纳入 scenarioSha/modelSha。
- goal 只接受 `service:<id>`/`restore:<asset>`；minVerification 解析为现有枚举。
- 固定向量：`{"schema":"poie.timed-snapshot/0.1","schemaVersion":"0.1","label":"base-01-timed","modelSha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","scenarioSha256":"5cbfc5a5711b316318fd244c89149b9ef58b6f5d8c417c4747638dc7c1c205f3","goal":"restore:pump-p1","minVerification":"demonstrated","startS":0.0,"horizonS":86400.0,"nowS":0.0,"deadlineS":null}` → `e694e44a32754e876008a99fef02a59904d126f3488a4156519445b27218ef65`。
- scenarioPath/scenarioSha/modelSha 必须与 run 重读一致。

### 5.2 Timed Variant

```text
TimedVariant {
  schema:"poie.timed-variant/0.1", schemaVersion:"0.1", id,snapshotId,label,description,
  startMode:"beginning"|"at", startS?, deadlineS?,
  storageDischargeW:{resource: watts}
}
```

- 无 patch/goal/minVerification 字段；goal/minVerification 来自 snapshot。
- `startMode` 在 **variant** 上（snapshot 只有默认 startS）；`beginning` 时 start=0 且 startS 必须为 null/0；`at` 时 startS 必填且 ∈ [0,horizonS]；3 个变体可分别使用 beginning/at/不同 startS，仍同 snapshot 比较。
- storageDischargeW 只允许 **降低** 现有 `storage_discharge_w[resource]`；未知资源/负值/NaN/超上限拒绝。
- deadline 一律相对 **resolved start** 校验（beginning=0，at=startS）。
- startS 覆盖 snapshot 默认 startS；`startMode` 由 variant 决定；deadlineS 若设置必须 > **resolved start** 且 ≤ horizonS，未设置用 snapshot.deadlineS。

### 5.3 Timed Result

```text
TimedResult {
  schema:"poie.timed-result/0.1", schemaVersion:"0.1", resultId,variantId,snapshotId,runId,
  mode:"timed",
  settings:{startMode,startS,horizonS,deadlineS?},
  outcome:{timed:"feasible|blocked|unknown|search_limit|too_late|insufficient_resource|capacity_conflict|power_limited",caveat},
  completionS?, warnings[],
  residuals:{completionS,horizonS,warningCount,verdict},
  abstained:bool, abstainReason?,
  resultSha256
}
```

- `resultSha256 = sha256("poie.timed-result/0.1\n" + canonical_json({snapshotSha256,scenarioSha256,modelSha256,variant:{startMode,startS,deadlineS,storageDischargeW},settings,outcome,completionS,warnings,residuals,abstained,abstainReason}))`；排除 id/createdAt；可选字段 null 不省略。
- 固定向量（C2a v0.2 加 residuals/abstention）：`{"snapshotSha256":"bbbb…","scenarioSha256":"5cbf…","modelSha256":"aaaa…","variant":{"startMode":"beginning","startS":0.0,"deadlineS":null,"storageDischargeW":{"electric-energy":3000.0}},"settings":{"startMode":"beginning","startS":0.0,"horizonS":86400.0,"deadlineS":null},"outcome":{"timed":"power_limited","caveat":"relative to this timed scenario and bounded model; not a physical feasibility proof"},"completionS":null,"warnings":[],"residuals":{"completionS":null,"horizonS":86400.0,"warningCount":0,"verdict":"power_limited"},"abstained":false,"abstainReason":null}` → `e6f616514da00c9a4fdce83ed7bc0bf5611e73d14236ce2bf73f381a103c80fc`。
- caveat 固定：`"relative to this timed scenario and bounded model; not a physical feasibility proof"`。

### 5.4 Timed Compare

```text
TimedCompareResult {
  schema:"poie.timed-compare/0.1", schemaVersion:"0.1", snapshotId, baselineResultId?,
  rows:[{resultId,variantId,outcome,completionS,delta}],
  sortKey:"timedVerdict|completionS|resultId", warnings[], complete:true
}
```

- 只接受同 snapshot 的 `TimedResult`；跨 snapshot → `invalid_parameter`；重复去重并 warning；不混 C1 静态结果。
- `delta.completionDeltaS`：row 和 baseline 的 completionS 都非 null 时 = row - baseline（行减基线）；任一为 null 时 = null（不做减法）；baseline 必须在 rows 中，否则 `baselineResultId=null`、delta=null 并 warning；排序固定顺序 `feasible, blocked, unknown, search_limit, too_late, insufficient_resource, capacity_conflict, power_limited` → completionS null 最后 → resultId。

## 6. 命令

| 命令 | 输入 | 输出 |
| --- | --- | --- |
| `whatif_timed_scenario_import` | sandbox 相对 path | `{scenarioSha256,modelSha256,description}` |
| `whatif_timed_scenario_list` | — | `{scenarioSha256,scenarioPath,modelSha256,description,importedAt}[]` |
| `whatif_timed_snapshot_create` | TimedSnapshotInput（startS/horizonS；nowS 固定 0） | TimedSnapshot |
| `whatif_timed_variant_create` | snapshotId,label,description,startMode,startS?,deadlineS?,storageDischargeW | TimedVariant |
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
whatif_timed_snapshots(snapshot_id PK, schema_version, snapshot_json, snapshot_sha256, created_at)
whatif_timed_variants(variant_id PK, snapshot_id, schema_version, variant_json, created_at)
whatif_timed_results(result_id PK, variant_id, snapshot_id, schema_version, mode, settings_json, result_json, result_sha256, created_at)
```

- `PRAGMA user_version=5`；4→5 建表，幂等；不改 C1 `whatif_*` 表/JSON。
- scenario 以 digest 为 PK；同内容不同 path 只保留首个 path，import 返回已有记录（warning），不覆盖 digest。
- run 重读 scenario/model，校验 digest；不信任缓存。

## 8. 隔离/安全

- 只操作 sandbox 文件与内存 TimedScenario；不写 SAP/ledger/federation，不联网。
- path/model 解析全部走 A0 sandbox；拒绝逃逸 sandbox root 的绝对路径/`..`/symlink/超限（同目录 `../base-01` 合法）。
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
- 验收：同 snapshot 的 3 个 timed 变体（不同 startMode/startS、储能限制、截止提前）可比较，caveat 显示，无 C1 结果 digest 变化。

## 11. C2b（延后，需重跑门禁）

- promote/plan_to_proposed：审批 token 通用化、parameterized SAP validation、Authorization.basis。
- assetDelay（延迟修复语义）、checkpoint。
- timed PatchDto/assetStatus 覆盖。

## 12. 参考

- [Phase C1](phase-c-what-if.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)
