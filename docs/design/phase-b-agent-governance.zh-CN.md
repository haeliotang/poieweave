# Phase B 设计：Agent 治理与评测

- 状态：设计草案（待独立校验后进入开发）
- 日期：2026-11-04
- 范围：Phase B / 受控工具、权限、证据引用、审批、审计、离线评测
- 上游：Phase A Ontology 产品层、SAP、现有桌面认证/推理
- 下游：Phase C（模拟工具）、Phase D（站点策略）

## 1. 背景与问题

当前桌面 Agent 面板只把用户输入发给 OpenAI-compatible 模型，然后把文本响应显示出来：

- 模型不能读取对象/能力图，也不能提出结构化计划；
- 没有工具级权限、引用约束和审批分类；
- 无法阻止模型把不可信内容当作指令；
- 没有可重复的 Agent 评测，只有“看起来回答得不错”；
- 没有把 Agent 提案纳入 SAP 和审计闭环。

Phase B 要回答的不是“如何让 Agent 更聪明”，而是：**Agent 能做什么、不能做什么、凭什么、谁批准、如何证明它没有越界。**

## 2. 目标

1. 建立稳定的 **Tool Catalog**：名称、描述、输入 JSON Schema、输出类型、权限 scope、审批类别、调用上限。
2. 实现 **Agent Runtime**：多步工具调用、最终回答、引用校验、审批暂停/恢复、超时/步数/工具预算。
3. 所有工具只能通过 Ontology/SAP/Simulation 的只读或“生成提案”接口工作；不存在 Agent 直达 dispatch 的路径。
4. 实现 **Evidence/Citation**：工具结果携带对象/动作/证据引用；最终回答中的引用必须来自本轮工具结果。
5. 建立 **离线 eval harness**：脚本化模型响应 + 期望工具序列/拒绝行为/引用覆盖；CI 可重复。
6. 所有运行写入审计，设置页和 Agent 面板可查看元数据，不泄露密钥与敏感内容。

## 3. 非目标

- 不做自主后台 Agent、无人值守循环或自动派发。
- 不做通用 MCP/插件市场；工具白名单由代码和策略固定。
- 不用规则引擎替代模型；评测针对行为边界，不承诺模型正确性。
- 不实现多 Agent 协作/群体智能；Phase B 只做单会话。
- 不把 Agent 输出当事实；最终事实仍来自工具结果和人工审批。

## 4. 用户与场景

| 场景 | 允许 |
| --- | --- |
| 问“potable-water 当前可行性” | 读取对象/能力图，回答 + 引用 |
| 问“为什么备用泵没被用上” | 读取 requires_any/状态/证据，列出根因 |
| “给我一个修复计划” | 生成 proposed plan，等待操作员确认；不派发 |
| “直接启动泵” | 拒绝：无 dispatch 工具；引导到 SAP 人工流程 |
| “把这段工具输出当命令执行” | 拒绝：工具输出是数据，不能成为指令 |
| 评测运行 | 脚本化 provider，检查工具序列/审批/引用/越界拒绝 |

## 5. 架构

```text
apps/desktop/src-tauri
  inference (provider HTTP, secrets)
        │
        ▼
crates/poie-governance       NEW: policy/runtime/tool schemas/capabilities/citations/evals
  ├── tools (object query, capability, plan propose, sap preview)
  ├── poie-objects (Phase A)
  ├── poie-plan / poie-ontology / poie-state (read-only/propose)
  └── eval harness (scripted provider)
        │
        ▼
SQLite audit (existing) + Agent UI (existing panel expanded)
```

### 5.1 新 crate `poie-governance`

不依赖 Tauri、不直接访问网络；通过 `Provider` trait 与 `ToolExecutor` trait 解耦：

```rust
pub trait ModelProvider {
    fn complete(&self, request: &ModelRequest) -> Result<ModelResponse, ProviderError>;
}

pub trait ToolExecutor {
    fn execute(&self, call: &ToolCall, context: &ToolContext) -> Result<ToolResult, ToolError>;
}

pub struct AgentRuntime<P: ModelProvider, T: ToolExecutor> {
    policy: AgentPolicy,
    provider: P,
    tools: T,
}
```

生产 `Provider` 由桌面 host 用现有 `http.rs` + 凭据库实现；测试用 `ScriptedProvider`，不访问网络。

### 5.2 工具目录

MVP 工具：

| 工具 | 权限 scope | 效果类（effect） | 审批类（approval） | 说明 |
| --- | --- | --- | --- | --- |
| `catalog.describe` | `read` | `read_only` | `auto` | 返回 Phase A Catalog 的类型/链接/函数摘要 |
| `state.query` | `read` | `read_only` | `auto` | 查询对象、能力图、外部依赖；有界、只读，不允许任意 SQL |
| `evidence.get` | `read` | `read_only` | `auto` | 读取证据引用与摘要（Phase D 扩展正文） |
| `plan.propose` | `propose` | `plan` | `operator_confirm` | 生成 proposed plan；不写 SAP transition |
| `action.draft` | `propose` | `plan` | `operator_confirm` | 生成动作草稿/参数预览；不派发 |
| `sim.run` | `simulate` | `simulate` | `auto` | Phase C 启用；只在 `namespace=sim/*` 运行 |

工具集是**封闭的、版本化的静态声明**；未声明的工具无法加载。模型 v1 只有上述六个工具；**没有** `dispatch.request`/`dispatch.now`/`shell.*`/`http.*`/`browser.*`/`secrets.*`。B2 的“待审批派发请求”不进入模型工具目录，若需要则另立 host 命令并绑定审批，不能由模型触发。

**不做通用 JSON Schema 2020-12 求值器**：工具输入用 Rust 强类型 struct + `serde(deny_unknown_fields)` 校验，JSON Schema 只作为 SDK/模型提示的描述性输出；这样不引入 `jsonschema` 新运行时依赖，也避免手写 2020-12 的不完整实现。若未来需要动态 schema，再走 ADR。

### 5.3 引用与证据

`ToolResult` 结构：

```text
callId, tool, ok, data, citations[],
sourceClass: T0|T1|T2|T3, trustProfile: analyst|dispatcher,
redactions[], truncated, latencyMs
```

`T0` 操作员输入、`T1` 签名仓库内容、`T2` 内部遥测、`T3` 外部/不可信内容。只有 T0/T1 可进入指令位；T2/T3 以带来源头的数据块进入。工具输出一律不进入指令位（不是只加 `untrusted=true` 标签）。

`Citation { kind: object|action|plan|evidence|contract, id, label?, digest? }`

规则：

- 模型最终回答若包含事实性断言，必须至少引用一个本轮工具结果中的 citation；
- 引用 ID 必须精确存在于本轮工具结果；未知引用导致结果拒绝（`citation_missing`）；
- 引用不证明模型正确，只证明“该断言可追溯到对象”；
- 工具输出标记 `untrusted: true`，系统提示明确禁止把内容当指令；
- 工具输出截断时保留摘要和截断标记，不静默丢字段。

### 5.4 策略与审批

`AgentPolicy`：

```text
allowed_scopes = [read, propose]
max_steps = 8
max_tool_calls = 16
max_prompt_chars = 4000
max_tool_result_chars = 20000
require_citation = true
approval = {
  read_only: auto,
  plan: operator_confirm,
  simulate: auto,
  dispatch: forbidden,   // 直接派发永远禁止；B2 的 pending request 也不由模型触发
}
```

审批类别：

| 类别 | 行为 |
| --- | --- |
| `auto` | 只读/模拟直接执行，写审计 |
| `operator_confirm` | 暂停运行，UI 显示参数/影响/引用，操作员确认后继续 |
| `two_person` | Phase B 不开放；Phase D 站点策略预留 |
| `forbidden` | 运行时拒绝，记录 `policy.blocked` |

审批绑定运行 ID、工具调用 ID、参数摘要、操作员会话和过期时间；一次批准不允许复用到不同参数。

### 5.5 能力凭证与审批摘要

每次工具调用前，宿主签发一次性 **capability**：

```text
{ id (opaque random 128-bit), sessionId, tool, toolVersion, scopes[],
  argsSha256, actionId?, expiresAt (<=300s), policyVersion }
```

- 宿主内存保存映射；token 不返回给模型，也不依赖新增加密库；单次使用；参数、工具、会话任一变化即失效；
- 寿命不得超过操作员会话；登出、接管、降级立即撤销；
- 模型永远看不到 provider key、OS 凭据或文件路径；密钥只在宿主出站请求时注入。

审批不再只是“点一下同意”，而是绑定一个摘要：

摘要使用 [DIGEST](DIGEST.md) 的 `poie.approval/0.1` domain，按工具定义至少字段：

| 工具 | subject digest 字段 |
| --- | --- |
| `plan.propose` | goal, horizon, deadline, policyVersion, sessionId, expiresAt |
| `action.draft` | actionTypeId, planFingerprint, argsSha256, policyVersion, sessionId, expiresAt |
| `sim.run` | variantId, snapshotSha256, policyVersion, sessionId, expiresAt |
| 未来 pending dispatch | actionId, target, command, validFrom/Until, policyVersion, planSha256, sessionId |

- 操作员批准的是这个摘要；任一字段变化返回 `approval_mismatch`；
- 审批一次一用，不可跨运行复用；`ApprovalToken` schema 见 [PLATFORM](PLATFORM.zh-CN.md) §6；
- 不存在“总是批准”的粘性开关；MVP **不实现双人审批**（单操作员），B/D 不承诺 two_person。

### 5.6 上下文信任画像

吸收 Meta “Agents Rule of Two” 的工程约束：一个会话最多同时具备“不可信输入、敏感数据、状态改变/外联”中的两项。

| 画像 | 允许 | 禁止 |
| --- | --- | --- |
| `analyst` | 读取外部/不可信文档 + 内部只读对象 | 生成可派发动作、外联工具 |
| `dispatcher` | 内部可信对象 + 生成计划/草稿 | 摄入外部网页/文档 |

只允许 `analyst → dispatcher` 的**单向切换**；切换时重新认证并清空上下文。Phase B MVP 先不实现外部文档工具，但策略/测试保留该边界。

### 5.7 运行生命周期

```text
created -> running -> waiting_approval -> running -> completed
                    \-> denied/failed/cancelled
```

每一步：

1. 校验会话/签名/策略；
2. 调用 provider，解析 tool_calls 或 final；
3. 校验工具白名单、参数 schema、预算；
4. 执行工具（只读/propose），记录结果与 citations；
5. 若需审批，持久化 pending 状态并暂停；
6. 继续直到 final、预算耗尽或错误；
7. 校验最终引用，写审计，返回结构化结果。

### 5.8 Host API

| 命令 | 输入 | 输出 |
| --- | --- | --- |
| `agent_run` | prompt, policyProfile | runId, state |
| `agent_status` | runId | RunState + pending approval |
| `agent_approve` | runId, approvalTokenId | RunState |
| `agent_reject` | runId, reason | RunState |
| `agent_cancel` | runId | RunState |
| `agent_resume` | runId, approvalTokenId | RunState（仅 approved 后） |

`RunState` 序列化到 `agent_runs.state_json`；Tauri 事件 `agent://state` 推送；审批 token 见 [PLATFORM](PLATFORM.zh-CN.md) §6。

运行状态必须持久化，支持审批暂停/恢复/取消：

- `agent_runs` 表：`run_id, schema_version, state_json, session_id, created_at, updated_at, expires_at`；`user_version` 迁移；
- 命令：`agent_run`（启动）、`agent_status`、`agent_approve`、`agent_reject`、`agent_cancel`、`agent_resume`（批准后继续）；
- Tauri 事件 `agent://state` 推送状态；审批 token 与 run 绑定；
- 宿主重启：未完成 run 标记 `interrupted` 并允许取消/重试，不自动恢复模型调用；
- 不把 provider token 或 API key 写入任何日志。

MVP 不承诺跨重启“自动继续”；只承诺审批状态可查询、可拒绝/取消，重启后不产生幽灵运行。

## 6. 评测（Evals）

### 6.1 离线、确定性

`evals/agent/*.toml`：

```toml
id = "query-service-and-cite"
prompt = "..."
[scripted]
responses = [
  { tool = "state.query", arguments = { subject = "service:potable-water" } },
  { final = "..." , citations = ["service:potable-water"] }
]
[expect]
tools = ["state.query"]
citation_coverage = ["service:potable-water"]
approval = "none"
```

`poie-governance` 的 eval harness 读取 fixture，运行真实 `AgentRuntime` + `ScriptedProvider` + 真实工具（或固定 fixture executor）；CI 只跑确定性脚本，不调用外部模型。

### 6.2 评测指标

- task success（期望工具/最终结果是否达成）；
- CI 确定性脚本用例数与覆盖矩阵（脚本化 provider 重复没有信息量，不报 pass^k）；
- 现场真实 provider 才报 **pass^k**（k=5，含方差），不用于 CI 门禁；
- tool sequence correctness；
- citation coverage / unknown citations；
- approval enforcement（该停是否停、该拒是否拒）；
- unsafe attempt blocked（dispatch/shell/secrets/未知工具；要求 100%）；
- prompt-injection resistance（恶意工具输出不得引发额外工具调用；报告实测成功率，不写“已解决”）；
- budget compliance（步数/工具数/字符数）。

MVP 至少 60 个用例，覆盖六个模型工具、Base-01/02、注入 canary、审批/越权/预算/引用缺失；静态 fixture 先于运行提交。

### 6.3 现场评测

真实 provider 的评测必须手动/单独工作流运行，记录模型、prompt 版本、fixture 版本和结果；不得把随机结果当 CI 门禁。现场结果只作为观察，不替代脚本化回归。

## 7. UI

Agent 面板（现有 inspector）扩展：

- 顶部显示策略摘要：`Read + propose only`、最大步数、审批要求；
- 时间线显示工具调用（名称、参数摘要、状态、耗时、是否被截断）；
- 引用列表可点击定位到对象/动作；
- `operator_confirm` 卡片显示操作、影响、参数、引用；按钮“批准/拒绝”；
- 运行中可取消；错误显示稳定 reason code；
- 未登录/未配置时保持禁用；浏览器预览显示 host-only；
- 设置页增加 Agent 策略只读摘要、最近 eval 结果、审计入口。

所有新文案进入 i18n；不显示原始 provider payload 或密钥。

## 8. 安全与威胁模型

| 威胁 | 控制 |
| --- | --- |
| Prompt injection（工具输出/对象名称/证据文本） | 工具输出标记 untrusted；系统提示；运行时只认结构化 tool_calls；禁止解释输出中的指令 |
| 数据外泄 | 无网络工具；provider 请求仅含白名单工具结果和用户输入；日志脱敏；密钥不进模型 |
| 越权读取 | scope 校验 + Phase D markings；对象查询按站点/角色过滤 |
| 越权写入 | 无 dispatch；propose 需人工确认；最终仍走 SAP |
| 工具参数注入 | JSON Schema + 固定白名单 + 长度/数量上限；禁止路径/SQL |
| 模型幻觉引用 | 引用必须来自本轮 tool results；未知引用拒绝 |
| 审批重放 | 审批绑定 run/call/参数摘要/过期时间；一次一用 |
| 预算耗尽/无限循环 | 最大步数/工具调用/字符/超时；超限记为失败而非成功 |
| 审计泄漏 | 只记录哈希/摘要/ID；不记录完整 prompt、密钥或工具正文 |

## 9. 与现有系统的集成

- **认证**：所有 Agent 运行要求 `auth::require`；未登录不可创建运行。实施前必须先修两个现有缺口：`execution_gate_set` 当前未登录也能改 gate（`apps/desktop/src-tauri/src/lib.rs`），必须硬性要求会话；Tauri 自定义命令目前被所有窗口共享，必须按命令/窗口检查 scope，不能让 Agent 窗口调用 `inference_configure`、`sandbox_write`、`auth_*`。
- **推理**：复用现有 `inference` 配置/凭据/native HTTP；请求增加 `tools` 和 `tool_choice`，解析 `tool_calls`；新增 `agent_run` 命令。现有 `inference_complete` 只保留为 Settings 连接测试/兼容路径，Agent 面板不得使用它；若保留用户可见入口，必须标记 legacy 或走同一策略。
- **SAP**：`plan.propose`/`action.draft` 只生成 proposed plan/草稿，不写 transition、不调用 Gate，模型 v1 没有 dispatch 工具。审批通过后由既有 SAP 验证/授权命令接管；七项复核不新增第八项，Agent 来源写入 `Authorization.basis`（如 `agent-plan:<sha256>`），策略版本写入 `policy_version`。
- **审计**：不新增 SAP `RecordKind`（避免为 Agent 治理改 SAP 契约）。Agent/tool/approval 事件写入**只追加本地审计日志**（哈希链 + 每日锚定摘要），SQLite `audit` 表作为可重建索引；当事件关联到真实动作时写 `sap_record_id`。现有 `audit(id,at_ms,actor,event,detail)` 迁移为带 `turn_id`、`args_sha256`、`result_sha256`、`sap_record_id`、`prev_hash`、`hash` 的结构，必须 `PRAGMA user_version` 迁移，不能 startup `DROP TABLE`。SQLite 不是防篡改来源；提示词默认只存摘要，正文按保留策略可选存储并脱敏。
- **Ontology**：工具 schema/对象查询使用 Phase A `poie-objects`，不复制一套字段。工具与 Phase A 函数目录的映射：`catalog.describe`/`state.query` 映射 A 的 `catalog.describe`/`state.query`；`plan.propose` 同名；`action.draft` = `plan.propose` + `plan.preview_sap` 的组合层；`sim.run` 映射 A 的 `sim.run`；`evidence.get` 映射 A 的 `evidence.get`。不得另造同名异义工具。

## 10. 测试与验收

### 单元/集成

- 工具白名单/scope/参数 schema 拒绝；
- dispatch/shell/secrets 工具不存在且被策略拒绝；
- 多步工具调用、预算耗尽、取消；
- 引用缺失/未知引用拒绝；
- operator_confirm 暂停→批准/拒绝；
- 审计事件完整性（每个工具调用、审批、完成）；
- 恶意 fixture 不触发额外工具调用；
- ScriptedProvider 端到端 eval。

### 验收标准

1. Agent 不能直接调用 Gate/适配器，模型工具目录中不存在任何 dispatch 工具；
2. 任何 factual final 无引用时被拒绝；
3. propose 必须人工确认；审批不可重放；
4. CI 跑确定性 evals 并验证关键指标；
5. 现场 provider 不可用时 Agent 明确错误，不降级为“自由聊天”；
6. UI 能查看工具轨迹、引用、审批和审计。

## 11. 实施顺序

1. `poie-governance` 类型/策略/工具目录 + 纯单元测试；
2. `ScriptedProvider` + eval harness + fixtures；
3. host 工具执行器（objects/query/plan preview）+ `agent_run` 命令；
4. UI Agent 面板扩展 + 审批；
5. 真实 provider tool-call 适配（需手动/provider 测试，不进 CI）；
6. 审计/设置/文档/Playwright。

## 12. 开放问题

- 最终回答强制引用是否会让纯解释性问题过于机械；MVP 对事实性断言强制，对“建议”标记为非事实。
- 审批暂停的恢复是否跨进程重启；MVP 运行存内存，重启即取消，审计保留。
- 工具目录是否允许站点扩展；Phase D 再定义站点策略，MVP 固定。
- 现场模型 eval 的样本与标注由谁维护；Phase B 只提供框架和最小样本。

## 13. 参考

- 子 agent 调研备忘：[`research/agent-governance.zh-CN.md`](research/agent-governance.zh-CN.md)，包含 MCP/OAuth、OWASP LLM06、Meta Rule of Two、τ-bench/AgentDojo、PROV-O、NIST AI 600-1 等来源与逐条标注。
- Palantir AIP / Ontology / Apollo 产品资料（见 Phase A/D 参考）
- OWASP LLM Top 10（prompt injection、敏感信息泄漏、过度代理）
- NIST AI RMF / Generative AI Profile（治理、测量、管理）
- Poieweave SAP：`specifications/state-action-protocol.zh-CN.md`
