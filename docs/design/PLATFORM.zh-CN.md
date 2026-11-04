# 跨阶段平台契约：序列化、错误、命令 scope、模型与审批

- 状态：设计草案（与 Phase A–D 同批校验）
- 日期：2026-11-04
- 目的：消除 Phase A–D 各自发明 DTO、错误、scope、digest、审批原语的漂移；所有阶段共享本文件。

## 1. Schema 注册表与版本策略

统一注册表 `docs/design/SCHEMAS.md`（实现时可由 `scripts/check-schema-registry.py` 校验），每个对外 JSON/TOML 结构必须有：

```text
schema = "poie.<domain>/<major>.<minor>"
```

| schema | 所有者 | 兼容规则 |
| --- | --- | --- |
| `poie.ontology-catalog/0.1` | A | 新增可选字段 = minor；字段重命名/语义变更 = major |
| `poie.object-page/0.1` | A | 同上 |
| `poie.action-preview/0.1` | A | 同上 |
| `poie.agent-run/0.1` | B | 运行状态事件；major 变更需迁移 |
| `poie.agent-eval/0.1` | B | fixture |
| `poie.whatif-snapshot/0.1`、`poie.whatif-variant/0.1`、`poie.whatif-result/0.1` | C | 同上 |
| `poie.site-bundle/0.1` | D | 未知 major 拒绝；minor 必须可忽略 |
| `poie.delivery-state/0.1` | D | 同上 |

规则：

- 客户端/宿主收到未知 **major** 必须拒绝（`unsupported_schema`）；未知 **minor** 可接受，但忽略未知字段并记录警告。
- 每个持久化结构（`Variant`/`VariantResult`/`DeliveryState`/`AgentRun`）必须带 `schemaVersion`。
- 兼容矩阵与迁移函数写入实现；MVP 不支持跨 major 自动迁移，只报错。
- 表结构由 `PRAGMA user_version` 管理；每次迁移单向、可备份、失败回滚。

## 2. 序列化策略：领域类型 vs 公开 DTO

**决定：公开 API 使用独立 DTO，不直接给正式领域类型加 `Serialize`。**

理由：PO/SAP 领域类型是正式契约，字段名/枚举拼写不能为了 UI 被冻结；DTO 可以 `camelCase`、带 `schemaVersion`，并做显式转换。

- `AssetStatus`、`LotQuality`、`Verification`、`Patch`、`StateId` 保持 Deserialize-only（现状）。
- A 定义 `ObjectInstanceDto`、`LinkInstanceDto`、`ActionTypeDto`、`SapPreviewDto`；从 `Model` 构建。
- C 定义 `PatchDto`、`PerturbationDto`、`SnapshotDto`、`VariantDto`、`VariantResultDto`；`PatchDto -> Patch` 的唯一转换在 `poie-whatif`，拒绝未知/多余字段。
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
| `unknown_type` / `unknown_object` | A | 类型/对象不存在 |
| `model_mismatch` | A/C/D | model digest 与当前不一致 |
| `preview_unavailable` | A | host 未提供 dry-run |
| `invalid_parameter` / `invalid_patch` | A/B/C | 参数/变体非法 |
| `approval_required` / `approval_mismatch` / `approval_expired` | B/C/D | 审批缺失/被替换/过期 |
| `budget_exceeded` | B/C | 步数/工具/时间超限 |
| `stale_snapshot` | C | snapshot/证据已变 |
| `not_promotable` | C | 结果不可提升 |
| `bundle_tampered` / `bundle_incomplete` / `bundle_incompatible` | D | bundle 校验失败 |
| `policy_blocked` | B/C/D | 站点策略拒绝 |

错误不得回显密钥、完整 prompt、绝对路径；`details` 只放稳定 ID/摘要。

## 4. 命令 scope 矩阵

窗口标签：`main`、`focus-federation`、`focus-execution`、`focus-evidence`、`settings`、`agent`（后续）。

| 命令 | 登录 | 允许窗口 | 读写 | 策略 |
| --- | --- | --- | --- | --- |
| `host_info` | 否 | 全部 | 读 | 无 |
| `domain_overview` / `execution_snapshot` / `federation_snapshot` | 否（只读演示） | main/focus-* | 读 | 无 |
| `auth_status/register/sign_in/sign_out` | 否 | 全部 | 写本地 | 仅 auth 命令 |
| `audit_list` | 是 | settings/main | 读 | 会话 |
| `inference_status` | 否 | settings/main | 读 | 配置脱敏 |
| `inference_configure/forget/test/complete` | 是 | settings/main | 写/网络 | 会话 + 站点策略 |
| `execution_gate_set` | 是 | main/execution | 写 | 会话 + 审计（A0 修复） |
| `sandbox_read/write/list` | 是 | settings/main | 读写 | 会话 + 路径/大小/窗口 |
| `open_workspace_window` | 否 | main | 导航 | 允许 view 白名单 |
| `ontology_*` | 否（只读） | main/capabilities/agent | 读 | A0 后按窗口 |
| `agent_*` | 是 | main/agent | 写/网络 | 会话 + scope + policy |
| `whatif_*` | 是 | main/planning | 写（模拟） | 会话 + policy |
| `delivery_*` | 是 | settings/main | 写 | 会话 + 审批 + policy |

实现要求：

- 每个命令显式接收/读取 `tauri::Window` 并调用 `authorize(window, command, session, policy)`；不得依赖“所有窗口共享全部命令”。
- `capabilities/default.json` 仅为默认；为 settings/agent 窗口增加独立 capability 文件（或命令内 scope 检查），二者至少一种，且测试覆盖。
- `execution_gate_set` 改为先 `auth::require` 再修改；`sandbox_*` 改为 no-follow + 原子写（Phase D 复用）。
- 站点策略 `HostPolicy`：默认 `{ agentScopes:["read","propose"], remoteInference:false, autoUpdate:false, channel:"stable" }`；D 的 SQLite 覆盖；所有命令经同一 `policy.check`。

## 5. 当前模型与 digest

桌面唯一“当前模型”：

```text
SourceModel = Base-01 model.toml + pump-p1 = degraded patch
```

- host 在启动时加载并 patch，保存 `Arc<Model>` 与 `model_digest`（`DIGEST.md`），不再只保存 `DomainOverview`。
- `domain_overview`、`ontology_catalog`、`whatif_snapshot`、`execution` 都从同一个 `SourceModel` 构建；切换模型必须同时更新版本并拒绝旧引用。
- D 的 site bundle 增加 `ModelStore`：内置 fallback + imported bundle；优先级 `imported > built-in`，启动时校验 digest/契约/证据；MVP 不热切换正在运行的模型，导入后需重启/显式 reload（并记录审计）。
- 若 D 暂不实现 ModelStore，D 文档必须删除“导入后切换模型”的验收；当前设计选择实现 ModelStore（最小只读）。

## 6. 审批原语（B/C/D 共用）

```rust
pub enum ApprovalClass { Auto, OperatorConfirm, TwoPerson, Forbidden }

pub struct ApprovalToken {
    pub id: String,
    pub session_id: String,
    pub run_id: Option<String>,
    pub tool_or_action: String,
    pub subject_digest: String, // sha256 per DIGEST.md; tool/action specific
    pub policy_version: String,
    pub expires_at_ms: u64,
    pub used: bool,
}
```

- `OperatorConfirm`：宿主创建 pending token；UI 展示 subject/影响/引用；操作员批准后置 used；任何字段变化使 digest 不匹配。
- `TwoPerson`：Phase B/D 不开放（删除预留，避免与单操作员冲突）；需要时另立 ADR + 角色模型。
- `Forbidden`：直接拒绝并审计。
- B 的 agent、C 的 promote、D 的 channel/import 使用同一原语；不允许布尔 `approved` 参数。
- 所有 token 持久化（SQLite）以支持暂停/恢复/审计；过期自动拒绝。

## 7. 新 crate/依赖与工作区

新增（按阶段）：

| crate | 阶段 | 依赖 |
| --- | --- | --- |
| `poie-digest` | A | serde, serde_json, sha2 |
| `poie-objects` | A | poie-ontology, poie-digest, serde |
| `poie-sdkgen` | A (bin) | poie-objects, serde_json |
| `poie-governance` | B | poie-objects, poie-state (只读), poie-plan, poie-digest, serde_json |
| `poie-whatif` | C | ontology, plan, sim, state, viability, digest |
| `poie-delivery` | D | ontology/state/viability, digest, serde |

规则：

- 全部加入根 `Cargo.toml` `members`；`apps/desktop/src-tauri` 继续 `exclude`，桌面用 path 依赖并更新自己的 `Cargo.lock`。
- 每个 PR：`cargo fmt/clippy/test/doc`、`cargo-deny`、`check-test-matrix`、`check-docs`；新增依赖写 ADR（JSON Schema、serde、sha2、SBOM 工具）。
- 新增运行时依赖默认拒绝；能用手写/已有依赖实现就不引入（如 capability 用 opaque 句柄，不引入 JWT 库）。

## 8. 有限实现顺序（写入各阶段）

```text
A0 auth/scope/digest/DTO -> A1 catalog -> A2 SDK/UI
B1 policy/tool/run state/audit migration -> B2 approvals/evals/UI
C1 snapshot/perturbation/outcome -> C2 compare/promote/checkpoint
D1 manifest/model store/bundle -> D2 channel/plan/recall/UI
```

每个阶段完成后，更新 `docs/status/test-matrix.zh-CN.md`、`status-snapshot`、README 和本文件的实现状态；未通过校验不得开始下一阶段。
