# Phase C 设计：What-if / 场景推演工作区（C1 MVP）

- 状态：设计草案（Phase C 门禁重跑后方可开发）
- 日期：2026-11-05
- 范围：C1 静态基态上的结构/计划 What-if；C2 的 timed/storage/promote 明确延后
- 上游：Phase A Catalog、Phase B 审批原语（C1 不使用审批）
- 下游：Phase D 版本/证据绑定

## 1. 背景与问题

Poieweave 有 `poie-ontology` 图求解、`poie-plan` 计划和 VEC，但桌面没有“如果泵晚修好 / 备用泵故障 / 截止提前”的可重复推演。Phase C1 在**静态 Base-01 演示模型**上做结构/计划变体，不触碰 live 状态、不写 SAP、不做 timed/storage；这些留到 C2，避免在 B 审批和 timed API 未完备时伪装能力。

## 2. 目标

1. 不可变 Snapshot：固定 SourceModel digest、目标、horizon、验证等级、时间基准。
2. Variant：复用 `poie-ontology::Patch` 的 13 字段 DTO + 目标/截止覆盖；无 timed/storage 扰动。
3. 运行变体：结构求解 + 计划；结果含结构/计划 outcome、根因、外部依赖、plan fingerprint。
4. Baseline vs 多 variant 对比：结果持久化、可复现、完整差异。
5. UI Sim 模式：host-only；展示 `SIMULATION`，不把模拟当 live。
6. C1 不提供 promote/timed；C2 另立设计并重跑 C 门禁。

## 3. 非目标

- 不做 live 状态修改、SAP 记录、Authorization、dispatch、federation 写入。
- 不做 timed/storage/assetDelayS/checkpoint；不做 power_limited（C2）。
- 不做 promote/plan_to_proposed；不做审批链（C2，依赖 B1 审批完成）。
- 不做任意脚本/Python/optimizer；变体数量/字段有上限。
- 不声称结构 `blocked/candidate` 等于物理不可行/可行；必须带 caveat。
- 不做浏览器端真实运行；浏览器预览只显示 host-only 占位。

## 4. 用户与场景

| 场景 | 变体 |
| --- | --- |
| 泵晚修好 | `assetStatus`/`assetDegraded` 修改 pump-p1 |
| 备用泵故障 | `assetStatus` 改 standby 为 failed，观察 requires_any |
| 截止提前 | `deadlineS` 覆盖 |
| 目标改变 | `goal` 覆盖为另一个 service/restore |
| 隔离品放行 | `lotResource`/`lotQuality` 改到 accepted 资源类型 |
| 多方案比较 | 创建 A/B/C，比较 outcome/完成时间/根因 |

## 5. 数据模型

### 5.1 Snapshot

```text
Snapshot {
  schema: "poie.whatif-snapshot/0.1",
  schemaVersion, id, label,
  snapshotSha256: string,       // computed, see below
  modelSha256,                  // SourceModel digest
  evidenceSha256: string|null,  // VEC/report digest if supplied; else null
  goal: string, minVerification: string,
  horizonS, nowS, startS, deadlineS?,
  maxDepth, maxActions, seed,
  createdAt, createdBy
}
```

- `goal` 只接受 `service:<id>` 或 `restore:<asset>`；用现有 graph `Goal` 解析。
- `evidenceSha256` 由调用者显式提供（当前 VEC report 的 SHA-256）或 null；不现场哈希仓库。
- `snapshotSha256 = sha256("poie.snapshot/0.1\n" + canonical_json(snapshot_without_sha))`；排除 `snapshotSha256/id/createdAt/createdBy/seed`，其余字段全部纳入（见 DIGEST）。
- 命令 `whatif_snapshot_create` 接收上述输入；host 用当前 SourceModel 校验 modelSha256。

### 5.2 Variant

```text
Variant {
  schema: "poie.whatif-variant/0.1",
  schemaVersion, id, snapshotId, label, description,
  patch: PatchDto,              // 13 字段，一一对应 poie-ontology::Patch
  goal: string|null, minVerification: string|null,
  deadlineS: number|null,
  createdAt, createdBy
}
```

- `PatchDto` 必须覆盖 `Patch` 全部 13 字段（asset_status/asset_requires/asset_requires_any/lot_quantity/lot_quality/lot_resource/process_verification/asset_degraded/asset_capability_verification/add_asset/add_resource/add_lot/add_process）；转换在 `poie-whatif`，拒绝未知字段。
- 不包含 `Perturbation`/timed/storage；C1 不做 timed。
- 限制：每 snapshot ≤8 variant；每 patch ≤32 entries；拒绝未知 ID/单位/维度/负数。

### 5.3 Result

```text
OutcomeKind {
  structure?: "candidate|blocked|unknown|search_limit",
  plan?: "planned|infeasible|unknown|search_limit",
  caveat: "相对当前静态快照与有界求解器，不是物理可行/不可行证明"
}
VariantResult {
  schema: "poie.whatif-result/0.1",
  schemaVersion, resultId, variantId, snapshotId, runId,
  mode: "structure|plan",
  settings: { nowS, startS, horizonS, maxDepth, maxActions, deadlineS? },
  outcome: OutcomeKind,
  planFingerprint?, planSha256?, completionS?,
  actions[{ actionId, process, target, startS, endS }],
  rootCauses[{ kind, subject, detail }],
  externalDependencies[{ resource, maxSi }],
  resultSha256, warnings[]
}
```

- `resultSha256 = sha256("poie.variant/0.1\n" + canonical_json({snapshotSha256, variant:{patch,goal,minVerification,deadlineS}, mode, settings}))`；排除 variant id/createdAt/createdBy、result id/runId；同输入同 digest。
- 结果持久化到 SQLite `whatif_results`；`compare` 使用 resultId。

## 6. 架构

```text
crates/poie-whatif       NEW: Snapshot/PatchDto/Variant/Outcome/Compare; pure Rust
  ├── poie-ontology      Model/Patch/graph
  ├── poie-plan          plan
  ├── poie-objects       object refs (optional)
  └── poie-digest        canonical JSON/SHA-256
        │
apps/desktop/src-tauri    whatif.rs: DB/commands/audit; no live SAP writes
apps/desktop/src          Sim mode + variant editor + compare; host-only
```

- `poie-whatif` 不依赖 Tauri/网络/poie-exec/poie-sim（C1）。
- 每个 variant 构造独立的 `Model`（`Model::patched`）；不共享 live ledger/plan。

### 6.1 运行流程

1. `whatif_snapshot_create(input)`：校验 modelSha256/目标/单位，计算 snapshot digest，存 DB。
2. `whatif_variant_create(snapshotId,label,patch,goal?,minVerification?,deadlineS?)`：校验 patch，存 DB。
3. `whatif_variant_run(variantId, mode=structure|plan)`：
   - load snapshot/model；`Model::patched`;
   - structure: graph solve goal;
   - plan: `poie-plan::plan`。
     `PlanningRequest { model, now_s: snapshot.nowS, horizon_s: snapshot.horizonS, goals: [PlanningGoal { goal, deadline_s: variant.deadlineS ?? snapshot.deadlineS, priority }], min_verification: variant.minVerification ?? snapshot.minVerification, max_actions: snapshot.maxActions }`；`startS` 仅记录为调度起点提示，C1 不把 `startS` 传给当前 `PlanningRequest`。
     `PlanError` 映射：`InvalidRequest`/`NoCandidate` → `infeasible`；`ActionBudget`/`Horizon` → `search_limit`；未知状态/未知根因 → `unknown`；成功 → `planned`。
     `deadlineS` 只进入 `PlanningGoal.deadline_s`，不是 `PlanningRequest` 字段。
   - 覆盖优先级：variant 的 goal/minVerification/deadline > snapshot；未设置时用 snapshot。
   - result digest; store DB; no writes to live.
4. `whatif_compare(snapshotId, baselineResultId?, resultIds[])`：按 outcome/完成时间/resultId 稳定排序，返回全部差异。
5. C1 无 promote；命令表不包含 promote。

### 6.2 隔离

- C1 不创建 SAP 记录/ActionLedger/federation 写入；只读 SourceModel + 本地 Model 副本。
- 不声称“live 不变”作为安全边界；测试断言 **没有可写的 live 对象**，DB 只写 whatif 表。
- host 的 Sim 模式只是 UI 提示；不改变 SAP/dispatch 能力。

## 7. API

| 命令 | 输入 | 输出 |
| --- | --- | --- |
| `whatif_snapshot_create` | SnapshotInput | Snapshot |
| `whatif_snapshot_list` | limit | Snapshot[] |
| `whatif_variant_create` | snapshotId,label,description,patch,goal?,minVerification?,deadlineS? | Variant |
| `whatif_variant_list` | snapshotId | Variant[] |
| `whatif_variant_run` | variantId, mode (`structure|plan`) | VariantResult |
| `whatif_compare` | snapshotId, baselineResultId?, resultIds[] | CompareResult |
| `whatif_result_get` | resultId | VariantResult |
| `whatif_result_list` | snapshotId, variantId?, limit | VariantResult[] |
| `whatif_variant_delete` | variantId | ok |

CompareResult：

```text
CompareResult {
  schema: "poie.whatif-compare/0.1",
  schemaVersion, snapshotId, baselineResultId?,
  rows[{ resultId, variantId, mode, outcome, completionS?, planFingerprint?, delta }],
  sortKey: "outcome|completionS|resultId",
  complete: true
}
```

- `resultIds` 为空：比较该 snapshot 全部结果；跨 snapshot/未知/重复 resultId → `invalid_patch`/`unknown_result`；不同 mode 可并列但 `delta` 只在同 mode 计算。
- `delta` 为 `{completionDeltaS?, outcomeChanged: bool}`；不比较不同 mode 的数值。
- `whatif_result_list(snapshotId, variantId?, limit)` 返回持久化结果，重启后可达。

错误码（PLATFORM）：`unknown_snapshot`、`unknown_variant`、`unknown_result`、`invalid_patch`、`unsupported_schema`、`model_mismatch`、`budget_exceeded`、`unauthorized`、`scope_denied`。C1 不需要 `approval_required`；host 命令仍用 `{code,message,details}` 包装（B0 CommandError 可先复用字符串前缀，C1 PR 迁移到 CommandError）。

## 8. 持久化

SQLite `user_version=3`：

```text
whatif_snapshots(snapshot_id PK, schema_version, session_id, snapshot_json, snapshot_sha256, created_at)
whatif_variants(variant_id PK, snapshot_id, schema_version, variant_json, created_at)
whatif_results(result_id PK, variant_id, snapshot_id, schema_version, mode, settings_json, result_json, result_sha256, created_at)
```

- 迁移从 `user_version=2` 到 3：`CASE WHEN user_version<3` 创建上述表并设 3；重复启动幂等；不改 audit/agent 表；失败回滚。
- `whatif_variant_delete` 级联删除 result（或拒绝若被 compare 引用）；MVP 拒绝删除有结果的 variant。

## 9. UI

- 顶栏 `sim` 模式启用；进入后状态栏 `SIMULATION · snapshot`。
- Planning/Capabilities 增加 “What-if” 抽屉：
  - baseline 摘要；创建 variant；只暴露 patch 白名单字段（asset status/requires/lots/process verification）+ goal/deadline；
  - Run structure/plan；结果卡片显示 outcome/caveat/根因/依赖/plan fingerprint；
  - Compare 表：baseline + variants，按稳定排序，展示全部差异。
- 浏览器预览（非 Tauri）只显示 host-only 占位，不调用命令；Playwright **只测占位/禁用**，不做表单校验（表单只在 host 中可用）。
- 所有文案进入 i18n；SIMULATION 不能只用颜色。

## 10. 依赖与过程门禁

- `poie-whatif` 依赖：poie-ontology、poie-plan、poie-digest、poie-objects（可选）、serde/serde_json。
- 加入根 `members`；主机 `Cargo.toml` path dep；根/桌面 `Cargo.lock`、test-matrix/status-snapshot、desktop workflow `crates/**` 路径、ADR、cargo-deny。
- C1 不依赖 poie-state/poie-sim/poie-viability；C2 再加。
- 实施顺序：C0 schema/digest + DB migration → `poie-whatif` types/run → host commands → UI → Playwright placeholder → docs/checks。

## 11. 测试与验收

### 单元/集成

- Snapshot digest 稳定；id/createdAt 不影响 digest；model mismatch 拒绝。
- PatchDto 13 字段转换/未知字段/上限；同一 variant 两次运行 resultSha256 相同。
- structure/plan outcomes 分离，caveat 存在；plan failure 映射 `infeasible`。
- compare 排序稳定、完整差异；result 持久化恢复。
- 运行前后无 SAP/ActionLedger/federation 写入；DB 只写 whatif 表。
- 错误码/权限：未登录/scope 拒绝。

### UI

- Playwright：Sim 标记、host-only 占位、禁用状态；不测真实运行或表单校验。
- 截图更新。

### 验收

1. 可创建/运行/比较至少 3 个结构/计划变体；
2. 结果确定可复现；模型 digest 不匹配拒绝；
3. 无 live/SAP 写入；
4. UI 不会把模拟结果当 live；
5. C1 不声称 timed/promote 能力。

## 12. C2（明确延后，需重跑门禁）

- timed/storage/assetDelay：`TimedScenario::from_parts` 已存在；真正缺口是 `AssetTimeline` 公共字段/shift、startMode、storage capacity/initial/maxCharge 与 `assetDelayS` 的“修复延迟但不延迟故障”语义、`TimedRunner` 的 CaseDoc expect/evidence validity。
- promote：需要 B1 审批 token 通用化、`plan_to_proposed`、Authorization.basis 定义、SAP 授权命令；C2 另行设计。
- checkpoint/result registry 扩展。

## 13. 参考

- [PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)
- `poie-ontology::Patch`、`poie-plan::plan`、SAP §7.3
