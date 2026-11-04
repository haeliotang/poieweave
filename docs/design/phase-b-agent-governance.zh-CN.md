# Phase B 设计：Agent 治理与评测

- 状态：设计草案（Phase B 门禁重跑后方可开发）
- 日期：2026-11-05
- 范围：受控工具、审批状态机、引用协议、审计链、离线评测、桌面 Agent 面板
- 上游：Phase A0/A1（已合并）、SAP、现有认证/推理/HTTP
- 下游：Phase C（sim.run）、Phase D（站点策略/evidence）

## 1. 背景与问题

现有 Agent 面板只是把 prompt 发给 OpenAI-compatible 模型并显示文本：没有工具、引用、审批、审计链或可重复评测。Phase B 要把它变成**受治理的、可暂停/恢复的、可重建审计的**工具调用代理；核心不是让模型更自由，而是限制它能做什么、凭什么、谁批准。

## 2. 目标

1. 封闭的版本化 Tool Catalog：每个工具的参数/输出/citation/上限/审批类都是代码契约。
2. 审批是**先于执行**的持久状态机；审批 token 原子消费、一次一用、可过期/拒绝。
3. 最终回答使用 `answer.emit` 机器可读协议；每条事实 claim 必须引用本轮工具结果。
4. 只追加审计链 + SQLite 可重建索引；`user_version` 迁移，不再启动 DROP。
5. 确定性离线 eval + 现场 pass^k 分离；CI 只跑 scripted provider。
6. Agent 面板展示工具轨迹、引用、审批卡、取消/恢复；旧自由聊天入口禁用。
7. 不引入 JSON Schema 求值器；工具参数用 Rust 强类型 + `serde(deny_unknown_fields)`。

## 3. 非目标

- 不做自动派发/后台自治/自批；无 `dispatch` 工具。
- 不做通用 MCP/插件/浏览器/shell/网络工具；B1 不接远程工具。
- 不做多 Agent 协作、长期记忆、隐藏 CoT 存储。
- 不把模型文本当观测/验证；不宣称通过任何合规认证。
- 不做双人审批；B/D 均为单操作员。
- 不把 eval 当现场安全证明；CI 不调用真实模型。

## 4. 架构

```text
apps/desktop/src-tauri
  inference.rs (provider HTTP / secrets)
  agent.rs     NEW: runtime registry, commands, audit, UI events
        │
        ▼
crates/poie-governance      NEW: policy/tool contracts/approval/run state/evals
  ├── poie-objects (A Catalog)
  ├── poie-state (records/review read-only)
  ├── poie-plan (propose/preview), poie-ontology
  └── poie-digest (approval/audit hashes)
        │
        ▼
SQLite agent_runs/approval_tokens + append-only audit log
```

`poie-governance` 不依赖 Tauri、网络、`poie-exec`；它不构造 `Authorization`、不写 `ActionLedger`、不调用 Gate。host `agent.rs` 负责 provider HTTP、审批 UI 回调和审计落盘。

## 5. 前置 B0（实现第一步）

| 前置 | 决策 |
| --- | --- |
| Session ID | `auth::Session` 增加 `session_id: String`（128-bit 随机，hex）；创建会话时生成；登出即失效 |
| 审批表 | 新增 `approval_tokens`（见 §7）；所有审批必须持久化，不能只在内存 |
| 审计 | 新增 `poie.agent-audit/0.1` 记录与 `agent-audit.log`；`db.rs` 改 `PRAGMA user_version` 迁移，删除启动 DROP |
| gate 鉴权 | A0 已修 `execution_gate_set`；B 不再重复实现，只读取状态 |
| 错误包络 | `agent_*` 命令返回 `{code,message,details}`；先新增 `CommandError`，其余命令逐步迁移 |

## 6. 工具目录（B1 六个，封闭）

| 工具 | 版本 | 参数（Rust struct） | 输出 | citation | 审批 | 每轮上限 | 执行入口 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| `catalog.describe` | 1.0.0 | 无 | `OntologyCatalog` | 无 | auto | 1 | A `ontology_catalog` |
| `state.query` | 1.0.0 | `{subject,asOf?}` | `{objects[],links[],capabilities[],verdict?,dependencies[]}` | object/report | auto | 4 | A objects + `graph` |
| `plan.propose` | 1.0.0 | `{goal,horizonS,deadlineS?}` | `PlanSummary{planFingerprint,actions[],completionS}` | plan | auto（纯计算） | 2 | `poie-plan::plan` |
| `plan.preview_sap` | 1.0.0 | `{planFingerprint}` | `SapPreviewDto{checks[],assumed[],rejections[],authorizedActions:0,authorizationCreated:false}` | plan | auto | 2 | 只读 `poie-plan::preview_plan_through_sap`（host 解析 fingerprint） |
| `action.draft` | 1.0.0 | `{planFingerprint,actionId}` | `ActionDraft{planFingerprint,action,preview}` | plan/action | **operator_confirm** | 2 | 宿主从已算 plan 构造 DTO；审批后才执行 |
| `answer.emit` | 1.0.0 | `{claims[],suggestions[]}` | `{accepted,rejectedClaims[]}` | claim 内嵌 | auto（终态） | 1 | runtime 校验 |

A 的 `poie-objects::functions()` 在 B 首 PR 同步改为这同一组六个函数（删除 `evidence.get/sim.run`，新增 `action.draft/answer.emit`），SDK 重新生成；**B 的工具名/参数/输出是 A 函数目录的唯一投影**，不得存在第二个目录。

- 没有 `dispatch.*`/`shell.*`/`http.*`/`browser.*`/`secrets.*`；`sim.run`/`evidence.get` 留给 C/D，B1 不向模型暴露。
- 工具名/版本/参数/输出/citation/approval 在 `poie-governance::tools` 中静态声明；模型提示里的 JSON Schema 只是描述，运行时用对应 Rust struct + `deny_unknown_fields` 校验。
- **`plan.propose` 是纯计算 auto**；`action.draft` 在审批前不执行。审批展示的 subject 是 `planFingerprint + actionId + argsSha256`，因此操作员批的是已存在的计划中的具体动作，而不是“未来计划”。

### 6.1 只读 SAP preview（B2）

复用 Phase A 定义的 `poie-plan::preview_plan_through_sap(plan, assumed)`（A §7.2.1），host 按 `planFingerprint` 解析 Plan：

- 只使用函数内 throwaway ledger 做 precondition/review；**不写任何 live/external ledger、不创建/保存 Authorization、不产生 Authorized transition**；返回 `authorizedActions=0`、`authorizationCreated=false`；
- ownership/reservations/authorization 默认 `assumed`，UI 单独显示；
- `ActionDraft` 只能使用该 preview，禁止调用 `validate_plan_through_sap`；
- 结构测试：`poie-governance` 无 `poie-exec` 依赖；源码中不出现 `Authorization {`；preview 只写 throwaway local ledger，返回 `authorizedActions=0`/`authorizationCreated=false`，不触碰 live/external ledger。

## 7. 审批状态机（B3/B5）

`approval_tokens` 表：

```text
id TEXT PRIMARY KEY, session_id, run_id, call_id, tool, tool_version,
subject_digest, policy_version, args_json_redacted, impact_json,
citations_json, state TEXT, created_at, decided_at, started_at, expires_at
```

`state ∈ pending|approved|rejected|expired|executing|interrupted|consumed|cancelled`。

单一原子状态机（并发安全）：

1. `agent_approve`：`pending → approved`；`agent_cancel`：`pending|approved → cancelled`；
2. `agent_resume`：`approved → executing`（原子条件更新）；只有影响 1 行才签发 capability 并执行一次；成功 `consumed`，业务失败 `approved`（可重试），系统崩溃由启动恢复处理；
3. 启动恢复：`executing` token 若 `started_at` 超过租约（默认 120s）→ `interrupted`；操作员可 `agent_resume`（interrupted→executing，要求同一 callId 的幂等边界）或 `agent_cancel`（→cancelled）；
4. 空操作幂等：tool call 以 `(runId, callId)` 为幂等键；重试先查已完成结果，不重复副作用；`action.draft` 只读，重复执行只返回同一 DTO；
5. `agent_reject`：`pending → rejected`；过期由读取/消费时判定并写 `expired`；
6. 两个 resume 并发时只有一个能 `approved → executing`，另一个得到 `approval_mismatch`。

流程固定为 **审批 → 签发 capability → 执行**：

1. runtime 需执行 `operator_confirm` 工具时，先构造完整 args/impact/citations 与 `subject_digest`，写 `pending` token，暂停；
2. UI 展示 subject/args/impact/citations/过期；
3. `agent_approve` 原子置 `approved`；`agent_reject` 置 `rejected`；
4. `agent_resume` 原子置 `executing`，签发 capability（寿命 ≤ min(300s, session 剩余)），执行一次，置 `consumed`；
5. 任何字段变化返回 `approval_mismatch`；过期返回 `approval_expired`；`action.draft` 审批后才产生 draft。

`plan.propose` auto；`action.draft` 审批；`answer.emit` auto。

Capability 生命周期：审批通过后才签发；绑定 sessionId/tool/version/argsSha256/tokenId；单次使用；128-bit 随机来自 `getrandom`；寿命取 `min(300s, session.expires_at-now)`；登出/接管/降级/取消时从 runtime registry 撤销；过期由执行前检查。测试覆盖签发时机、过期、参数变更、跨会话复用、撤销。

## 8. Provider 循环与运行状态（B4/R3）

- `Provider` trait 同步；host 在 `spawn_blocking` 中运行；**DB 锁只在读写状态/审计时持有**，不在网络调用期间持有。用 `RuntimeRegistry`（`Mutex<HashMap<runId, RunHandle>>`）管理活跃 run，审批/取消通过 registry + 状态表协调。
- 请求：`tools`、`tool_choice=auto`、`timeout_ms=30s`、`max_response_bytes=262144`（按 UTF-8 字节，不是字符）、`max_context_bytes=32768`；每次请求前检查取消标志；取消后不得落“成功”审计。
- 解析：支持多个 `tool_calls`（B1 顺序执行）、坏 JSON/缺参 → `invalid_tool_arguments`；`finish_reason` 缺失/非预期 → `provider_protocol`；超时/超大响应 → `provider_timeout`/`provider_response_too_large`。
- 同一 session 同时最多一个 active run；`agent_run` 第二个请求返回 `run_active`。
- 预算：`max_steps=8`、`max_tool_calls=16`、`max_prompt_bytes=4096`（与现有 inference 的 4000 字符近似；实现按字节校验）、`max_tool_result_bytes=20480`；工具结果截断必须保留 JSON 合法结构与 citations，截断标记 `truncated=true`。

`RunState`（持久化到 `agent_runs.state_json`，schema `poie.agent-run/0.1`）：

```text
schemaVersion, runId, sessionId, profile, status,
turns[{ index, user?, assistant?, toolCalls[{
  id, tool, toolVersion, argsSha256, argsJsonRedacted,
  resultSha256, resultSummary, citations[], evidenceRefs[], truncated,
  decision, startedAt, finishedAt
}]}],
pendingApproval?{ tokenId, tool, subjectDigest, argsJsonRedacted, impact, citations[], expiresAt },
budget{stepsUsed,toolCallsUsed,promptBytesUsed},
citations[], error?{code,message}, createdAt, updatedAt
```

- 恢复：重启后未完成 run 标 `interrupted`，模型调用不自动恢复；已批准但未消费的 token 保留，可查询/取消。
- `agent://state` 事件发送完整 `RunState`（有界，≤64 KiB）；UI 也可轮询 `agent_status`。
- `agent_cancel` 在 provider 调用期间设置取消标志；调用返回后丢弃结果并写 `cancelled`。

## 9. 引用与最终回答（B6/R4）

统一使用 `EvidenceRef`（PLATFORM §5.1）：`{kind,id,fingerprint?,sha256?,asOf?,source}`，至少一个 digest/fingerprint。`Citation` 直接复用该结构，不再定义第二套字段。

`answer.emit` 协议：

```json
{"claims":[{"text":"...","citations":[{"kind":"object","id":"service:potable-water","sha256":"...","source":"model"}]}],
 "suggestions":[{"text":"...","basis":["..."]}]}
```

- 每条 claim 至少一个 citation；citation 必须精确存在于本轮工具结果的 citations/evidenceRefs；
- 未知/缺失 → `citation_missing`，`answer.emit` 被拒绝；模型可重试或输出 suggestion；
- `suggestions` 明确标为未验证建议，不参与事实校验；
- 最终 UI 只渲染 claim/suggestion 文本，citation 可点击定位；不把模型文本写入 SAP verification。

## 10. 审计（B7）

记录 schema `poie.agent-audit/0.1`（JSONL）：

```json
{"schema":"poie.agent-audit/0.1","eventId":"...","atMs":0,"actor":"operator|agent",
 "sessionId":"...","runId":"...","turnId":0,"callId":null,
 "tool":null,"toolVersion":null,
 "decision":"allowed|denied|approval_required|approved|rejected|completed|failed|cancelled|interrupted",
 "reason":"...","argsSha256":null,"resultSha256":null,"sapRecordId":null,
 "prevHash":"...","hash":"..."}
```

- `hash = sha256("poie.agent-audit/0.1\n" + canonical_json(record_without_hash))`，含 `prevHash`；
- genesis：`prevHash = null`，文件首行；每行单写者、追加 + `fsync`，尾部残行在读取时忽略并记录；
- 每日 `anchors/<date>.json`：`{date, lastHash, prevAnchorHash}`；锚点链只防本地截断/替换的检测，不防整体替换；
- `db.rs` 用 `PRAGMA user_version` 迁移：`audit` 表加 `event_id/schema/session_id/run_id/turn_id/call_id/tool/tool_version/decision/reason/args_sha256/result_sha256/sap_record_id/prev_hash/hash/legacy`；历史行标记 legacy，不伪造链；删除启动 DROP；重建时按这些字段与日志逐字段比对。
- 重建：`audit_rebuild`（或测试函数）从日志重放派生 SQLite 索引，结果与索引一致；链断/锚点不匹配 `audit_integrity_failed`。

## 11. 策略与画像（R2）

```text
analyst:    catalog.describe, state.query, plan.propose, plan.preview_sap, answer.emit
dispatcher: analyst + action.draft
```

- analyst 不能 `action.draft`；dispatcher 也不得摄入外部不可信文档（B1 无外部文档工具）。
- 只有 `dispatcher` 可触发 `operator_confirm` 的 draft；`action.draft` 审批后仍只是 draft，不写 SAP。
- 不存在 `dispatch`；`policyVersion="poie-agent/0.1"` 固定。

## 12. 评测（R5）

Fixture schema `poie.agent-eval/0.1`：

```toml
schema = "poie.agent-eval/0.1"
id = "..."
preregistered_at = "2026-11-05"
prompt = "..."
profile = "analyst|dispatcher"
[script]
responses = [{ tool = "...", args = {...} }, { final = "..." }]
[approval]
action = "approve|reject|expire"  # optional
[expect]
tools = [...]
claims = [{ text = "...", citations = ["object:..."] }]
error = "citation_missing"          # optional
```

- B1 至少 **72** 个确定性用例（6 工具 × 正常/坏参数/超预算/未授权 = 24，再 × 引用/审批/注入 = 72）；覆盖矩阵：

| 工具 | 正常 | 坏参数 | 超预算 | 未授权 |
| --- | --- | --- | --- | --- |
| catalog.describe | 3 | 3 | 3 | 3 |
| state.query | 3 | 3 | 3 | 3 |
| plan.propose | 3 | 3 | 3 | 3 |
| plan.preview_sap | 3 | 3 | 3 | 3 |
| action.draft | 3 | 3 | 3 | 3 |
| answer.emit | 3 | 3 | 3 | 3 |

`scripts/check-agent-evals.py` 校验 fixture schema、计数、工具覆盖与审批脚本。
- 脚本化 provider 只测 runtime/policy，不测模型能力；现场真实 provider 才报 pass^k（k=5），不进 CI 门禁。
- 注入指标拆成“运行时 containment 必须 0 次越权/外联”与“模型是否被说服（仅现场报告）”。

## 13. Host API 与 UI（R1/R6）

命令：`agent_run`、`agent_status`、`agent_approve`、`agent_reject`、`agent_cancel`、`agent_resume`，全部 `Result<T, CommandError>`：

```rust
pub struct CommandError { pub code: String, pub message: String, pub details: serde_json::Value }
```
错误码：`unauthorized/scope_denied/unsupported_schema/policy_blocked/run_active/run_not_found/run_interrupted/approval_required/approval_mismatch/approval_expired/approval_rejected/invalid_tool_arguments/provider_timeout/provider_response_too_large/provider_protocol/budget_exceeded/citation_missing/audit_integrity_failed/not_implemented`。

- Agent 面板要求登录 + 已配置推理；显示 profile、工具轨迹、citations、审批卡（subject/args/impact/citations/expiry）、取消/重试。
- 浏览器预览保持 host-only。
- `inference_complete` 只保留 Settings 连接测试；Agent 面板改走 `agent_run`，不得回退到自由聊天。
- `agent://state` 负载为完整 RunState（有界）。

## 14. 依赖与过程门禁（R7）

- `poie-governance` 依赖：`poie-objects`、`poie-state`（只读）、`poie-plan`、`poie-ontology`、`poie-digest`、`serde/serde_json/toml`；**不依赖** `poie-exec`、Tauri、网络。
- 加入根 `members`；根/桌面 `Cargo.lock`、test-matrix/status-snapshot、desktop workflow `crates/poie-governance/**` 路径、cargo-deny、ADR（新依赖）、`check-docs`。
- 补 `scripts/check-schema-registry.py`（A0 承诺）和 `scripts/check-agent-evals.py`。
- 实施顺序：B0 前置 → `poie-governance` 类型/工具契约 → 审批/审计迁移 → 只读 preview → host agent loop → evals → UI。

## 15. 测试与验收

### 单元/集成

- 工具参数严格拒绝未知字段；无 dispatch 工具；profile 白名单；
- 审批：pending→approve/reject/expire/executing/interrupted/cancelled/consumed 原子性；参数/session/subject 改变 → mismatch；过期拒绝；崩溃后 executing→interrupted→重试/取消；
- preview 只写 throwaway local ledger，不创建 Authorization、不产生 Authorized transition；返回 authorizedActions=0/authorizationCreated=false；
- `answer.emit` 无 citation/未知 citation 拒绝；suggestions 不校验为事实；
- 审计链 genesis/prevHash/hash/锚点/尾部残行/重建/迁移；
- capability 签发/过期/撤销/跨会话复用；
- provider 多 tool_calls/坏 JSON/缺参/超时/超大响应/取消；
- 确定性 eval 72 例 + 覆盖矩阵；CI 不访问网络。

### 验收

1. 模型工具目录无 dispatch；越权请求 100% 拒绝并留痕；
2. 所有 operator_confirm 在执行前审批；token 一次一用；
3. 事实性 claim 无 citation/未知 citation 被拒；
4. 审计链可检测断裂、可重建；SQLite 迁移不丢历史；
5. CI 确定性 eval 绿；现场 pass^k 单独记录；
6. UI 可完成 approve/reject/cancel/restart-interrupted 流程；
7. 旧自由聊天入口不再调用 `inference_complete`。

## 16. 开放问题

- B1 是否需要在 `state.query` 中支持外部文档；暂不支持，Rule of Two 留到 D。
- 审计日志保留策略与磁盘上限；MVP 保留并记录大小，超限拒绝新 run。
- 现场 eval 标注/抽检流程；Phase B 只提供框架，不承诺统计功效。
- `pass^k` 的 k/样本数；默认 k=5，现场报告记录方差。

## 17. 参考

- [agent-governance 研究备忘](research/agent-governance.zh-CN.md)
- [PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)
- SAP §7.3/§7.4；ADR-0003 append-only log
