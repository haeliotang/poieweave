# Phase A 设计：Ontology 产品层

- 状态：设计草案（待独立校验后进入开发）
- 日期：2026-11-04
- 范围：Phase A / 本体对象、链接、动作目录、类型化 API、SDK 与桌面对象工作台
- 上游：[Physical Ontology](../../specifications/physical-ontology.zh-CN.md)、[SAP](../../specifications/state-action-protocol.zh-CN.md)
- 下游：Phase B（Agent 治理）、Phase C（What-if）、Phase D（站点交付）

## 1. 为什么要做这一层

Poieweave 已有比普通“语义层”更严格的物理本体：资产、资源、库存、工艺、服务、能力、验证等级、质量类、时间线，以及可执行的 SAP 动作和七项派发复核。但当前这些能力主要通过 Rust API、静态场景和桌面快照暴露：

- 前端只认识 `DomainOverview` 的固定字段，新增实体类型要改 TypeScript；
- 没有稳定的“对象类型 / 链接类型 / 动作类型 / 函数类型”产品契约；
- 场景、实验和桌面视图各自组装数据，外部应用没有可依赖的 SDK；
- 动作的前置条件、参数、权限、审批和证据要求没有一处可枚举的目录。

Phase A 把这些能力收束成一个**可读、可预览、可类型化**的产品层。它不是把正式契约降级成低代码本体，而是在正式模型之上增加稳定的产品投影。

## 2. 目标

1. 从 `poie-ontology::Model` 自动生成确定性的 **Ontology Catalog**：对象类型、链接类型、属性 schema、函数目录。
2. 提供只读的 **Object API**：按类型/ID 查询对象、链接、属性、证据和版本；不直接修改 live 状态。
3. 提供 **Action Catalog**：把工艺动作变成可枚举的动作类型，包含模型可导出的输入/输出/占用/恢复/时长/验证等级；审批/风险/参数校验策略由 Phase B 的治理层补充，不在 Catalog 里硬编码。
4. 生成 TypeScript 类型/SDK，让桌面前端和外部脚本使用同一套契约，避免手写快照类型漂移。
5. 在桌面 Capabilities/Planning 中提供对象浏览器、链接关系、属性与动作预览；所有提交仍走 SAP。

## 3. 非目标

- 不做通用低代码建模器；对象类型来自正式模型，不在 UI 里随意新建。
- 不做通用图数据库；当前数据量小，确定性投影足够。
- 不允许 Phase A 直接派发设备命令；动作只能预览/提交给 SAP，执行端仍显式未接入。
- 不改变 PO/SAP/VEC 的语义；Catalog 是投影，不是第二套本体。
- 不做跨版本 schema 迁移平台；只定义版本字段和拒绝未知版本的策略。

## 4. A0：宿主安全前置（Phase A 开发的第一步）

这些不是新功能，但必须在任何对象/动作 UI 之前修掉；否则 Phase A 的只读声明不成立。跨阶段 DTO/错误/scope/审批/模型约束见 [PLATFORM](PLATFORM.zh-CN.md)；当前模型固定为 Base-01 + `pump-p1=degraded` 的 `SourceModel`，host 必须持有 `Arc<Model>` 并计算 digest，不能只持有 `DomainOverview`。

1. `execution_gate_set` 必须先 `auth::require` 再改 `GateState`/`ExecutionSnapshot`，所有变更强制审计；当前“先改再可选审计”的路径（`apps/desktop/src-tauri/src/lib.rs`）删除。
2. `sandbox_read/write` 增加会话、路径/大小限制；A0 将 **read 和 write 都改为 no-follow**（`symlink_metadata`/`O_NOFOLLOW`，拒绝任何 symlink；write 再使用临时文件/原子 rename），并添加“根内符号链接指向根外”的逃逸负例。删除独立 `sandbox_root` 命令（统一由 `host_info.sandboxRoot` 返回），或把它登记到 PLATFORM 矩阵；当前选择**删除**。定义命令 scope 矩阵：哪些窗口/命令允许读写、哪些只读、哪些要求登录。
3. 增加宿主命令白名单测试：未登录/无 scope/错误窗口调用 `execution_gate_set`、`sandbox_write`、`inference_configure`、`agent_*` 必须拒绝并留痕；`auth_status/register/sign_in/sign_out` 是**唯一无需先登录的 bootstrap 例外**。
4. `execution.rs` 和 `federation_view.rs` 的硬编码授权（`Authorization{by:"maintainer", basis:"issue #105"}`）从产品路径移除；现有 `execution.rs` 测试中断言 `review_status=="ready"`/全部 `authorized` 的部分必须改为 `review_status=="blocked"`、`rejections==actions.len()`、`authorized/dispatched=0`。同时清理浏览器 fallback：`apps/desktop/src/domain.ts` 的 `FALLBACK_EXECUTION` 改为 `state="synthetic-preview"`、`reviewStatus="blocked"`、`dispatchAllowed=false`、checks 不全 passed；`App.tsx`/`i18n.ts` 不再显示“复核通过/已授权”，显示“合成预览/未授权”。A0 **不生成任何 Authorization**：execution 视图把现有动作标为 `synthetic-preview`，只显示 proposed/validated 与“未授权”状态；测试断言 `authorized`/`dispatched` 数量为 0，UI 不再显示“复核通过”，未登录时同样只读。
5. 命令错误改用 `{code,message,details}` 包络；命令 scope 矩阵见 PLATFORM §4，不新增裸 `String` 错误。

A0 的验收进入 Phase A PR；没有它，Phase A 不开始。**preview 语义**：A0 阶段 `dispatch_allowed` 在 `ExecutionSnapshot` 中恒为 `false`（三态 `false`），UI 显示“合成预览/未授权”，不得用 `true` 表示“复核通过”；只有后续显式授权流程才可能改变。测试需断言：execution snapshot 中 `authorized`/`dispatched` 为 0；`execution.rs`/`federation_view.rs` 不再出现硬编码 `Authorization`；未登录调用 gate/sandbox 被拒；命令 scope 矩阵用例通过。

## 5. 用户与使用场景

| 用户 | 场景 |
| --- | --- |
| 现场操作员 | 在 Capabilities 中查资产/资源/服务，理解状态、外部依赖和证据 |
| 规划者 | 从 Action Catalog 查看一个工艺动作的输入/输出/占用/恢复、验证等级和前置条件 |
| 桌面内开发者 | 通过 Tauri IPC + 生成式 TypeScript SDK 使用对象/动作类型；外部网络 API 不在 MVP 范围 |
| Agent（Phase B） | 通过同一 Catalog 获得工具 schema 和权限元数据，不允许自造数据源 |
| 审计/评审 | 查看对象类型和属性来源，确认 UI 没有隐藏语义 |

## 6. 领域模型

### 6.1 对象类型

首批对象类型（由 Model 自动生成）：

| 类型 ID | 来源 | 关键属性 |
| --- | --- | --- |
| `asset` | `model.assets` | status、status_evidence、requires、requires_any、provides、capability_verification |
| `resource` | `model.resources` | unit、mass_per_unit、quality |
| `lot` | `model.lots` | resource、quantity_si、quality（MVP 不暴露未进入 `Lot` 结构的 `location/evidence`） |
| `process` | `model.processes` | duration、verification、requires_capabilities、occupies、restores |
| `service` | `model.services` | requires、requires_any |
| `capability` | 资产 `provides` 去重 | provider_count、verification_summary |
| `evidence` | 模型/实验/合同引用 | kind、digest、validity、source |

MVP 先实现前六类；`evidence` 对象只返回引用，不新建证据账本。

**属性证据**：`source` 的枚举固定为 `model | declared | observed | evidence`。模型结构直接导出的字段（id、unit、duration、verification 等）`source=model`；`Asset.status_evidence` 为 `Some` 时为 `source=evidence`、`asOf` 取证据时间（未知则 null）；为 `None` 时为 `source=declared`，表示模型作者的合成观测（PO §3.1）。`asOf` 来自 host 快照时间（`nowS`）；没有证据时间的属性 `asOf=null`。UI 必须区分 model/declared/observed/evidence，不能把声明显示为观测。`Lot.location/evidence` 不进入 MVP（验证后的 `Lot` 无这些字段）。

### 6.2 链接类型

| 链接 ID | from → to | 语义 |
| --- | --- | --- |
| `asset.requires` | asset → asset | AND 支撑 |
| `asset.requires_any` | asset → asset | OR 支撑组 |
| `asset.provides` | asset → capability | 能力提供 |
| `process.input` | process → resource | 输入流 |
| `process.output` | process → resource | 输出流 |
| `process.occupies` | process → asset | 占用 |
| `process.restores` | process → asset | 恢复 |
| `process.requires_capability` | process → capability | 能力要求 |
| `service.requires` | service → asset | 必需资产 |
| `service.requires_any` | service → asset | 替代支撑 |
| `lot.of_resource` | lot → resource | 批次所属资源 |

链接方向固定、按稳定 ID 排序；枚举结果必须与模型文件顺序无关。

### 6.3 动作类型

动作目录只包含 **process id 本身**（例如 `replace-sleeve-p1`，不加 `process.` 前缀），由 `Model.processes` 生成。SAP 生命周期动作（validate/authorize/dispatch/receipt/verify）是 SAP 契约的静态集合，**不进入 Ontology Catalog**；Phase B 的工具目录另行映射，不能从 Catalog 调用 dispatch。

每个动作类型只包含**模型可导出**的字段：

```text
id (即 process id), target_types,
inputs[] (resource, quantity, unit),
outputs[] (resource, quantity, unit),
occupies[], restores[], requiresCapabilities[], durationS?, requiredVerification
```

`target_types` 的派生规则：`occupies ∪ restores` 非空时为 `["asset"]`（并携带具体 asset IDs），否则为 `["process"]`；`inputs/outputs` 来自模型流，`durationS` 来自 `Process.duration_s`，`requiresCapabilities` 来自 `Process.requires_capabilities`，均不是调用者可改的任意参数。这些字段直接来自 `Process`，不是通用参数表；不提供“任意参数/任意字段”的动作表单，也不把 `runs` 等规划变量包装成参数。`risk_class`、`approval_class`、`allowed_in_modes`、`executor_kind`、`budget_template` 属于 Phase B 治理层（按 `ActionType.id` 关联），缺失时 **fail-closed**，不进入可提交状态。

MVP 不生成新的 SAP 动作实例；仅把计划动作和目录元数据关联。Phase C 负责从 What-if 结果创建 proposed 动作。

### 6.4 函数目录

函数是只读或只生成提案的命名操作，不是可任意执行的后门：

| 函数 ID | 输入 | 输出 | 副作用 |
| --- | --- | --- | --- |
| `catalog.describe` | 无 | Ontology Catalog | 无 |
| `state.query` | subject/type, asOf | 对象/链接/能力/依赖 | 无 |
| `plan.propose` | goal, horizon, deadline | proposed plan | 无（只返回） |
| `plan.preview_sap` | planFingerprint（host 解析） | SAP validation summary（只读 dry-run，假定项显式标注） | 无 |
| `action.draft` | planFingerprint, actionId | 动作草稿/预览（需 Phase B 审批后执行） | 无 |
| `answer.emit` | claims, suggestions | 最终回答协议（Phase B） | 无 |

Phase A 实现前四项；`action.draft`/`answer.emit` 由 Phase B 治理层实现。Phase B 的 Tool Catalog 是这六个函数的**唯一投影**（同名/同参数），不得另造一套工具语义。

## 7. 架构与模块边界

```text
crates/poie-ontology       正式物理模型、graph solve、schedule
        │
        ▼
crates/poie-objects        NEW: Catalog / Object projection / Action catalog
        │
        ├── poie-digest     NEW: canonical JSON + SHA-256（A/C/D 共用）
        ▼
        │  (pure Rust, no Tauri)
        ├──> apps/desktop/src-tauri   host commands + settings
        └──> crates/poie-sdkgen (binary) -> apps/desktop/src/sdk/ontology.ts
                 (唯一生成路径；CI: cargo run -p poie-sdkgen -- --check)
```

### 7.0 Digest 规范（A1 前冻结）

新增 `crates/poie-digest`：canonical JSON + SHA-256，不依赖 Tauri、不访问网络。A/C/D 所有 digest 必须使用同一实现。

- **canonical JSON**：UTF-8；对象键按字典序；数组保持业务稳定顺序（按 id 排序，除非语义是顺序列表）；无多余空白；数字只接受有限值，f64 用 `serde_json` 的稳定表示；禁止 NaN/Infinity。
- `model_digest = sha256("poie.model/0.1\n" + canonical_json(ModelCanonical))`，`ModelCanonical` 由已验证的 `Model` 字段构建（资源/资产/批次/工艺/服务按 id 排序；`&'static Unit` 只输出 symbol/dim/scale）。
- `catalog_digest = sha256("poie.catalog/0.1\n" + canonical_json(catalog_without_digest))`。
- `plan_digest` 保留现有 `Plan::digest()` 字符串作为人类可读标识；需要 SHA-256 时使用 `sha256("poie.plan/0.1\n" + canonical_json({digest, actions...}))`，不要声称两者相同。
- `simulation_digest` 同样保留 `Simulation::digest()` 字符串，D/C 记录字符串并附哈希。
- `manifest_sha256 = sha256("poie.manifest/0.1\n" + canonical_json(manifest without integrity))`。

验收：同一模型两次生成 `model_digest`/`catalog_digest` 相同；修改一个字段 digest 必变；未知 schema 或非有限数字拒绝。该规范见 [`DIGEST.md`](DIGEST.md)，由 A/C/D 共享测试向量。

### 7.1 新 crate `poie-objects`

职责：

- 只读投影：`Model` → `OntologyCatalog`、`Vec<ObjectInstance>`、`Vec<LinkInstance>`；
- 动作目录：仅从 `Model.processes` → `Vec<ActionType>`；不依赖 `poie-plan`，不生成 SAP 授权；
- 属性证据：每个属性带 `source`、`asOf`、`verification`，声明状态必须标注 `declared/synthetic`（PO §3.1）；
- 确定性：排序、digest、schema version；
- 不依赖 Tauri、不读写数据库、不派发；动作预览本身不调用 SAP 写路径。host 负责生成只读 dry-run 的 `SapPreview`（假定项显式标注）并作为输入传给 Catalog/UI，`poie-objects` 不伪造授权，也不显示“7/7 已通过”。

新增 `crates/poie-sdkgen`（workspace 二进制）：读取 Catalog 生成 SDK 与 schema，生成物提交；CI 重生成后零 diff。生成器本身必须可测试，不能只依赖易漂移的 Python 脚本。

建议 API：

```rust
pub struct PropertyDef {
    pub id: String,
    pub label: String, // 确定性 humanize(id)；不是模型字段
    pub kind: String,            // string|number|quantity|enum|boolean|ref
    pub unit: Option<String>, pub required: bool, pub read_only: bool,
    pub source: String,          // model|declared|observed|evidence（与 PropertyValue 同一枚举）
}
pub struct PropertyValue {
    pub value: serde_json::Value, // 字符串/数字/枚举；不得放自由文本结构
    pub source: String,           // model|declared|observed|evidence
    pub as_of: Option<String>,
    pub verification: String,    // declared|modeled|demonstrated|observed
}
pub struct LinkType {
    pub id: String, pub from: String, pub to: String,
    pub cardinality: String, // "many_to_many"（MVP 所有链接）；未来 1:1 需显式声明
}
pub struct LinkInstance { pub type_id: String, pub from: String, pub to: String }
pub struct EvidenceRef {
    pub kind: String, pub id: String,
    pub fingerprint: Option<String>, pub sha256: Option<String>, // 至少一个非空
    pub as_of: Option<String>, pub source: String,
}
pub struct ObjectType {
    pub id: String,
    pub label: String, // 确定性 humanize(id)；不是模型字段
    pub properties: Vec<PropertyDef>, pub links: Vec<LinkType>,
}
pub struct ObjectInstance {
    pub id: String, pub type_id: String,
    pub version: String, // MVP = model_digest（快照版本，不是逐对象修订号）
    pub properties: BTreeMap<String,PropertyValue>,
}
pub struct ObjectDetail {
    pub schema: String, // poie.object-detail/0.1
    pub model_digest: String, pub catalog_digest: String,
    pub object: ObjectInstance, pub links: Vec<LinkInstance>,
    pub evidence: Vec<EvidenceRef>,
}
pub struct ActionType {
    pub id: String, // 即 process id；UI 可本地 humanize，不伪造 label/description 字段
    pub target_types: Vec<String>, pub inputs: Vec<FlowView>, pub outputs: Vec<FlowView>,
    pub occupies: Vec<String>, pub restores: Vec<String>,
    pub requires_capabilities: Vec<String>,
    pub duration_s: Option<f64>, pub required_verification: String,
}
pub struct FlowView { pub resource: String, pub quantity: String, pub unit: String }
pub struct FunctionType {
    pub id: String, pub description: String,
    pub input_schema: serde_json::Value, pub output_schema: serde_json::Value,
    pub effect: String, // read_only|propose|simulate
}
pub struct OntologyCatalog {
    pub schema: String,          // "poie.ontology-catalog/0.1"
    pub model_digest: String,    // sha256 per DIGEST
    pub catalog_digest: String,  // sha256 of catalog excluding this field
    pub object_types: Vec<ObjectType>,
    pub link_types: Vec<LinkType>,
    pub action_types: Vec<ActionType>,
    pub functions: Vec<FunctionType>,
}

pub struct ObjectPage {
    pub schema: String,
    pub model_digest: String,
    pub catalog_digest: String,
    pub objects: Vec<ObjectInstance>,
    pub next_cursor: Option<String>,
}

// SapPreview/CheckPreview/AssumedContext/PreviewError 的权威定义在 `poie-plan::sap`；
// 这里只列字段形状，不在 poie-objects 复制实现：
//
// SapPreview { checks: Vec<CheckPreview>, assumed: Vec<String>, rejections: Vec<String> }
// CheckPreview { id, status: observed|assumed|failed|unknown, detail }
// AssumedContext { ownership, reservations, authorization, now_s }
// PreviewError { InvalidPlan, UnsupportedContext, NoPreviewAvailable }
// 映射：InvalidPlan -> invalid_parameter；UnsupportedContext -> preview_unavailable；
// NoPreviewAvailable -> preview_unavailable；host 统一包成 {code,message,details}。

pub struct PlannedActionView {
    pub action_id: String, pub process: String, pub target: String, pub runs: u32,
    pub start_s: f64, pub end_s: f64, pub preconditions: Vec<String>,
    pub allocations: Vec<(String, f64)>, pub occupies: Vec<String>,
    pub restores: Vec<String>, pub verification: String,
}
pub struct ActionPreview {
    pub schema: String, // poie.action-preview/0.1
    pub model_digest: String, pub catalog_digest: String,
    pub plan_fingerprint: String,
    pub action: PlannedActionView,       // 来自 poie-plan::PlannedAction
    pub sap_preview: Option<plan::SapPreview>, // host 传入的只读 dry-run；假定项显式标注
    pub assumptions: Vec<String>,
    pub reasons: Vec<String>,
}
```

所有序列化结构使用 `serde(rename_all = "camelCase")`，字段有稳定顺序；未知 schema 版本必须拒绝。公开 API 使用 DTO（不直接给领域类型加 `Serialize`），错误统一 `{code,message,details}`，见 [PLATFORM](PLATFORM.zh-CN.md) §2–§4。

`SapPreview`/`CheckPreview`/`AssumedContext`/`PreviewError` 定义在 `poie-plan::sap`（不放在 `poie-objects`）；`ActionPreview`/`PlannedActionView` 是 host `objects.rs` 的 DTO，不属于 `poie-objects`，因此不存在 `poie-objects -> poie-plan` 边。`poie-objects` 只生成 Catalog/Object/Link。

### 7.2.1 只读 SAP preview API

现有 `poie-plan::validate_plan_through_sap` 会创建 `Authorization{by:"planning-validation"}` 并把动作推进到 `authorized`，不能用于预览。Phase A 新增：

```rust
pub fn preview_plan_through_sap(
    plan: &Plan,
    assumed: &AssumedContext,
) -> Result<SapPreview, PreviewError>;
```

- 只读：不写 `ActionLedger`、不创建/保存 `Authorization`、不产生 transition；
- `SapPreview { checks: Vec<CheckPreview>, assumed: Vec<String>, rejections: Vec<String> }`；
- `CheckPreview { id, status: observed|assumed|failed|unknown, detail }`；ownership/reservations/authorization 默认 `assumed`；
- 测试断言：调用前后 ledger/authorization 计数不变；所有 `assumed` 项在 UI 中单独显示，不能显示为“7/7 通过”。
- host 的 `ontology_action_preview` 调用该函数并把结果传给 `poie-objects`/UI，`poie-objects` 不依赖 `poie-plan`。

### 7.2 桌面 host 命令

| 命令 | 输入 | 输出 |
| --- | --- | --- |
| `ontology_catalog` | 无 | `OntologyCatalog` |
| `ontology_objects` | typeId, cursor?, limit? | `ObjectPage` |
| `ontology_object` | typeId, id | `ObjectDetail`（属性+链接+证据引用） |
| `ontology_action_preview` | plan object（host 持有）或 planId, actionId, assumedContext? | `ActionPreview`（基于 PlannedAction 的只读 dry-run） |
| `ontology_sdk_schema` | 无 | 生成 SDK 所需的 JSON schema |

要求：

- 只读；不写 SQLite，不改变 `GateState`/`ExecutionSnapshot`；
- 返回 `modelDigest` 和 `catalogDigest`，前端切换场景时校验一致性；
- 查询必须可重复，不能依赖 HashMap 迭代顺序；
- 错误使用 [PLATFORM](PLATFORM.zh-CN.md) §3 的 `{code,message,details}` 包络；本命令至少覆盖 `unknown_type`、`unknown_object`、`unsupported_schema`、`invalid_parameter`、`invalid_cursor`、`limit_exceeded`、`model_mismatch`、`preview_unavailable`、`scope_denied`。

### 7.3 TypeScript SDK

生成器 `crates/poie-sdkgen` 依赖 `poie-objects`，从固定的 `scenarios/base-01/model.toml` 构造 Catalog（与 host 使用同一 Model），不读取未验证的临时 JSON。输出：

- `apps/desktop/src/sdk/ontology.ts`：对象/链接/动作/函数类型与字符串字面量 ID；
- `apps/desktop/src/sdk/client.ts`：对 host 命令的薄封装（仅 Tauri 环境）；
- `apps/desktop/src/sdk/README.md`：生成方法与版本约束。

CI 校验：运行 `cargo run -p poie-sdkgen -- --check`，重新生成后必须与提交文件一致（零 diff），否则退出码 2；未知 schema 版本退出码 3。生成器纳入 workspace、`cargo test`、`check-test-matrix` 计数；黄金文件为 `apps/desktop/src/sdk/ontology.ts` 与 `catalog.golden.json`。

MVP 不发布 npm 包；先把 SDK 作为桌面内共享类型层，后续可独立发布。

## 8. UI 设计

### 8.1 Capabilities 对象浏览器

- 左侧实体列表保留；选中后 inspector 显示：
  - 类型、ID、状态、版本/证据；
  - 属性表（带单位和验证等级）；
  - 链接分组（requires / provides / inputs / outputs / services）；
  - “在图中定位”与“查看动作”入口。
- 增加类型筛选（asset/resource/lot/process/service/capability），但保持现有布局和配色，不引入第七个 rail 图标。
- 对象图节点/边来自 Catalog：节点集合 = 当前服务可达的 asset/resource/process/service/capability 子集；边来自 `LinkInstance`。
- 布局连续：右列 service，中列 process，左列 asset/resource，按 stable id 排序并分层排布；不持久化手写坐标。删除 `GRAPH_NODES` 常量；测试断言节点/边集合等于 Catalog 子集，布局可重复。
- Playwright 只能跑浏览器预览；对象详情/动作预览通过 **mock Catalog fixture** 渲染，host 端用 Rust 集成测试覆盖真实命令；二者都纳入验收。

### 8.2 Planning / Execution 动作预览

- 动作行显示类型 ID、验证等级和前置条件；
- inspector 的 Action 详情增加 Catalog 参数和只读 `SapPreview`；假定项必须单独显示，不能显示为“已通过”；
- “提交”只进入 SAP proposed/validated 流程；Phase A 不新增派发按钮；
- 现有 `execution.rs`/`federation_view.rs` 中写死的 `Authorization` 必须从产品路径移除；A 只显示 proposed/validated 与 synthetic-preview，不生成授权。

### 8.3 可访问性与国际化

- 所有新控件有可见标签或 `aria-label`；
- 新文案进入 `src/i18n.ts` 的 zh/en 字典；
- 截图/Playwright 增加对象浏览器与动作预览的 smoke 用例。

## 9. 安全与失败模式

| 风险 | 控制 |
| --- | --- |
| Catalog 变成第二套语义 | 所有类型由 Model 生成，Catalog 带 model digest；测试对比模型字段 |
| 未知 schema 被误读 | 拒绝未知版本，前后端都校验 |
| 对象 ID 泄漏敏感信息 | Phase D 增加 markings；MVP 只存模型内 ID |
| 动作预览诱导直接派发 | 预览无 submit/can_submit 字段；没有 dispatch 命令；提交由 Phase C/现有规划路径进入 SAP |
| 大对象导致渲染卡顿 | 分页 cursor + limit 上限；服务端强制 |
| 前端手写类型漂移 | SDK 生成 + CI 重生成校验 |

## 10. 兼容与迁移

- 不删除现有 `domain_overview`/`execution_snapshot`；Phase A 新增并行命令，前端逐步迁移。
- 首个版本只读；A0 完成前，现有 `execution_gate_set`/`sandbox_*` 的鉴权缺口必须修掉；A 自身不新增写命令。
- 旧快照字段保留到 Phase C 完成后再评估删除。
- Catalog schema 版本独立于 PO 契约版本；两者都记录在 manifest。

## 11. 测试与验收

### 单元/集成

- Catalog 对 Base-01 模型生成稳定的对象/链接/动作计数；
- 同一模型两次生成的 catalog digest 相同；
- 未知 schema/未知对象/非法参数返回稳定错误码；
- 链接端点类型与数量校验；
- 每个 process 都有一个 process-id 动作，inputs/outputs/occupies/restores/durationS/requiresCapabilities/verification 与模型逐字段一致；不暴露通用参数；
- SDK 重生成一致；
- `preview_plan_through_sap` 前后 ledger/authorization/transition 计数不变，`assumed` 项单独标注；
- host 的 catalog/domain 使用同一 `SourceModel`（digest 一致），不能出现两个 pump 状态；
- host 命令未登录/scope 不符时拒绝（execution_gate_set、sandbox_write、inference_configure）；
- 对象图节点集合来自 Catalog，删除 `GRAPH_NODES` 后仍能渲染。

### 桌面/UI

- Playwright：Capabilities 打开对象详情、显示链接、Action 预览、中英切换；
- `pnpm build`、`cargo test`、`cargo clippy -D warnings`；
- host 命令的 JSON 字段快照测试。

### 验收标准

1. 桌面内消费者只依赖 Catalog/SDK 就能枚举对象、链接和动作；外部网络 API 另立阶段；
2. 前端不再手写 Base-01 对象列表；
3. 任何动作预览都无法绕过 SAP；
4. CI 能阻止 SDK/类型漂移；
5. 现有六个视图和实验重放不回归。

## 12. 实施顺序

0. A0：gate/sandbox/inference 鉴权与命令 scope 修复；`execution.rs`/`federation_view.rs` synthetic preview 替换；UI fallback 清理；`scripts/check-schema-registry.py` + 根 CI；测试；
1. `poie-digest` + `poie-objects` 数据模型 + Model 投影 + 测试；加入根 `members`/根与桌面 `Cargo.lock`/测试矩阵；写 ADR（sha2、serde、生成器），定义 limit 默认/上限（默认 100、最大 500/page；catalog ≤1 MiB；cursor 为 opaque string，缺省从头开始）；
2. host `objects.rs` 命令 + JSON/schema 快照 + 错误包络；
3. `poie-sdkgen` + CI 零 diff + 黄金文件；
4. Capabilities 对象浏览器与动作预览；
5. Playwright（Catalog mock/host E2E 策略）与截图；
6. `cargo fmt/clippy/doc/cargo-deny/check-test-matrix/check-docs`，文档/状态快照更新。

## 13. 风险与开放问题

- 对象/链接数量增长后是否需要分页/索引；当前模型规模小，MVP 用稳定排序 + limit。
- “动作类型”是否应该独立于 process，支持非工艺动作（审批、检查、迁移）；Phase A 先只做 process + SAP 目录，Phase B 再扩展。
- 是否公开 HTTP API；MVP 仅 Tauri IPC，外部 SDK 通过 host 命令/未来服务化，不引入网络监听。
- 证据对象是否进入 Catalog；MVP 只返回引用，Phase D 再做证据/版本绑定。

## 14. 参考

- Palantir Ontology（data + logic + action + security）：<https://www.palantir.com/docs/foundry/architecture-center/ontology-system/>
- Palantir architecture overview：<https://www.palantir.com/docs/foundry/architecture-center/overview/>
- Poieweave PO / SAP / VEC 契约与桌面 README（见文首链接）。
