# Phase C 设计：What-if / 场景推演工作区

- 状态：设计草案（待独立校验后进入开发）
- 日期：2026-11-04
- 范围：Phase C / 状态快照、扰动、模拟、对比、提升为计划
- 上游：Phase A 对象/动作目录、Phase B Agent 治理、现有 `poie-plan` / `poie-sim` / VEC
- 下游：Phase D 站点交付与证据版本绑定

## 1. 背景与问题

Poieweave 已有三类“假设”能力，但彼此割裂：

- 结构能力图回答“有没有路线”；
- `poie-plan` 给出一个计划，`poie-sim` 可以跑时间线/电池仿真；
- VEC 提供可行性/扩张核算，但主要在实验报告里；
- 桌面 cockpit 的 `Sim` 模式是禁用的，用户无法在当前对象上做 what-if。

Phase C 要把“如果泵晚 2 小时修好 / 如果备用泵不存在 / 如果储能只有 1 kW 放电 / 如果截止时间提前”变成
**不接触 live 状态的可重复实验**，并允许把一个被批准的变体提升为正式计划。

## 2. 目标

1. 定义 **Scenario Snapshot**：不可变基态（模型、状态、目标、时间线、证据版本）。
2. 定义 **Variant**：相对基态的有界扰动（资产状态、库存、质量、储能功率、目标/截止时间）。
3. 在隔离命名空间运行变体：结构求解/规划/SAP 预览/可选时间线仿真；结果带 digest。
4. 并列比较 baseline 与多个变体：verdict、计划动作、外部依赖、完成时间、失败根因、证据引用。
5. **Promote**：操作员选择变体结果，重新校验后创建 proposed SAP 动作；不自动派发。
6. UI、API 和评测共享同一套确定性结果。

## 3. 非目标

- 不做任意 Python/脚本执行；扰动只允许预定义字段。
- 不把模拟结果直接写入 live projection/ledger/状态。
- 不做无限分支搜索或自动优化器；MVP 是显式变体比较。
- 不替代 VEC 的正式统计/采样方法；Phase C 先提供确定性单变体/有限变体结果。
- 不做跨站点共享模拟状态（Phase D 再定义站点策略）。

## 4. 用户与场景

| 场景 | 操作 |
| --- | --- |
| 泵延迟修复 | 给 `pump-p1` 增加 `assetDelayS`（修复完成推迟），运行 timed/plan，比较是否仍满足截止 |
| 备用泵故障 | 把 standby 资产设为 `failed`，观察 `requires_any` 与服务 verdict（不删除资产） |
| 储能功率受限 | 降低 `electric-energy` 最大放电功率，观察 `power_limited` |
| 隔离品放行 | 把 quarantined lot 改为 accepted 资源/走 release 工艺，比较计划 |
| 多方案比较 | 创建 A/B/C 变体，按完成时间、外部依赖、阻断根因排序 |
| 提升 | 选择变体 → 重新校验 → 生成 proposed actions |

## 5. 领域模型

### 5.1 Snapshot

```text
ScenarioSnapshot {
  schemaVersion, id, label, modelId, modelSha256,
  baseState: {
    assetStatus, lots, storageLimits { resource, capacitySi, initialSocSi, maxDischargeW },
    assetTimelines, stockTimelines, evidenceVersion, stateId?
  },
  nowS, seed, horizonS, maxDepth, maxActions, evidenceValidityS,
  goal, minVerification, startS, deadlineS?,
  createdAt, createdBy
}
```

MVP 基态来自 host 启动时的 Base-01 + degraded 快照。**C MVP 明确只针对这个静态演示快照**：没有可变 live 模型/证据，因此不声称“live 状态变化后 stale”；`stateId` 可为 `None`，`stale_snapshot` 只在 host 的 model/evidence digest 变化（例如 Phase D 导入后重启）时触发。删除“模拟前后 live 不变”的假阳性断言，改为“C 不写任何 ActionLedger/DB/federation”。

### 5.2 Variant 与 Perturbation

变体由两部分组成：`Patch`（模型层，复用 `poie-ontology::Patch`）+ `Perturbation`（时间/储能/目标层，不污染 Patch 契约）：

```text
Variant {
  id, snapshotId, label, description, schemaVersion,
  patch: PatchDto {   // 与 poie-ontology::Patch 一一对应的公开 DTO；不是第二份语义
    assetStatus, assetRequires, assetRequiresAny,
    lotQuantity, lotResource, lotQuality,
    processVerification, addAsset, addResource, addLot, addProcess
  },
  perturbation: Perturbation {
    storageDischargeW: map<resourceId,number>,   // 构造 TimedScenario
    assetDelayS: map<assetId,number>,            // 资产可用/修复整体推迟（timed 变体）
    goal: GoalDoc?,
    minVerification: Verification?,
    deadlineS: number?
  },
  createdAt, createdBy
}
```

限制：最多 N 个变体（MVP 8）；每个 patch/perturbation 字段数量上限；拒绝未知 ID/单位/维度和负值；变量不能引用另一个变体的结果。`PatchDto` 只在 `poie-whatif` 内转换为 `poie-ontology::Patch`，字段由转换测试逐项对齐；领域类型不加 `Serialize`。

`poie-sim` 目前没有 checkpoint/序列化 API；如果 MVP 要“从当前状态快照开始”而不是重跑固定轨迹，必须先增加 `Checkpoint {time_s, assets, stocks, batteries, external_flows, digest}` 与 `Simulation::checkpoint/from_checkpoint`，否则 Phase C 只能做结构/计划变体，timed 变体从固定场景起点重跑并明确标注。

`assetDelayS` 不是 `Patch` 字段：由 `poie-whatif` 构造带时间偏移的 `AssetTimeline`/`TimedScenario` 或调度输入；如果 timed 变体尚未实现，UI 必须将该变体标为 `structure/plan only`，不能显示 `power_limited` 等时间结论。

### 5.3 结果

```text
OutcomeKind {
  mode: structure | plan | timed,
  structural: candidate | blocked | unknown | search_limit | null,
  timed: feasible | blocked | unknown | search_limit | too_late |
         insufficient_resource | capacity_conflict | power_limited | null,
  caveat: "blocked/candidate 只相对当前快照与有界求解器，不是物理不可行证明"
}

VariantResult {
  schemaVersion, variantId, snapshotId, runId,
  outcome: OutcomeKind,
  planFingerprint?, planSha256?, completionS?,
  resultSha256, actions[], rootCauses[], externalDependencies[],
  sapPreview { checks[], assumed[], rejections[] }?,
  evidenceRefs[], warnings[]
}
```

`resultSha256` 由 [DIGEST](DIGEST.md) 的 `poie.variant/0.1` 生成，覆盖 snapshotSha256 + canonical patch/perturbation + 运行设置；同输入同输出。结构候选与时序可行必须分开渲染，禁止把 `candidate` 显示成 `feasible`。

## 6. 架构

```text
crates/poie-whatif         NEW: snapshot/variant/patch/run/compare/promote
  ├── poie-ontology        Model + Patch + graph solve + schedule
  ├── poie-plan            plan + SAP preview
  ├── poie-sim             optional timed/storage simulation
  ├── poie-objects         Phase A object/action refs
  └── poie-viability       optional VEC metrics (MVP 不必须)
        │
        ▼
apps/desktop/src-tauri      commands + persistence + audit
apps/desktop/src           What-if workspace UI
```

`poie-whatif` 不依赖 Tauri；host 负责 DB、auth、command 和 audit。

### 6.1 运行流程

1. `snapshot_create`：从当前 domain 构建 snapshot（只读，记录 digest）。
2. `variant_create`：校验 patch，存 DB（或内存 + DB JSON）。
3. `variant_run`：`Model::patched` → graph solve → plan → SAP preview（可选 timed sim）；不写 live。
4. `variant_compare`：取 baseline 与多个结果，按统一排序生成差异。
5. `variant_promote`：重新运行/复用结果，检查 snapshot 仍匹配当前 live 模型与证据版本，创建 proposed actions；返回 SAP 校验；不 dispatch。
6. 所有操作需要登录；promotion 需要操作员确认（复用 Phase B 的审批语义）。

### 6.2 隔离与一致性

- 每个 variant 使用独立 `Model` 副本和独立 ledger/plan；绝不复用 live `ActionLedger`。
- C 的 variant/run 只在内存中计算，**不写 SAP 记录/ActionLedger**；如果未来需要仿真记录，必须使用 `Namespace::Simulation(branch_id)` 并通过 `ensure_live_dispatch` 闸门。UI 的 `Sim` 标签不是安全边界。
- 运行使用固定 `now_s`/snapshot 时间，避免 wall clock 改变结果。
- 如果 host 的 model/evidence digest 在 promote 前改变（例如 Phase D 导入后重启），拒绝 promote（`stale_snapshot`）并提示重建 snapshot；C MVP 没有可变 live state，不做“自动 rebase”。
- 模拟结果不进入 `execution_snapshot`/`federation_snapshot`；状态栏显示 `SIMULATION` 标记。
- `Sim` 模式在 UI 中启用后，所有写操作仍被禁止，直到显式 promote。
- 同一次比较必须列出 baseline 与全部 variant 的完整差异，不能只显示“最优项”；结构 `Verdict` 与 `TimedVerdict` 分别保存在 `OutcomeKind.structural/timed`，不得合并成 VEC 的 `VerdictKind`。

## 7. API

| 命令 | 输入 | 输出 |
| --- | --- | --- |
| `whatif_snapshot_create` | 无 | Snapshot |
| `whatif_snapshot_list` | limit | Snapshot[] |
| `whatif_variant_create` | snapshotId,label,patch | Variant |
| `whatif_variant_list` | snapshotId | Variant[] |
| `whatif_variant_run` | variantId, mode (`structure|plan|timed`) | VariantResult |
| `whatif_compare` | snapshotId, baselineVariantId?, variantIds[] | CompareResult |
| `whatif_variant_promote` | variantId, approvalTokenId | PromotionResult（proposed actions + SAP preview） |
| `whatif_variant_delete` | variantId | ok |

```text
CompareResult {
  schemaVersion, snapshotId, baselineResultId,
  rows[{ variantId, outcome, completionS?, planFingerprint?, delta, rootCauses[] }],
  sortKey: "completionS|verdict|variantId", order: "asc", ties: "variantId"
}
PromotionResult {
  schemaVersion, variantId, snapshotId, planFingerprint, planSha256,
  proposedActions[], sapPreview{checks[],assumed[],rejections[]},
  approvalTokenId, auditId
}
```

`ApprovalToken` 使用 [PLATFORM](PLATFORM.zh-CN.md) §6；输入不接受布尔 `approved`。错误码：`unknown_snapshot`、`invalid_patch`、`stale_snapshot`、`approval_required`、`approval_mismatch`、`approval_expired`、`not_promotable`、`budget_exceeded`、`unsupported_schema`。

### 7.1 只创建 proposed 的 SAP API

C 的 promote 不能调用 `validate_plan_through_sap`（它会创建授权并推进到 `authorized`）。新增/使用只读或提案 API：

- `poie-plan::preview_plan_through_sap`（A 已定义）用于预览；
- `poie-plan::plan_to_proposed`（C 新增）只创建 `ActionState::Proposed`，不写 `Authorization`、不 dispatch；返回 `ProposedActions`；
- promote 流程：stale 检查 → 重新规划 → `plan_to_proposed` → 展示 SAP preview/审批 → 成功后交给现有 SAP 验证命令；C 不直接 authorize/dispatch。

### 7.2 timed/storage 变体构造

`TimedScenario` 当前的 `storage_discharge_w` 没有受校验的 setter，且 capacity/initial SoC 来自场景文件。C 必须增加：

- `TimedScenario::with_storage(id, resource, capacity, initialSoc, maxChargeW, maxDischargeW)`：校验能量单位、容量/SoC 上界、非负功率、重复资源；
- `Snapshot.baseState.storageLimits` 保存同样字段；
- `storageDischargeW` 变体只修改已存在 storage 的 `maxDischargeW`，不可新建无 capacity 的 storage；
- 若要“从当前时刻”运行，使用 checkpoint 或从固定场景起点重跑并显式标注 `startMode=scenario-start`。

### 7.3 持久化

MVP 持久化 `snapshots`/`variants`（DTO JSON）到 SQLite，`runs` 只存内存 + 摘要/审计；DB 用 `user_version` 迁移。`whatif_variant_list` 恢复变体但结果需重跑，不持久化可执行的 live 状态。

## 8. UI

### 8.1 入口

- 前端顶栏 `sim` 模式启用（当前只有 TS `Mode` 联合类型，host 需增加 mode 状态），点击后打开 What-if 抽屉；
- Planning 视图增加“新建变体/比较”入口；
- 状态栏显示 `SIMULATION · <snapshot>`，与 live 明确区分。

### 8.2 工作区

```text
┌ Baseline（当前） ┐ ┌ Variant A ┐ ┌ Variant B ┐
│ verdict / 完成时间 │ │ ...       │ │ ...       │
│ 计划动作          │ │ ...       │ │ ...       │
│ 阻断/依赖         │ │ ...       │ │ ...       │
└──────────────────┘ └───────────┘ └───────────┘
        [运行] [比较] [提升为计划]
```

- Variant 编辑器只暴露白名单字段，带单位/ID 校验；
- 结果变化用文字 + 图形双编码（不能只靠颜色）；
- 失败展示根因/证据引用，不隐藏不确定；
- “提升”显示影响摘要、SAP 复核结果、审批按钮；成功后跳转 Planning/Execution。

## 9. 安全与失败模式

| 风险 | 控制 |
| --- | --- |
| 模拟污染 live | 独立 Model/ledger，禁止写 execution/federation/state |
| 变体注入任意字段 | 白名单 patch + schema + 上限 |
| 过期 snapshot 提升 | 比较 model digest/evidence version，过期即拒绝 |
| 误提升 | 审批 + 显示计划/影响 + 可撤销 proposed 动作 |
| 结果不可复现 | 固定 snapshot/now/settings，digest 校验 |
| 资源耗尽 | 变体数/动作数/运行时间上限 |
| UI 误读 | `SIMULATION` 标记、状态栏、颜色+文字 |

## 10. 测试与验收

### 单元/集成

- patch 校验（未知 ID、单位、负值、上限）；
- 同一 snapshot+patch 两次运行 digest 相同；
- 运行前后 `ActionLedger`/`execution_snapshot`/`federation_snapshot` 无新增记录（C 不写 SAP/ledger）；
- 比较排序确定；
- stale snapshot promote 被拒绝；
- promote 只创建 proposed actions，SAP 校验不通过时拒绝；
- timed/storage 变体产生 `power_limited` 等结论；
- 审计事件完整。

### UI

- Playwright：打开 Sim、创建变体、运行、比较、看到 `SIMULATION` 标记；未登录/浏览器预览禁用；
- 截图更新。

### 验收标准

1. 用户可以创建/运行/比较至少 3 个变体，结果确定且可复现；
2. C 不写 `ActionLedger`/`execution_snapshot`/`federation_snapshot`，测试断言无新增记录；没有可变 live state 的假阳性断言；
3. 提升必须重新校验并经过操作员确认；
4. UI 不可能把模拟结果误认为 live；
5. 至少覆盖一个 `power_limited` 和一个 `requires_any` 变体。

## 11. 实施顺序

1. `poie-whatif` 类型/校验/运行/测试；
2. host DB/commands/audit；
3. UI Sim 模式和变体编辑器；
4. 比较视图与提升；
5. timed/storage 变体；
6. Playwright/截图/文档/测试矩阵。

## 12. 开放问题

- 变体结果是否需要持久化到 SQLite；MVP 运行结果存内存，只存定义/摘要。
- 是否允许多个快照/场景；MVP 只有当前 Base-01 snapshot，Phase D 可扩展站点。
- 是否引入 VEC 指标；MVP 先 verdict/完成时间/依赖/根因，后续再加统计。
- 提升的审批是否复用 Phase B Agent approval；MVP 用同一操作员确认语义，不要求 Agent。
- 多站点 what-if 的范围：MVP 只做单站点；如果展示联邦库存/租约影响，必须显式标注只读快照与范围，不得写回站点。
- 禁止自动 TTL/rebase 删除场景；场景元数据可追溯，清理只能显式操作并留审计。

## 13. 参考

- Palantir Ontology 决策层与场景对比思路（Phase A 参考链接）
- `poie-sim` timed/storage、`poie-plan`、VEC 契约
- `specifications/physical-ontology.zh-CN.md` §3.8–3.11
