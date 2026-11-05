# Phase C2a 设计：时间版/容量 What-if（C2a）

- 状态：设计草案（待 C2a 门禁）
- 日期：2026-11-05
- 范围：timed/storage 变体与运行；**promote/assetDelay/checkpoint 留到 C2b**
- 上游：C1 `poie-whatif`、`poie-sim::timed`、`poie-ontology::time`、B1 审批原语
- 下游：C2b promote、D2

## 1. 背景

C1 只支持静态 structure/plan。真实调度决策需要“泵晚修好 / 备用泵故障 / 储能放电受限 / 截止提前”的时间推演。C2a 在 **本地 TimedScenario 副本**上运行 timed 变体，不写 live/SAP/ledger，不做 promote。

## 2. 目标

1. Snapshot 绑定 **model digest + timed scenario digest**，可复现。
2. Variant 在 C1 PatchDto 基础上增加 `startMode`、`startS`、`storageDischargeW`、goal/deadline 覆盖；assetDelay 属 C2b。
3. storage 变体只调整已有 `storage_discharge_w`，容量/初始 SoC 以 StockTimeline 为准；不引入重复 StorageLimit。
4. timed 运行返回 TimedVerdict/completion/power_limited/capacity_conflict/warnings；保留 caveat。
5. scenario 以 sandbox 相对路径 + digest 绑定，不复制 Model/JSON。
6. 结果持久化，compare 扩展 C1 以支持 `mode=timed`。
6. UI 提供 timed 字段；C2a 无 promote。

## 3. 非目标

- 不做 promote/plan_to_proposed/Authorization（C2b）。
- 不做 checkpoint/中途状态恢复（C2b）。
- 不做物理可行性证明；timed 结论仍是模型/场景内结论。
- 不写 live/SAP/federation；不做网络。

## 4. 前置 API（必须先合并）

### 4.1 `poie-ontology::time`

C2a 不新增 AssetTimeline shift API（assetDelay 延后 C2b）。现有 `AssetTimeline::status_at`、`StockTimeline` 只读访问器足够。

### 4.2 `poie-sim::timed`

- **不新增 StorageLimit**：容量/初始量来自 `StockTimeline`；放电上限使用现有 `TimedScenario.storage_discharge_w`。
- 公开 getter：`scenario.storage_discharge_w()`、`asset_timelines()`、`stock_timelines()`、`description()`、`evidence_validity_s()`（字段已 pub，补 accessor 仅为封装）。
- 新增 `TimedRunner::evaluate(&TimedCase) -> Result<TimedOutcome, TimedError>`，**不要求 `ExpectDoc`**；现有 `run(&CaseDoc)` 保留用于 fixtures。
- `TimedCase { goal: Goal, start_s, deadline_s, min_verification }`；不包含未使用的 maxDepth/maxActions/horizon。deadline 为已解析秒；`deadline="prediction"` 留在 fixtures/`CaseDoc`，C2a 变体只接受显式秒。
- `TimedOutcome` 的 warnings/power_limited/capacity_conflict 语义保持。

### 4.3 场景来源与 digest

- C2a 使用真实文件 `scenarios/base-01-timed/scenario.toml`（schema `poie.timed-scenario/0.2`），不引用不存在的 Base-02 timed。
- 新增 host 命令 `whatif_scenario_import(path)`：`path` 是 **sandbox 相对路径**；只读文件、拒绝绝对路径/`..`/symlink/非 UTF-8/超 256 KiB；返回 `{scenarioSha256, modelSha256, description, timedRange}`。
- `scenarioSha256 = sha256("poie.timed-scenario/0.2\n" + raw_file_bytes)`：直接哈希 sandbox 文件的原始字节（TOML），不做 TOML→JSON canonicalization；读取时先校验 UTF-8/大小，再哈希；C2a **不序列化 TimedScenario**，run 时从同一 sandbox 路径重新 `TimedScenario::load`，再校验 digest。
- scenario 的 model digest 必须等于 snapshot.modelSha256；不一致 → `scenario_mismatch`。
- fixture 变更由 digest 捕获；文件被替换 → `stale_scenario`。

## 5. 数据模型

### 5.1 Snapshot 0.2

```text
Snapshot {
  schema:"poie.whatif-snapshot/0.2", schemaVersion:"0.2",
  id,label,modelSha256,scenarioPath,scenarioSha256,
  kind:"static"|"timed",
  goal,minVerification,startMode:"beginning"|"at",startS,deadlineS?,
  horizonS, nowS, maxDepth,maxActions, seed,
  createdAt,createdBy,snapshotSha256
}
```

- `kind=timed` 时 `scenarioPath`/`scenarioSha256` 必填；`kind=static` 时为 null。
- 命令 `whatif_snapshot_create` 扩展 `kind/scenarioPath/scenarioSha256`；host 从 sandbox 重新加载并校验 digest。
- `startMode`：`beginning`（start=0）或 `at`（start=startS）；**checkpoint 不支持**（C2b）。
- Snapshot digest `poie.snapshot/0.2` 排除 id/createdAt/createdBy/seed，纳入 kind/scenarioPath/scenarioSha；`evidenceSha256` 仍可选，缺省写 null。

### 5.2 Variant 0.2

```text
Variant {
  schema:"poie.whatif-variant/0.2", id,snapshotId,label,description,
  patch:PatchDto, goal?,minVerification?,deadlineS?,
  startMode:"beginning"|"at", startS?,
  storageDischargeW: {resource: watts}   // 只允许降低已有 storage 上限
}
```

- 上限：storageDischargeW ≤ scenario 原 `storage_discharge_w[resource]`；未知资源/负值/NaN 拒绝。
- **不包含 assetDelayS/checkpoint/promote 字段**（C2b）。
- startMode/startS 覆盖 snapshot；`startMode=at` 时 startS 必须在 [0, timedRange]。
- PatchDto 先应用，再应用 storageDischargeW。

### 5.3 Result 0.2

```text
VariantResult {
  schema:"poie.whatif-result/0.2", resultId,variantId,snapshotId,runId,
  mode:"timed",
  settings:{startMode,startS,horizonS,deadlineS?,maxDepth,maxActions},
  outcome:{structure:null, plan:null, timed:"feasible|blocked|unknown|search_limit|too_late|insufficient_resource|capacity_conflict|power_limited", caveat},
  completionS?, warnings[],
  actions:[], rootCauses:[], externalDependencies:[],
  resultSha256
}
```

- 扩展 C1 `OutcomeKind` 增加 `timed?: string`，C1 `VariantResult` 的 actions/rootCauses/externalDependencies 加 `#[serde(default)]`，保证静态/时间结果都能反序列化。
- C1 `compare` 先按 result.mode 过滤；排序键 `timed verdict (按固定顺序) | structure|plan | completionS | resultId`；不混排 mode。
- `resultSha256 = sha256("poie.variant/0.2\n" + canonical_json({snapshotSha256,scenarioSha256,mode,storageDischargeW,settings,outcome,completionS,warnings,actions,rootCauses,externalDependencies}))`；排除 id/createdAt；可选字段 null 不省略。
- 删除 `tickDigest`（C2a 不定义；如需 tick 哈希，C2b 另立）。

## 6. 命令

| 命令 | 输入 | 输出 |
| --- | --- | --- |
| `whatif_scenario_import` | sandbox 相对 path | `{scenarioSha256, modelSha256, description, timedRange}` |
| `whatif_scenario_list` | — | imported scenarios |
| `whatif_snapshot_create` | 扩展 SnapshotInput(kind, scenarioPath, scenarioSha256, startMode, startS) | Snapshot |
| `whatif_variant_create` | 扩展 VariantInput(storageDischargeW, startMode, startS) | Variant |
| `whatif_variant_run` | variantId, mode=`timed` | VariantResult |
| `whatif_result_get/list`、`whatif_compare` | 复用 C1 | — |

错误码：`unsupported_schema`、`unknown_snapshot`、`unknown_variant`、`unknown_result`、`invalid_patch`、`invalid_parameter`、`model_mismatch`、`scenario_mismatch`、`stale_scenario`、`budget_exceeded`、`unauthorized`。

C2a 仍用登录会话；**不引入 promote 审批**。

## 7. 持久化

`PRAGMA user_version=5`：

```text
whatif_scenarios(scenario_sha256 PK, scenario_path, schema_version, model_sha256, imported_at)
whatif_snapshots 增加 scenario_path/scenario_sha256/kind 列（迁移）
whatif_results 增加 mode/settings_json 已存在；timed 结果复用
```

- `whatif_scenario_import` 存 path+digest，不存 JSON/Model；run 时重新加载并校验 digest。
- 迁移 4→5 用 `has_column` 条件 ALTER，幂等；`schema_version` 必填；失败回滚；静态行 backfill `kind='static', scenario_* NULL`。
- `db.rs` v4 tests 更新为 v5 并断言新列/表。

## 8. 隔离与 caveat

- 只用 `TimedScenario` 的内存副本与本地 runner；不写 SAP/ledger/federation。
- storageDischargeW 上限/未知资源/负值测试；StockTimeline 容量/初始量仍为唯一来源。
- 结果 caveat：`"relative to this timed scenario and bounded model; not a physical feasibility proof"`。
- 不声称“live 不变”是安全边界；测试断言无 live 写入路径。

## 9. UI

- What-if 面板增加 `kind=timed` 选项、场景选择、startMode/startS、storageDischargeW 字段；不显示 assetDelay。
- 结果显示 timed verdict/completion/warnings；`power_limited`/`capacity_conflict` 单独文案。
- 浏览器 host-only 占位不变。
- 不显示 promote/部署文案。

## 10. 测试与验收

- scenario digest 稳定；model/scenario mismatch 拒绝。
- storageDischargeW 上限/未知资源/负值拒绝；StockTimeline 容量/初始量不被重复定义。
- timed：feasible/too_late/power_limited/capacity_conflict/insufficient_resource 至少各 1。
- 同输入同 resultSha；跨 snapshot compare 拒绝。
- 无 SAP/ledger/federation 写入。
- UI：创建 timed snapshot/variant、运行、compare；host-only 占位。
- 验收：3 个 timed 变体（储能限制/截止提前/startMode）可比较且 caveat 显示；assetDelay 属 C2b。

## 11. C2b（延后，需重跑门禁）

- promote/plan_to_proposed：审批 token 通用化、parameterized SAP validation、Authorization.basis。
- checkpoint/中途恢复。
- 多场景批量。

## 12. 参考

- [Phase C1](phase-c-what-if.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)
