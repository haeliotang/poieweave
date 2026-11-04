# 跨阶段平台契约：序列化、错误、命令 scope、模型与审批

- 状态：设计草案（与 Phase A–D 同批校验）
- 日期：2026-11-04
- 目的：消除 Phase A–D 各自发明 DTO、错误、scope、digest、审批原语的漂移；所有阶段共享本文件。

## 1. Schema 注册表与版本策略

统一注册表 [`SCHEMAS.md`](SCHEMAS.md)（`scripts/check-schema-registry.py` 已加入 CI；文档表+代码常量必须一致），每个对外 JSON/TOML 结构必须有：

```text
schema = "poie.<domain>/<major>.<minor>"
```

完整 schema 表以 [`SCHEMAS.md`](SCHEMAS.md) 为唯一来源；本文件不复制表，避免两处漂移。

规则：

- 客户端/宿主收到未知 **major** 必须拒绝（`unsupported_schema`）；未知 **minor** 可接受，但忽略未知字段并记录警告。
- 每个持久化结构（`Variant`/`VariantResult`/`DeliveryState`/`AgentRun`）必须带 `schemaVersion`。
- 兼容矩阵与迁移函数写入实现；MVP 不支持跨 major 自动迁移，只报错。
- 表结构由 `PRAGMA user_version` 管理；每次迁移单向、可备份、失败回滚。

## 2. 序列化策略：领域类型 vs 公开 DTO

**决定：公开 API 使用独立 DTO，不直接给正式领域类型加 `Serialize`。**

理由：PO/SAP 领域类型是正式契约，字段名/枚举拼写不能为了 UI 被冻结；DTO 可以 `camelCase`、带 `schemaVersion`，并做显式转换。

- `AssetStatus`、`LotQuality`、`Verification`、`Patch` 保持 Deserialize-only；`StateId` 当前没有 serde，公开 DTO 用字符串表示，不直接序列化领域类型。
- A 定义公开 DTO：`ObjectInstance`、`LinkInstance`、`ActionType`、`SapPreview`、`OntologyCatalog`（这些类型本身就是公开 DTO，不再加 `Dto` 后缀）；从 `Model` 构建。
- C1 定义 `PatchDto`、`SnapshotDto`、`VariantDto`、`VariantResultDto`、`CompareResultDto`；`PatchDto -> Patch` 的唯一转换在 `poie-whatif`，拒绝未知/多余字段。`PerturbationDto` 属 C2。
- D 定义 `SiteManifestDto`、`DeliveryStateDto`、`DiagnosticsDto`。
- DTO 字段稳定、排序稳定；`serde(rename_all="camelCase")`；枚举用 `snake_case` 字符串并显式列出。
- 转换测试：DTO round-trip；领域类型新增字段时测试失败提醒更新 DTO/版本。

## 3. 错误包络

所有 Tauri 命令返回：

```json
{"code":"unknown_object","message":"...","details":{"typeId":"asset","id":"..."}}
```

`Result<T, CommandError>`，不再返回裸 `String`。稳定错误码：

| code | 阶段 | 含义 |
| --- | --- | --- |
| `unauthorized` | A/B/C/D | 未登录/会话过期 |
| `scope_denied` | A/B/C/D | 窗口/命令 scope 不匹配 |
| `unsupported_schema` | A/B/C/D | schema major 未知 |
| `invalid_cursor` / `limit_exceeded` | A/C | 分页参数非法 |
| `unknown_type` / `unknown_object` / `unknown_snapshot` / `unknown_variant` / `unknown_result` | A/C | 类型/对象/快照/变体/结果不存在 |
| `model_mismatch` | A/C/D | model digest 与当前不一致 |
| `preview_unavailable` | A | host 未提供 dry-run |
| `invalid_parameter` / `invalid_patch` | A/B/C | 参数/变体非法 |
| `approval_required` / `approval_mismatch` / `approval_expired` / `approval_rejected` | B/C/D | 审批缺失/被替换/过期/拒绝 |
| `budget_exceeded` | B/C | 步数/工具/时间超限 |
| `stale_snapshot` | C | snapshot/证据已变 |
| `not_promotable` | C | 结果不可提升 |
| `bundle_tampered` / `bundle_incomplete` / `bundle_incompatible` | D | bundle 校验失败 |
| `policy_blocked` | B/C/D | 站点策略拒绝 |
| `citation_missing` / `invalid_tool_arguments` | B | 最终断言引用缺失/工具参数非法 |
| `provider_timeout` / `provider_response_too_large` / `provider_protocol` | B | provider 调用/协议失败 |
| `run_active` / `run_not_found` / `run_interrupted` | B | Agent 运行状态 |
| `audit_integrity_failed` | B | 审计链/锚点损坏 |
| `not_implemented` | B/C | 工具阶段未启用 |

错误不得回显密钥、完整 prompt、绝对路径；`details` 只放稳定 ID/摘要。

## 4. 命令 scope 矩阵

窗口标签（与代码一致）：`main`；聚焦窗口由 `open_workspace_window` 创建为 `poie-view-federation`、`poie-view-execution`、`poie-view-evidence`；`settings`/`agent` 目前不是独立窗口，若新增必须在本矩阵登记标签。

| 命令 | 登录 | 允许窗口 | 读写 | 策略 |
| --- | --- | --- | --- | --- |
| `host_info` | 否 | 全部 | 读 | 无 |
| `domain_overview` / `execution_snapshot` / `federation_snapshot` | 否（只读演示） | main/poie-view-* | 读 | 无 |
| `auth_status/register/sign_in/sign_out` | 否（bootstrap） | 全部 | 写本地 | 仅 auth 命令 |
| `audit_list` | 是 | main | 读 | 会话 |
| `inference_status` | 否 | main | 读 | 配置脱敏 |
| `inference_configure/forget/test/complete` | 是 | main | 写/网络 | 会话 + 站点策略 |
| `execution_gate_set` | 是 | main/poie-view-execution | 写 | 会话 + 审计（A0 修复） |
| `sandbox_read/write` | 是 | main | 读写 | 会话 + 路径/大小/窗口；read/write 均拒绝 symlink；无 `sandbox_list`；A0 删除独立 `sandbox_root` 命令，改用 `host_info.sandboxRoot` |
| `open_workspace_window` | 否 | main | 导航 | 允许 view 白名单 |
| `ontology_*` | 否（只读） | main/poie-view-* | 读 | A0 后按窗口 |
| `agent_*` | 是 | main | 写/网络 | 会话 + scope + policy |
| `whatif_*` | 是 | main/poie-view-plan | 写（C1 结果表） | 会话 + `authorize(window,command,session,policy)`；C1 不写 SAP/live |
| `delivery_*` | 是 | main/settings | D1 元数据写 | 会话 + 审计；D1 无审批/Window；D2 再加审批/policy |

实现要求：

- `auth::Session` 必须带 `sessionId`（128-bit 随机 hex，创建会话时生成）；审批、capability、RunState 全用它；登出/过期即失效。
- 每个命令显式接收/读取 `tauri::Window` 并调用 `authorize(window, command, session, policy)`；不得依赖“所有窗口共享全部命令”。**D1 `delivery_*` 例外**：仅本地 sandbox 元数据，使用 `auth::require` + 审计，不引入 Window/审批/policy；D2 若改远程/策略再适用本行。
- `capabilities/default.json` 仅为默认；为 settings/agent 窗口增加独立 capability 文件（或命令内 scope 检查），二者至少一种，且测试覆盖。
- `execution_gate_set` 改为先 `auth::require` 再修改；`sandbox_*` 改为 no-follow + 原子写（Phase D 复用）。
- 站点策略 `HostPolicy`：默认 `{ agentScopes:["read","propose"], remoteInference:false, autoUpdate:false, channel:"stable" }`；D1 只读取 HostPolicy 展示；D1 不改 policy，D2 才允许 SQLite 覆盖/`policy.check`。

## 5. 当前模型与 digest

桌面唯一“当前模型”：

```text
SourceModel = Base-01 model.toml + pump-p1 = degraded patch
```

- host 在启动时加载并 patch，保存 `Arc<Model>` 与 `model_digest`（`DIGEST.md`），不再只保存 `DomainOverview`。
- `domain_overview`、`ontology_catalog`、`whatif_snapshot`、`execution` 都从同一个 `SourceModel` 构建；切换模型必须同时更新版本并拒绝旧引用。
- D **不导入/切换模型文件**：site bundle 只记录 `modelSha256`，应用仍使用构建期/本地 `SourceModel`；导入只更新交付清单/策略/证据引用。若未来要 model bundle，另立设计。
- 因此 A/C/D 只有一套模型来源，不存在 `ModelStore` 与 `SourceModel` 两个版本。

## 5.1 对象 ID 与 EvidenceRef

统一 canonical object ID：`<type>:<id>`，例如：

```text
asset:pump-p1
resource:electric-energy
lot:lot-sleeve-01
process:replace-sleeve-p1
service:potable-water
```

- type 只允许小写字母；id 只允许 `[a-z0-9._-]`（模型已校验）；
- 不允许在 ID 中编码路径/查询/空白；
- 引用在 API 中作为字符串；本地查找时拆分 type/id 并校验；
- `EvidenceRef { kind, id, fingerprint?, sha256?, asOf?, source }`；`kind` 为 `object|action|plan|experiment|contract|report`；fingerprint 与 sha256 二选一或同时提供，至少一个；`asOf` 为 RFC3339 或 null；
- B 的 citation、C 的 evidenceRefs、D 的 manifest.evidence 使用同一结构。

## 5.2 Action 的仿真来源（SAP §7.4）

从 What-if/Agent 提案产生的 action 必须能追溯到仿真分支/模型：

- 在 `Authorization.basis` 字符串中编码 `simulation=<branchId>;model=<modelSha256>`，或给 SAP `Action` 增加结构化 `simulationBranch`/`modelSha256` 字段；
- MVP 采用 **basis 编码**，不改 SAP 结构；若后续需要结构化，走 SAP v0.3 变更 + ADR + SAP-T 用例；
- 测试：解析 basis 得到 branch/model；缺失时 `not_promotable`。

## 6. 审批原语（B/C/D 共用）

```rust
pub enum ApprovalClass { Auto, OperatorConfirm, TwoPerson, Forbidden }

pub struct ApprovalToken {
    pub id: String,
    pub session_id: String,
    pub run_id: String,
    pub call_id: String,
    pub tool: String,
    pub tool_version: String,
    pub subject_digest: String, // sha256 per DIGEST.md; tool/action specific
    pub policy_version: String,
    pub args_json_redacted: String,
    pub impact_json: String,
    pub citations_json: String,
    pub state: String, // pending|approved|rejected|expired|executing|interrupted|consumed|cancelled
    pub created_at_ms: u64,
    pub decided_at_ms: Option<u64>,
    pub started_at_ms: Option<u64>,
    pub expires_at_ms: u64,
}
```

- `OperatorConfirm`：宿主创建 pending token；UI 展示 subject/args/影响/引用；`agent_approve` 原子置 approved，`agent_resume` 原子置 executing 后执行一次并置 consumed；任何字段变化使 digest 不匹配。
- `TwoPerson`：Phase B/D 不开放（删除预留，避免与单操作员冲突）；需要时另立 ADR + 角色模型。
- `Forbidden`：直接拒绝并审计。
- B 的 agent、C2 的 promote、D2 的 channel/import 使用同一原语；D1 元数据 import 不使用审批。不允许布尔 `approved` 参数。
- 所有 token 持久化（`approval_tokens` 表）以支持暂停/恢复/审计；批准：`pending → approved`；消费：`approved → executing → consumed`（原子条件更新）；过期自动拒绝。
- token 与 SAP 记录的关联：写入审计时记录 `sap_record_id`；`Authorization.basis` 包含 `approval:<tokenId>`（或结构化后字段），使 SAP 记录可反查 token。

## 7. 新 crate/依赖与工作区

新增（按阶段）：

| crate | 阶段 | 依赖 |
| --- | --- | --- |
| `poie-digest` | A | serde, serde_json, sha2 |
| `poie-objects` | A | poie-ontology, poie-digest, serde |
| `poie-sdkgen` | A (bin) | poie-objects, serde_json |
| `poie-governance` | B | poie-objects, poie-state (只读), poie-plan, poie-ontology, poie-digest, serde, serde_json, toml |
| `poie-whatif` | C1 | ontology, plan, objects, digest, serde |
| `poie-delivery` | D1 | digest, serde, serde_json |

规则：

- 全部加入根 `Cargo.toml` `members`；`apps/desktop/src-tauri` 继续 `exclude`，桌面用 path 依赖并更新自己的 `Cargo.lock`。
- 每个 PR：`cargo fmt/clippy/test/doc`、`cargo-deny`、`check-test-matrix`、`check-docs`；新增依赖写 ADR（JSON Schema、serde、sha2、SBOM 工具）。
- 新增运行时依赖默认拒绝；能用手写/已有依赖实现就不引入（如 capability 用 opaque 句柄，不引入 JWT 库）。

## 8. 有限实现顺序（写入各阶段）

```text
A0 auth/scope/digest/DTO -> A1 catalog -> A2 SDK/UI
B1 policy/tool/run state/audit migration -> B2 approvals/evals/UI
C1 snapshot/patch/outcome/compare -> C2 timed/promote/checkpoint
D1 manifest/offline import/state/rollback/diagnostics/UI -> D2 channel/signing/installers/approval/policy
```

每个阶段完成后，更新 `docs/status/test-matrix.zh-CN.md`、`status-snapshot`、README 和本文件的实现状态；未通过校验不得开始下一阶段。
