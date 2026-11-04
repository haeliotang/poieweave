# Poieweave 仓库差距审计：Phase A–D

- 审计日期：2026-11-04
- 审计对象：`/Users/hao/Documents/poieweave`，HEAD `bee82b7`（`main`）
- 方法：只读检查实际仓库；`cargo test -- --list`（根 workspace 388、桌面宿主 12）、`cargo metadata`、CI/文档/源码交叉核对；未修改应用源码或 git 状态
- 对照设计：[Phase A](../phase-a-ontology-product.zh-CN.md)、[Phase B](../phase-b-agent-governance.zh-CN.md)、[Phase C](../phase-c-what-if.zh-CN.md)、[Phase D](../phase-d-site-delivery.zh-CN.md)、[设计目录说明](../README.zh-CN.md)
- 基线：[PO 契约](../../../specifications/physical-ontology.zh-CN.md)、[SAP 契约](../../../specifications/state-action-protocol.zh-CN.md)、[VEC 契约](../../../specifications/viability-expansion.zh-CN.md)、[状态快照](../../status/status-snapshot.zh-CN.md)、[测试矩阵](../../status/test-matrix.zh-CN.md)、[阶段门槛](../../status/stage-gates.zh-CN.md)、[ADR-0001](../../adr/0001-open-source-boundary.zh-CN.md)

> 后续决策（2026-11-04，以 Phase B 设计文档为准）：crate 统一命名为 `poie-governance`；模型工具目录 v1 为六项，不向模型暴露 `dispatch.request`（宿主 B2 的待审批入口另行讨论）；审批词表统一为效果类 `read_only / plan / simulate / dispatch` + 审批类 `auto / operator_confirm / two_person / forbidden`；不使用旧的混合命名。


## 0. 总体结论

四份设计草案的方向与现有 PO/SAP/证据体系基本一致，但仓库现状仍是**“契约 + 库 + 只读桌面快照”**，不是产品层。A 的只读对象/动作目录是低风险起点；B 的治理边界是最高风险项；C 有 `poie-sim`/`Namespace`/`Patch` 可复用，但缺快照与提升路径；D 已有打包 CI，缺通道、回滚、离线交付与版本/证据绑定。

当前最需要先补的是四个“可验证底座”：**类型化对象投影**、**宿主侧工具/审批策略**、**可序列化模拟快照**、**release manifest**；否则 A–D 会各自复制字段与权限逻辑，违背 SAP 唯一执行语义与 ADR-0001 I2。

建议顺序：`A1 只读对象/链接 → A2 动作目录/SDK → B1 工具 scope/引用/审计 → C1 快照/扰动/比较 → B2 审批 → C2 提升 → D1 manifest/离线包 → D2 通道/运维`。D 的 manifest 字段设计应在 A2 后立即冻结，避免 C 的 digest 与 D 的版本字段返工。

## 1. 现有可复用基线

| 层 | 实际存在 | 关键路径 | 审计判断 |
| --- | --- | --- | --- |
| 本体 | 资产、资源、批次、工艺、服务、能力、验证等级、质量类、`Patch` | `crates/poie-ontology/src/{model,graph,scenario}.rs`；`scenarios/base-01/model.toml` | 对象类型唯一来源 |
| 状态/动作/执行 | 九类记录、生命周期、七项复核、租约、预留、仿真隔离、日志；执行门/接管/降级/独立验证 | `crates/poie-state/src/*`、`crates/poie-exec/src/{gate,verify,writable}.rs` | SAP 唯一权威；B 的 dispatch 唯一落点 |
| 规划/可行性/联邦 | 计划、重规划、SAP 校验；VEC 指标/报告；HLC/站点交换/冲突对账 | `crates/poie-plan/src/*`、`crates/poie-viability/src/*`、`crates/poie-federation/src/*` | C 可复用，但不可绕过 live 隔离 |
| 仿真 | 离散事件、电池、时间场景、守恒、digest | `crates/poie-sim/src/{simulation,timed,battery}.rs` | 有运行态，无 checkpoint/恢复 API |
| 桌面宿主 | Tauri 2、SQLite、认证、推理、沙箱、19 个命令 | `apps/desktop/src-tauri/src/{lib,auth,db,inference,secrets,http,execution,domain,federation_view,sandbox}.rs` | 产品 API 落点，但快照不可变 |
| 桌面 UI | 六视图、i18n、主题、对象/计划/执行/联邦/证据只读面板 | `apps/desktop/src/{App.tsx,domain.ts,i18n.ts,types.ts}` | 无对象写入、动作表单、what-if |
| CI/证据 | 根 CI 全门禁、桌面独立 workflow、实验重放、计数检查 | `.github/workflows/{ci,desktop-build,scheduled}.yml`、`scripts/*`、`experiments/*/PREREG.md` | 新 crate/实验沿用同一纪律 |

## 2. Phase A：Ontology 产品层

### A.1 现有资产与精确路径

- `crates/poie-ontology/src/model.rs`：`ModelDoc`/`Model`、`Asset`/`Resource`/`Lot`/`Process`/`Service`、`Verification`、`LotQuality`、`Patch`；字段与链接语义已齐全。
- `crates/poie-ontology/src/graph.rs`：`Goal`、`Query`、`Verdict`、`Step`、`Blocker`、`CandidateRoute`、`ExternalDependencyReport`；可作为“函数目录”的只读实现。
- `crates/poie-state/src/action.rs`：`Action`、`ActionState`、`Authorization`、`VerificationSpec`、`Reversibility`；`crates/poie-plan/src/plan.rs` 的 `PlannedAction` 已含 `preconditions`/`allocations`/`occupies`/`verification`，可作为动作预览后端。
- `apps/desktop/src-tauri/src/domain.rs`、`apps/desktop/src/domain.ts`、`App.tsx`：当前嵌入 Base-01，生成/手写同形 `DomainOverview`；`Capabilities`/`Plan`/`Inspector` 是现有 UI 入口。
- `apps/desktop/src-tauri/src/execution.rs:81-99`：`build_action` 把 `policy_version`、`state_version`、前置谓词、`authorization.by="maintainer"`、`basis="issue #105"` 全部硬编码；这是 A 必须替换的“演示性授权”。

### A.2 缺失的概念/API/UI/数据模型

1. **稳定对象契约**：无跨类别 `ObjectType`/`ObjectInstance`/`LinkType`/`LinkInstance`；对象只有 `EntityView{id,kind,status}`，资源/工艺没有属性字典。
2. **链接目录**：`requires`/`requires_any`/`provides`/`inputs`/`outputs`/`occupies`/`restores`/`requires_capabilities` 分散在模型字段，无方向、端点类型、稳定 ID、排序规则。
3. **动作目录**：`Process` 到 SAP `Action` 之间无可枚举 `ActionType`；缺参数 schema、目标类型、单位、预算模板、审批类、`allowed_in_modes`、`executor_kind`、`evidence_required`。
4. **函数目录**：`graph::solve`、`external_dependency_report`、`plan` 都是 Rust 函数，无名称、输入/输出 schema、副作用声明；无 `FunctionType`。
5. **SDK/生成/UI**：无 JSON Schema/TS 生成器；前端类型靠 `domain.ts` 手写；无对象浏览器分页/类型筛选/链接分组/属性来源；无动作表单与错误码。
6. **证据/写入**：`Asset.status_evidence`、`Lot.evidence` 是字符串，未统一为可解析 `EvidenceRef`；属性无统一 `as_of`/来源/验证等级；无对象/动作 host 命令，`domain_overview` 在 startup 后不可变。

### A.3 建议模块边界与依赖方向

```text
crates/poie-ontology ──> crates/poie-objects ──> apps/desktop/src-tauri (commands)
        │                      │
        └──> crates/poie-state ┘
crates/poie-sdkgen ──> poie-objects（只生成 schema/TS，不运行时依赖）
```

- 新增 `crates/poie-objects`：`OntologyCatalog`、`ObjectInstance`、`LinkInstance`、`ActionType`、`FunctionType`、`ObjectPage`、`ActionPreview`、`EvidenceRef`；纯 Rust、无 Tauri、无 SQL、无网络；camelCase、稳定排序与 digest。
- 新增 `crates/poie-sdkgen`：生成 `apps/desktop/src/sdk/ontology.ts`/`client.ts`，生成物提交且 CI 重生成零 diff；设计文档的 `scripts/generate-ontology-sdk.py` 也可，但生成器必须有 workspace 测试。
- 桌面新增 `src-tauri/src/objects.rs` 与 `ontology_catalog/objects/object/action_preview` 命令；前端新增 `src/sdk/` 与对象浏览器，逐步替换 `domain.ts` 手写类型。
- 依赖方向：`poie-objects -> poie-ontology, poie-state`；`poie-ontology` 不得反向依赖产品层；`poie-plan` 保持独立，动作目录不复制规划算法；新 crate 进根 `members`，桌面继续 `exclude`。

### A.4 与正式契约/SAP/证据的冲突风险

- **声明状态被误当观测**：PO §3.1 规定无 `status_evidence` 的状态是合成观测；对象层若把 `status` 显示为“事实”，违反证据语义。每个属性必须带 `source`、`as_of`、`verification`，并显式标注 declared。
- **动作目录变成第二套执行语义**：`ActionType` 只能生成 proposed 草稿；不得直接构造 `validated/authorized`，不得复制七项复核。预览必须调用 `validate_plan_through_sap` 或至少 `review_dispatch`。
- **硬编码授权污染目录**：`execution.rs` 的 `Authorization{by:"maintainer", basis:"issue #105"}` 不能进入产品目录；必须由操作员会话、`policy_version`、`approval_digest` 生成。
- **单位/版本/质量漂移**：参数只能经 `poie-ontology::units` 解析，不能把字符串塞进 `Action.command`；`LotQuality` 与资源类型一致性必须保留；Catalog schema 独立于 `poie.physical-ontology/0.3`，未知版本拒绝。

### A.5 最小可行切片与验收

MVP：Base-01 只读对象图 + 全部 `process.*` 动作目录 + 生成 TS 类型 + Capabilities 对象详情 + 动作预览（只显示，不提交）。

1. `cargo test -p poie-objects` 通过；catalog digest 两次运行一致；对象/链接计数与模型字段逐项对应。
2. 每个 `Model.processes` 都有 `ActionType`；参数单位、前置条件、验证等级来自模型，不含硬编码。
3. 未知 type/object/parameter/schema 返回稳定错误码；`ActionPreview.can_submit=false` 或只允许进入 proposed；`pnpm build`/Playwright 通过，六个旧视图不回归，docs/矩阵检查同步计数。

## 3. Phase B：Agent 治理与评测

### B.1 现有资产与精确路径

- `apps/desktop/src-tauri/src/auth.rs`：单本地操作员、PBKDF2 210k、8h 会话、`auth::require`；无角色/多用户。
- `apps/desktop/src-tauri/src/db.rs`：`operators`/`audit`/`inference_config` 三表；`audit` 只有 `id,at_ms,actor,event,detail`，无 `turn_id`/`hash`/`sap_record_id`。
- `apps/desktop/src-tauri/src/inference.rs`：仅 OpenAI-compatible `/chat/completions`、单 prompt、`max_tokens=256`、无 tools/tool_calls；审计只记模型/端点/字符数。provider 传输复用 `{secrets,http}.rs` 的平台凭据库 + 原生 `ureq`/rustls。
- `apps/desktop/src-tauri/src/lib.rs`：`inference_complete`/`inference_test` 是同一 `run_inference`；`execution_gate_set` 未登录仍改 gate（`auth::require` 只用于审计），这是 B 的先修缺口。
- `crates/poie-state/src/review.rs` + `crates/poie-exec/src/gate.rs` + `crates/poie-plan/src/sap.rs`：审批/派发唯一合法落点。
- `experiments/*/PREREG.md` + `raw/` + `scripts/check-experiment-replays.py`：可复用的“预登记—运行—重放”纪律；`inference.rs` 测试的 `MockTransport` 可扩展为脚本化 provider。
- `apps/desktop/src/App.tsx` 的 `agent-panel`/`SettingsDialog`：现有 UI 入口；`i18n.ts` 已有 agent/settings 文案。

### B.2 缺失的概念/API/数据模型

1. **Tool Catalog**：无工具名、版本、输入 JSON Schema、输出类型、scope、审批类、调用上限；模型不能读取对象，也不能产生结构化计划。
2. **权限/scope**：无 `read/propose/simulate/dispatch` 宿主级检查；Tauri 自定义命令没有按窗口/会话的 scope，`capabilities/default.json` 只有 `core:default`，任何 webview 都能调用全部命令。
3. **引用/证据**：`InferenceResult` 只有 `text/model/endpoint/latency`；无 `Citation{kind,id,digest}`、无 `as_of`、无 citation 校验、无“工具输出不可信”标记。
4. **审批/运行模型**：无 `auto_read/operator_confirm/two_person/forbidden`；无审批摘要绑定 run/call/参数/会话/过期；无 `AgentRun`/`Turn`/`ToolCall`/预算/取消/超时。
5. **审计**：SQLite 表可变、无哈希链、无 SAP 记录引用；无法重建“谁批了什么、依据哪版策略”。
6. **评测/策略**：无 agent fixture/脚本化 provider/指标（工具序列、引用覆盖、越权拦截、注入 canary、pass^k）；CI 无确定性 eval；无站点级 scope/remote inference/auto-update 开关（D 补，B 预留读取）。

### B.3 建议模块边界与依赖方向

```text
crates/poie-agent（纯 Rust，工具/策略/引用/运行）
  ├── poie-objects（A）
  ├── poie-state（review/action/record）
  ├── poie-plan（propose/preview）
  └── poie-exec（只读 Gate 语义，不调用 dispatch）
apps/desktop/src-tauri/agent.rs（provider HTTP、密钥、审批 UI 回调、审计）
experiments/agent-evals/（fixtures、PREREG、raw、REPORT）
```

- 设计文档用 `poie-agent`，研究备忘用 `poie-governance`/`poie-eval`；实现前必须统一命名，否则 CI 计数、依赖和文档会分裂。
- 核心策略 crate 不依赖 Tauri/网络；`ModelProvider`/`ToolExecutor` trait；生产 provider 复用 `http.rs`/`secrets.rs`，测试用 `ScriptedProvider`。
- 工具白名单封闭：`objects.get/search`、`capability.query`、`dependencies.trace`、`plan.propose`、`sap.preview`、`evidence.get`；**不存在** `dispatch.*`/`shell.*`/`http.*`/`secrets.*`。研究备忘的 `dispatch.request` 与 B 设计“无 dispatch 工具”冲突，建议 v1 不提供，或仅作宿主内部审批入口且不暴露给模型。
- 审批类统一为 `auto_read/operator_confirm/two_person/forbidden`；`operator_confirm` 绑定 `run_id + tool_call_id + args_sha256 + approval_digest + session + expires_at`，一次一用。
- 审计新增 `turn_id/tool_id/args_sha256/result_sha256/policy_version/sap_record_id/prev_hash/hash`；SQLite 只做索引，权威事件进 SAP 记录；`db.rs` 引入 `PRAGMA user_version`，不能继续 startup `DROP TABLE`。
- 评测沿用仓库纪律：`experiments/agent-evals/PREREG.md` 先提交；fixture 记录模型/端点/prompt 摘要/温度/工具序列；CI 只跑 `ScriptedProvider`；报告 `pass^k`、引用正确率、越权拦截率、注入 canary 实测值，不写“已防御”。

### B.4 与正式契约/SAP/证据的冲突风险

- **模型输出不是观测**：SAP §3.2 补充规则 3 要求 `self_reported`；Agent 文本不得写 `verification`，不得把“我已完成”当证据。引用只证明可追溯，不证明正确。
- **Prompt injection**：对象名称、证据文本、工具输出都是不可信数据；工具输出必须 `untrusted=true`，不得进入指令位。研究备忘的 Rule of Two 与“无网络工具”应写成不变量，而不是提示词。
- **审批绕过**：`execution_gate_set` 目前未登录也可改 gate；B 的审批 UI 之前必须改为硬性 `auth::require` + scope 检查，否则“操作员确认”是装饰。
- **Tauri 命令面过大**：所有窗口共享 host；`open_workspace_window` 只校验 view 名。若 Agent 能调用 `inference_configure`/`sandbox_write`/`auth_*`，就是越权；必须按命令做 scope 检查，必要时拆分窗口/命令包装。
- **审计不是证据链**：SQLite `audit` 可变且无链；不能宣称防篡改。若 D 引用它，必须补哈希链/SAP 引用，否则与 ADR-0003 的只追加语义冲突。
- **外部端点**：推理端点是唯一出站；`validate_endpoint` 已限制 HTTPS/localhost，但模型返回的 Markdown 链接/图片可能成为外泄通道，UI 应按纯文本渲染，禁止自动抓取。

### B.5 最小可行切片与验收

MVP：只读三工具（object/plan/evidence）+ 强制 citation + `auto_read`/`operator_confirm` 两类审批 + 结构化审计 + 一个离线 eval fixture；不接真实 provider 到 CI，不开放 dispatch。

1. 未登录/无 scope/窗口不匹配时，所有 agent 命令拒绝；`execution_gate_set` 未登录也必须拒绝。
2. 工具参数经 JSON Schema 校验；未知工具/参数/超预算拒绝且错误码稳定。
3. 事实性断言无 citation 或引用不存在时拒绝；citation 能解析到 `RecordId`/`StateId`/`Plan::digest()`/实验产物 digest。
4. `operator_confirm` 暂停→批准/拒绝；修改 target/command/valid_until/policy_version 任一字段后审批失效。
5. 审计覆盖 run/tool/approval/denial，不记录密钥/完整 prompt，可用 SAP 记录重建关键事件；CI 跑 `cargo test -p poie-agent` + 离线 eval + `check-experiment-replays.py`，注入 fixture 不产生额外调用/派发，UI 显示工具轨迹/引用/审批/拒绝原因。

## 4. Phase C：What-if / 场景工作区

### C.1 现有资产与精确路径

- `crates/poie-sim/src/simulation.rs`：`Simulation<'m>`、`Event`、`StateRule`、`Battery`、`MassAudit`、`digest()`；`run_until`/`run_to_completion`；无 checkpoint/序列化。
- `crates/poie-sim/src/timed.rs`：`TimedScenario`、`TimedRunner`、`TimedOutcome`、`TimedVerdict`（含 `too_late`/`insufficient_resource`/`capacity_conflict`/`power_limited`）；`storage_discharge_w` 在场景上，不在 `Patch` 上。
- `crates/poie-ontology/src/model.rs::Patch`：支持 asset status、lot quantity/quality/resource、process verification、requires/requires_any、degraded、capability verification、add_asset/resource/lot/process；**不含** storage power、时间线扰动、goal/deadline。
- `crates/poie-plan/src/replan.rs`：`ReplanResult{plan,kept,cancelled,added}`，可作为“修改后重规划”的基础；`crates/poie-viability/src/{metrics,report}.rs` 提供指标/截尾/报告约束。
- `crates/poie-state/src/isolation.rs::ensure_live_dispatch` + `record.rs::Namespace::Simulation`：仿真隔离的既有闸门；C 必须使用，而不是新造标志位。
- `crates/poie-federation/src/federation.rs`：站点交换/冲突；多站点 what-if 只读使用，不能把变体写进 live 站点。
- `apps/desktop/src/App.tsx`：`Mode` 的 `sim/shadow/live` 目前 `MODE_ENABLED=false`；`Topbar` 只改 UI 标签，不是安全边界。

### C.2 缺失的概念/API/数据模型

1. **Snapshot**：无不可变 `ScenarioSnapshot`（模型 digest、状态标识、证据版本、目标、时间线、创建者）；`domain_overview` 只返回展示字段，没有 `StateId`/lot/存储限制。
2. **Variant**：无 `Variant`/`PatchEnvelope`；`Patch` 不能表达 timed 扰动、storage 功率、目标/截止时间；无变体数量/字段上限。
3. **运行/比较**：无 `variant_run`、无 baseline vs variant 统一比较（verdict、完成时间、外部依赖、根因、SAP 预览）；`TimedVerdict` 与结构 `Verdict` 词表不同，C 设计把 `candidate` 与 `too_late` 混在一个枚举，需先归一化。
4. **提升/持久化**：无 `promote`；没有“stale snapshot 检查 + 重新规划 + SAP 校验 + proposed 动作”的路径；无 snapshot/variant/run 表、迁移版本、外键/清理策略。
5. **UI**：无工作区、变体编辑器、并列比较、`SIMULATION` 状态；`Sim` 模式禁用；`GRAPH_NODES` 仍是手写常量。
6. **联邦/证据**：无“站点 B 的库存/租约是否受影响”的 what-if 视图；变体结果没有引用 snapshot/model digest、证据版本、seed/now；`Plan::digest()`/`Simulation::digest()` 未进入结果契约。

### C.3 建议模块边界与依赖方向

```text
crates/poie-whatif
  ├── poie-ontology（Model/Patch/graph/schedule）
  ├── poie-plan（plan/replan/sap preview）
  ├── poie-sim（timed/checkpoint）
  ├── poie-state（Namespace/isolation/StateId）
  ├── poie-viability（可选指标）
  └── poie-federation（可选只读站点快照）
apps/desktop/src-tauri/whatif.rs（DB/auth/commands/audit）
apps/desktop/src/views/whatif/*（编辑器/比较/提升）
```

- `poie-whatif` 纯 Rust、无 Tauri；所有变体用独立 `Model` 副本与独立 `Plan`/ledger；不写 live `ActionLedger`/`execution_snapshot`/`federation`。
- 在 `poie-sim` 增加 `Checkpoint {time_s, assets, stocks, batteries, external_flows, digest}` 与 `Simulation::checkpoint/from_checkpoint`；这是 C 的硬前提，否则只能重跑不能“快照”。
- `Patch` 保持模型层语义；timed/storage 扰动放入 `whatif::Perturbation`，由它构造 `TimedScenario`/`Simulation`，不要污染 `Patch` 契约。
- 提升路径：`VariantResult -> PlanningRequest -> Plan -> validate_plan_through_sap -> proposed actions`；不复制变体中的 action 实例，不直接写 live；stale 检查比较 model/evidence/StateId，失败返回 `stale_snapshot`。
- 依赖 DAG 保持 `ontology ← sim/plan/viability ← whatif ← desktop`，无环；`poie-objects` 不依赖 sim/plan。

### C.4 与正式契约/SAP/证据的冲突风险

- **命名空间泄漏**：所有变体记录必须是 `Namespace::Simulation(branch_id)`；任何 `ensure_live_dispatch` 通过都意味着 C 失败。UI 的 `Sim` 标签不是隔离，host 必须强制。
- **提升绕过 SAP**：C 设计允许“复用结果”提升；但 live 状态/证据可能已变，必须重新规划并过七项复核。复用只允许复用“输入与 digest”，不能复用授权。
- **digest 不完整**：变体 digest 必须覆盖 model digest、canonical patch、snapshot/now、seed、plan/sim 设置；否则不可复现，违反 VEC §7 与实验纪律。
- **结论词表混用**：结构 `Verdict` 与 `TimedVerdict` 不是同一枚举；C 的单一 `verdict` 字段必须先定义“结构结论 + 时序结论”的归一化，否则 UI 会把 `feasible` 当 `candidate`。
- **守恒/负库存**：`Simulation` 已检查守恒与负库存；C 不能只展示 `completion_s` 而隐藏 `violations`/`MassAudit`；否则重演 H4 的核算错误。
- **联邦冲突**：多站点 what-if 的库存/租约冲突必须显式报告，不能 last-write-wins；C 若只做单站点，UI 要标注范围。

### C.5 最小可行切片与验收

MVP：Base-01 单 snapshot；3 个变体（泵延迟、备用泵失败、储能功率限制）；结构/计划/可选 timed 运行；baseline vs variant 比较；提升为 proposed plan；UI `SIMULATION` 标记。

1. `cargo test -p poie-whatif` 通过；同 snapshot+patch 两次运行 digest 相同；未知 ID/单位/负值/超上限拒绝。
2. 运行前后 live `DomainOverview`、`ActionLedger`、`federation_snapshot` 完全不变（测试断言）。
3. 变体记录全部为 `Namespace::Simulation`；无 `live` 记录；`power_limited` 与 `requires_any` 变体各至少 1 个。
4. stale snapshot 提升拒绝；提升只产生 proposed，SAP 校验失败或审批缺失拒绝；UI 可创建/运行/比较至少 3 个变体，排序确定，状态栏显示 `SIMULATION`，审计记录 snapshot/variant/run/promote。

## 5. Phase D：站点交付

### D.1 现有资产与精确路径

- `.github/workflows/desktop-build.yml`：Linux `.deb`、Windows NSIS、macOS universal `.app` + zip；tag `v*` 触发 `release` job，上传 `.deb/.exe/.zip` 与 `SHA256SUMS`。
- `.github/workflows/ci.yml`：根 workspace fmt/clippy/test/rustdoc/链接/矩阵/重放/Python/场景/实验/cargo-deny；不构建桌面。`.github/workflows/scheduled.yml` 每周 verify-all/release tests/手动 fresh-clone/cargo-deny。
- `apps/desktop/src-tauri/tauri.conf.json`：bundle targets `all`，但 workflow 只传 `--bundles deb|nsis|app`；无 updater 插件、无 Windows WebView2 离线安装模式。
- `apps/desktop/src-tauri/Cargo.toml:3`、`apps/desktop/package.json`：版本 `0.1.1`；根 crates 全部 `0.1.0`；tag `v0.1.1`。
- `apps/desktop/src-tauri/src/db.rs:58-86`：startup `DROP TABLE IF EXISTS events/notes`，无 `user_version`/迁移框架。
- `scripts/verify-all.sh`、`docs/MAINTENANCE.zh-CN.md` §8/§10：发布前清单、桌面本地验证；未覆盖桌面 release、版本 bump、回滚、离线包。
- `scenarios/base-01-timed/digests.toml`、`experiments/*/REPORT*`、`docs/status/test-matrix.zh-CN.md`：现有可绑定的证据资产。

### D.2 缺失的概念/API/数据模型

1. **Release channel**：只有 GitHub latest release，无 `stable/canary/site`、无 prerelease 规则、无升级边界。
2. **Site bundle**：无 `SiteManifest`/`BundleFile`/路径校验/摘要/原子导入；无模型、合同、证据、策略、文件清单绑定。
3. **离线交付**：无 air-gapped bundle；Windows NSIS 默认 WebView2 获取方式未声明为离线；无离线安装/验证脚本。
4. **诊断/可观测性**：无 diagnostics 包、无结构化 host 日志、无健康检查/指标导出；只有 SQLite 审计。
5. **回滚**：无上一版本保留、无 bundle rollback、无 DB 迁移/备份；安装器降级路径未定义。
6. **版本/证据绑定**：无 `RELEASE.json`/manifest；`host_info` 只返回 desktop `CARGO_PKG_VERSION`，不含 git commit、target、contract/scenario/evidence digest。
7. **供应链/门禁**：无 SBOM、无 artifact attestation/provenance、无签名强制；`SHA256SUMS` 只覆盖桌面产物，不覆盖 manifest 自身。
8. **发布门禁**：tag release 只依赖 desktop build，不要求根 `verify-all`/证据重放；`desktop-build.yml` 的 `pull_request` paths 未包含 `apps/desktop/tests/**` 与 `playwright.config.ts`，UI 测试改动的 PR 不会触发该 workflow。

### D.3 建议模块边界与依赖方向

```text
crates/poie-delivery（manifest/bundle/path validation/diagnostics schema；纯 Rust）
  ├── poie-ontology（model digest）
  ├── poie-state（state/plan digest）
  └── poie-viability（可选 evidence metadata）
scripts/release-manifest.py（生成 RELEASE.json + SHA256SUMS + 校验）
apps/desktop/src-tauri/delivery.rs（sandbox 文件 IO、SQLite DeliveryState、命令）
.github/workflows/desktop-build.yml（构建 + channel + manifest + release）
```

- `poie-delivery` 不依赖 Tauri/网络；host 执行文件复制；bundle 路径校验复用 `sandbox.rs` 的相对路径规则。
- manifest 至少包含：`schema`、`release_id`、`git_commit`、`desktop_version`、`core_crate_versions`、`contract_versions`（PO/SAP/VEC 的 version+revision）、`scenario_digests`（含 `base-01-timed/digests.toml`）、`evidence_bundle_digest`、`channel`、`rollback_to`、`artifacts[{name,os,target,sha256,bytes}]`、`integrity.manifest_sha256`。
- `host_info` 增加 `releaseId`/`channel`/`manifestDigest`；desktop 通过构建期 `build.rs` 或嵌入 `release.json` 获取，不访问网络。
- 根 CI 增加 `scripts/check-release-manifest.py`（校验版本/提交/文件摘要/合同版本/场景 digest）；桌面 workflow 在 tag 上生成并上传 `RELEASE.json`/SBOM；release job 依赖根验证结果（`workflow_run` 或 release job 前置 step）。
- 桌面继续在根 workspace 之外；`poie-delivery` 进根 workspace。不要为了 release 把 desktop 加进根 workspace（Linux WebKit 依赖会破坏现有 CI）。

### D.4 与正式契约/SAP/证据的冲突风险

- **版本单一真相**：core `0.1.0`、desktop `0.1.1`、tag `v0.1.1` 三者不一致；manifest 必须明确“desktop release 版本”和“core API 版本”两个字段，不能用一个版本号覆盖。
- **过度声明**：root README 声称 macOS DMG、Windows portable exe，但 workflow 只构建 `.app` zip/NSIS；release 资产不含 DMG/portable。D 的 UI/文档必须按实际产物命名。
- **证据绑定不能伪造**：manifest 只能引用已存在的 scenario digest/evidence report；不得把“未运行”写成“已验证”。未签名/未公证要显式标记 `unverified`，不能宣称可信。
- **回滚与只追加审计**：回滚安装包不能回滚 SAP 只追加日志；DB 迁移必须前向兼容/备份，回滚后要能读旧 schema 或拒绝启动并提示；不能 `DROP TABLE` 掩盖迁移。
- **离线安全**：bundle 只做 SHA-256 完整性时，必须明确不是签名；导入路径要拒绝绝对路径/`..`/符号链接；否则站点可被替换模型/证据。
- **ADR-0001 I3**：开源版必须能独立跑公开场景；离线 bundle 不能依赖闭源组件或云服务；远程推理默认关闭。

### D.5 最小可行切片与验收

MVP：`RELEASE.json` v0.1 + `scripts/check-release-manifest.py` + SHA256SUMS + 一个 Linux/macOS 离线 bundle + `host_info` 展示 release/channel + 设置页“交付”只读摘要；不做自动更新。

1. manifest 对 tag commit、desktop 版本、core 版本、PO/SAP/VEC revision、scenario digest 全部可校验；任一不符 CI 失败。
2. `SHA256SUMS` 覆盖所有发布资产与 manifest；篡改一个文件导致导入/校验失败。
3. 断网机器导入 bundle 后能运行 Base-01/Base-01 timed（无推理、无网络）；`host_info` 显示正确 release/channel。
4. 诊断包不包含 API key、prompt、绝对敏感路径；导出前白名单扫描。
5. 回滚到上一 bundle 后版本/证据检查通过，审计保留；DB 迁移有 `user_version` 与失败回退；tag release 依赖根验证，PR 修改桌面测试/Playwright 配置会触发 desktop workflow。

## 6. 跨阶段排序与依赖约束

| 顺序 | 工作 | 必须先完成 | 原因 |
| --- | --- | --- | --- |
| 1 | A1 只读对象/链接 | 无 | 所有阶段的 ID/语义来源 |
| 2 | A2 动作目录/SDK | A1 | B 的工具 schema、C 的变体动作字段依赖它 |
| 3 | B1 工具 scope/引用/审计 | A2 | 没有 catalog 就无法定义工具输入/输出 |
| 4 | C1 snapshot/扰动/比较 | A1/A2 | 需要对象 ID、`Patch`、sim/plan API；可并行 B1 |
| 5 | B2 审批/运行生命周期 | B1 | 审批必须绑定 tool/args/run |
| 6 | C2 promote | C1 + B2 | 提升需要审批、引用、SAP 重新校验 |
| 7 | D1 manifest/离线包 | A2 + C1 | 需要 catalog/schema、scenario digest、plan/sim digest |
| 8 | D2 channel/observability/rollback | D1 + A–C 稳定 | 通道与回滚需要可绑定版本/证据 |

硬约束：

- **C 的 promote 不得早于 B2**；否则“操作员确认”没有实现，违反 SAP 授权语义。
- **B v1 不提供 dispatch**；未来 dispatch 必须等 M10 物理/执行门验收，`poie-exec::ExecutionGate` 仍是唯一派发点。
- **D 的 channel 不得早于 A2/C1 的 digest 契约**；否则 manifest 字段反复改版。
- **任何阶段都不得先改 `poie-ontology` 的语义再补用例**；PO/SAP 变更必须先升版本、写修订记录、加 SAP-T 用例。
- 设计 README 的“严格 A→B→C→D”可作为发布顺序；工程上允许 B1/C1 并行，D1 可在 A2 后启动，但不得跳过上述安全前置。

## 7. 陈旧与不一致清单

| 位置 | 现象 | 影响/建议 |
| --- | --- | --- |
| `apps/desktop/src/App.tsx:944` | 写“384 listed Rust tests”；实际 `cargo test -- --list` 为 388，README、`apps/desktop/README.md`、`test-matrix` 均为 388 | 改为 388；把该文案纳入计数检查器，或从 UI 删除硬编码 |
| `docs/status/stage-gates.zh-CN.md:20-21,36,38,48-51` | ontology “57 单元+16 集成”、state “53+13”、runtime 12、exec 12、plan 18、viability 33、federation 7；实际分别为 96、97、13、19、22、61、13 | 日期 2026-10-31 已落后；按 `cargo test --list` 重写，并保留“SAP-T1–T15”而非 T1–T11 |
| `README.md:27,82` | 声称 macOS DMG、Windows “portable executable”；`desktop-build.yml` 只构建 `.app` zip 与 NSIS，release 不产出 DMG/portable | 站点交付前必须改文案或改构建；当前属过度声明 |
| `SECURITY.md` + 版本 | `SECURITY.md` 写“no stable release”，README 已有 v0.1.1；core `0.1.0`、desktop `0.1.1`、tag `v0.1.1` 无单一版本源 | 明确 research preview 定义；Phase D 用 manifest 区分 desktop release 与 core API 版本 |
| `docs/status/status-snapshot.zh-CN.md:1,40,46` | 日期 2026-11-01；“#112 三项推迟”；干净克隆 `d0d449c`；而 HEAD `bee82b7`，2026-11-03 已合并 #112 P18–P20 | 更新日期、#112 状态、验证提交；否则外部评审会读到旧结论 |
| `apps/desktop/src/domain.ts:219-224` | Base-02、Base-01 timed、expansion 标为 `planned`，实际 Base-02/timed 已在 CI 跑 | 改为 active/replay 状态，或从 UI 移除；避免用户误判 |
| `apps/desktop/src/App.tsx:940` | PO 写“2026-11-01 revision (#105/#106/#111)”，PO 实际已含 2026-11-03 #112/P18–P20 | 更新文案；最好从 manifest/契约元数据读取 |
| `crates/poie-state/src/lib.rs:31` | 仍写“Not implemented yet: site-to-site log exchange ... M12”，但 `poie-federation` 已实现库级 M12 | 改为“本 crate 不实现；见 `poie-federation`” |
| `.github/workflows/desktop-build.yml:20-31` | PR paths 缺 `apps/desktop/tests/**`、`playwright.config.ts`；push paths 有重复条目 | UI 测试改动不会触发桌面 workflow；去重并补齐 |
| `docs/design` 内部 | B 设计用 `poie-agent` 且“无 dispatch 工具”；研究备忘用 `poie-governance`/`poie-eval` 并提 `dispatch.request`；审批词表也不同 | 实现前统一 crate/工具/审批词表；否则 B 的代码与文档会分裂 |
| `docs/design/phase-c-what-if.zh-CN.md` §5.2/§5.3 | 设计把 `storageDischargeW` 放进 Variant.patch，但 `poie-ontology::Patch` 无此字段；`VariantResult.verdict` 混用结构 `candidate` 与时序 `too_late` 等 | C 实现前先定义 `Perturbation` 与 `Verdict` 归一化；否则编译/语义都对不上 |
| `apps/desktop/src-tauri/src/lib.rs:185-207` | `execution_gate_set` 未登录也改 gate，`auth::require` 只用于可选审计 | B/C 的审批前提；必须先改为硬性拒绝 |
| `apps/desktop/src-tauri/src/db.rs:58-86` | 每次启动 `DROP TABLE IF EXISTS events/notes`，无 `user_version` | D 的迁移/回滚无法安全实现；先建迁移框架 |

## 8. 对根 workspace/CI 的具体约束

1. 新领域 crate 必须进根 `Cargo.toml` `members`（`poie-objects`、`poie-agent`、`poie-whatif`、`poie-delivery`）；`apps/desktop/src-tauri` 继续在 `exclude`。每个新 crate 通过 fmt/clippy/test/rustdoc，新增测试后同步 `test-matrix`/`status-snapshot`，否则 `check-test-matrix.py` 失败。
2. 新增实验按 `experiments/<id>/PREREG.md` 先登记，产物确定性、CI diff 重放；SDK 生成物提交并有重生成零 diff 检查；Markdown 链接必须可解析（`check-docs.py`）。
3. 桌面命令扩展同时更新 `lib.rs` `generate_handler!`、`apps/desktop/README.md` 命令表、Playwright smoke；host 测试保持 `cargo test --offline`；新依赖过 `cargo-deny`；授权/派发/证据/隔离代码按 ADR-0001 I2 留在开源核心，桌面不得私加“影子检查”。

> 本审计只新增 `docs/design/research/repo-gap-audit.zh-CN.md`；未修改应用源码、CI 或既有设计文档。
