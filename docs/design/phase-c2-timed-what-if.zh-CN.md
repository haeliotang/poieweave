# Phase C2a 设计：时间版/容量 What-if（C2a）

- 状态：设计草案（待 C2a 门禁）
- 日期：2026-11-05
- 范围：timed/storage 变体与运行；**promote/proposed plan 留到 C2b**
- 上游：C1 `poie-whatif`、`poie-sim::timed`、`poie-ontology::time`、B1 审批原语
- 下游：C2b promote、D2

## 1. 背景

C1 只支持静态 structure/plan。真实调度决策需要“泵晚修好 / 备用泵故障 / 储能放电受限 / 截止提前”的时间推演。C2a 在 **本地 TimedScenario 副本**上运行 timed 变体，不写 live/SAP/ledger，不做 promote。

## 2. 目标

1. Snapshot 绑定 **model digest + timed scenario digest**，可复现。
2. Variant 在 C1 PatchDto 基础上增加 `startMode`、`startS`、`storageDischargeW`、`assetDelayS`、goal/deadline 覆盖。
3. `assetDelayS` 只延后“恢复/修复”类状态变化，不延后故障预测窗口。
4. timed 运行返回 TimedVerdict/completion/power_limited/capacity_conflict/warnings；保留 caveat。
5. 结果持久化、compare 复用 C1 registry。
6. UI 提供 timed 字段；C2a 无 promote。

## 3. 非目标

- 不做 promote/plan_to_proposed/Authorization（C2b）。
- 不做 checkpoint/中途状态恢复（C2b）。
- 不做物理可行性证明；timed 结论仍是模型/场景内结论。
- 不写 live/SAP/federation；不做网络。

## 4. 前置 API（必须先合并）

### 4.1 `poie-ontology::time`

- `AssetTimeline::changes() -> &[StatusChange]` 只读；`StatusChange` 公开 `at_s/status`。
- `AssetTimeline::shift_restorations(delta_s, horizon_s) -> Result<(), TimeError>`：
  - 只移动 `status != Failed && status != Unknown` 的 change（恢复/修复）；
  - 不移动 predicted_failure；
  - 结果排序、有限值、delta ∈ [0, 24h]；不修改原 timeline（返回新对象或 `shifted_restorations`）。
- `AssetTimeline::initial()` 只读。
- `StockTimeline` 只读访问器（资源/初始量/变化）。

### 4.2 `poie-sim::timed`

- `TimedScenario` 保留 `storage: BTreeMap<String, StorageLimit>`：

```text
StorageLimit { resource, capacity_si, initial_soc_si, max_discharge_w }
```

- 公开 `storage()`、`asset_timelines()`、`stock_timelines()`、`storage_discharge_w()`、`description()`、`evidence_validity_s()`。
- 新增 `TimedRunner::evaluate(&TimedCase) -> Result<TimedOutcome, TimedError>`，**不要求 `ExpectDoc`**；现有 `run(&CaseDoc)` 保留用于 fixtures。
- `TimedCase { goal: Goal, start_s, deadline_s, min_verification }`；start/deadline 已解析为秒，不再用字符串。
- `TimedOutcome` 增加 `warnings` 已有；`power_limited` 由 storage discharge 触发，`capacity_conflict`/`insufficient_resource` 保持。

### 4.3 场景 digest

`poie.timed-scenario/0.1`：canonical JSON 包含 model digest、asset timelines（初始/变化/predicted failure/validity）、stock timelines、storage limits、discharge W、description、evidence validity。`scenarioSha256` 由该 domain 计算。

## 5. 数据模型

### 5.1 Snapshot 0.2

```text
Snapshot {
  schema:"poie.whatif-snapshot/0.2", schemaVersion:"0.2",
  id,label,modelSha256,scenarioSha256,
  kind:"static"|"timed",
  goal,minVerification,startMode,startS,deadlineS?,
  horizonS, nowS, maxDepth,maxActions, seed,
  createdAt,createdBy,snapshotSha256
}
```

- `kind=timed` 时 `scenarioSha256` 必填；`kind=static` 时为 null。
- 命令 `whatif_snapshot_create` 扩展 `kind`/`scenarioSha256`；host 从 **C2a 场景文件** 加载 TimedScenario（Base-02 的 case 文件）并校验 digest。
- `startMode`：`beginning`（start=0）或 `at`（start=startS）；**checkpoint 不支持**（C2b）。
- Snapshot digest 排除 id/createdAt/createdBy/seed，纳入 kind/scenarioSha。

### 5.2 Variant 0.2

```text
Variant {
  schema:"poie.whatif-variant/0.2", id,snapshotId,label,description,
  patch:PatchDto, goal?,minVerification?,deadlineS?,
  startMode:"beginning"|"at", startS?,
  storageDischargeW: {resource: watts},   // 只允许降低/设置已有 storage 上限
  assetDelayS: {asset: seconds}            // 只延迟该资产的 restoration
}
```

- 上限：storageDischargeW ≤ scenario 原 max_discharge_w；assetDelayS ≤ 24h；未知资源/资产拒绝。
- `assetDelayS` 先应用 restore shift，再应用 PatchDto；顺序固定。
- 不包含 checkpoint/promote 字段。

### 5.3 Result 0.2

```text
VariantResult {
  schema:"poie.whatif-result/0.2", resultId,variantId,snapshotId,runId,
  mode:"timed",
  settings:{startMode,startS,horizonS,deadlineS?,maxDepth,maxActions},
  outcome:{timed: "feasible|blocked|unknown|search_limit|too_late|insufficient_resource|capacity_conflict|power_limited", caveat},
  completionS?, tickDigest?, warnings[],
  resultSha256
}
```

- `resultSha256` 绑定 snapshotSha/scenarioSha/variant/storage/assetDelay/settings/outcome/completion/warnings；排除 id/createdAt。
- compare 复用 C1 `CompareResult`，只比较同 mode。

## 6. 命令

| 命令 | 输入 | 输出 |
| --- | --- | --- |
| `whatif_snapshot_create` | 扩展 SnapshotInput(kind, scenarioSha256, startMode, startS) | Snapshot |
| `whatif_variant_create` | 扩展 VariantInput(storageDischargeW, assetDelayS, startMode, startS) | Variant |
| `whatif_variant_run` | variantId, mode=`timed` | VariantResult |
| `whatif_result_get/list`、`whatif_compare` | 复用 C1 | — |

错误码：`unsupported_schema`、`unknown_snapshot`、`unknown_variant`、`invalid_patch`、`invalid_parameter`、`model_mismatch`、`scenario_mismatch`、`power_limit_exceeded`、`budget_exceeded`、`unauthorized`。

C2a 仍用登录会话；**不引入 promote 审批**。

## 7. 持久化

`PRAGMA user_version=5`：

```text
whatif_scenarios(scenario_sha256 PK, scenario_json, model_sha256, created_at)
whatif_snapshots 增加 scenario_sha256/kind 列（迁移）
whatif_results 增加 mode/settings_json 已存在；timed 结果复用
```

- 导入/保存场景 JSON 前重算 digest；未知/不匹配拒绝。
- 迁移从 4→5：加列/建表，幂等，不改 C1 已有行。

## 8. 隔离与 caveat

- 只用 `TimedScenario` 的内存副本与本地 runner；不写 SAP/ledger/federation。
- `assetDelayS` 语义测试：延迟修复不延迟 predicted failure；验证 failure 窗口仍在原时间。
- 结果 caveat：`"relative to this timed scenario and bounded model; not a physical feasibility proof"`。
- 不声称“live 不变”是安全边界；测试断言无 live 写入路径。

## 9. UI

- What-if 面板增加 `kind=timed` 选项、场景选择、startMode/startS、storageDischargeW、assetDelayS 字段。
- 结果显示 timed verdict/completion/warnings；`power_limited`/`capacity_conflict` 单独文案。
- 浏览器 host-only 占位不变。
- 不显示 promote/部署文案。

## 10. 测试与验收

- scenario digest 稳定；model/scenario mismatch 拒绝。
- assetDelay 只移动 restoration；failure 窗口不变；上限/负值拒绝。
- timed：feasible/too_late/power_limited/capacity_conflict/insufficient_resource 至少各 1。
- 同输入同 resultSha；跨 snapshot compare 拒绝。
- 无 SAP/ledger/federation 写入。
- UI：创建 timed snapshot/variant、运行、compare；host-only 占位。
- 验收：3 个 timed 变体（延迟修复/储能限制/截止提前）可比较且 caveat 显示。

## 11. C2b（延后，需重跑门禁）

- promote/plan_to_proposed：审批 token 通用化、parameterized SAP validation、Authorization.basis。
- checkpoint/中途恢复。
- 多场景批量。

## 12. 参考

- [Phase C1](phase-c-what-if.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)
