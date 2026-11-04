# Phase B 设计门禁校验（独立）

- 校验对象：[Phase B](phase-b-agent-governance.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)、[README](README.zh-CN.md)、[研究备忘](research/agent-governance.zh-CN.md)；代码 `apps/desktop/src-tauri/src/{auth,db,inference,http,lib,objects}.rs`、`crates/poie-{state,plan,objects,digest}`、`apps/desktop/src/{App.tsx,i18n.ts}`、`Cargo.toml`、CI。
- 快照：`main` HEAD **`892d018`**；写本文件前工作树干净。本报告只对该快照负责。
- 方法：只读对照；未跑全量 `cargo test`，未调用真实 provider；未改源码/设计/git，仅写本文件。
- 基线：A0/A1 已合并（`lib.rs:188-208` 的 `execution_gate_set` 已 `auth::require`）；C/D 不参与本判定。

## 0. 判定

**CHANGES_REQUIRED。** 方向与边界正确（无 dispatch、A 复用、确定性 eval），但 7 个阻断项集中在“契约未冻结”和“审批/审计状态不可实现”；均可在不重写架构的前提下就地闭合。

## 1. B0 四前置

| 前置 | 当前事实 | 判定 |
| --- | --- | --- |
| session id | `auth.rs:22-26` 的 `Session` 只有 display_name/expires_at_ms，无 id；B:161/171/228 与 PLATFORM:141 却要求 `sessionId` | **BLOCKER-3**（capability/审批/run 全部依赖它） |
| approvals 表 | PLATFORM:152 要求 token 持久化；B:237 只列 `agent_runs`，无 token 表/列/状态/原子消费；PLATFORM:137-147 的 `used: bool` 不足以表达 approved/rejected/expired | **BLOCKER-3** |
| audit 链/格式 | B:318 只有散文；无记录字段全集、哈希输入/domain/genesis/anchor 格式；`db.rs:58-89` 每次启动 `DROP TABLE`、无 `user_version` | **BLOCKER-7** |
| `execution_gate_set` 鉴权 | `lib.rs:188-208` 已先 `auth::require`（A0 修复） | **PASS**；B:315 的“当前未登录也能改”是旧句，须删 |

## 2. BLOCKER（B 实施前必须关闭）

### B1 工具契约未冻结，六个工具对 A 只是“同名”
- B:90-103 只有名称/scope/审批类，没有每个工具的参数字段、输出结构、上限、citation 形态；B:257 fixture 的 `subject` 在任何契约中未定义。
- `poie-objects/src/lib.rs:387-405` 的 A 函数目录 `input_schema/output_schema` 均为 `{"type":"object"}`；host 实际只有 `objects.rs:9-41` 的 catalog/objects/action preview；`plan.propose`、`evidence.get`、`plan.preview_sap`、`sim.run` 无可执行入口；A 还把 `evidence.get` 标为“Phase D 启用”（phase-a:123）。
- 要求：逐工具表（Rust 入口、args struct、output DTO、citation 规则、上限、B1 是否暴露），说明哪些是 B 新实现；不得以“同名”代替行为契约。

### B2 `action.draft` 的 SAP 预览会伪造 Authorization
- B:319 把 `action.draft` 定义为 `plan.propose` + `plan.preview_sap`；`poie-plan/src/sap.rs:56-85` 的 `to_action` 构造 `Authorization{by:"planning-validation",scope:"dispatch"}`，`123-128` 把每个动作推进到 `Authorized`（内存 ledger）。
- 这与 B:101（不写 transition）、B:336（不能调用 Gate/适配器）以及 A1 预览自己声明“no authorization is created”（`objects.rs:63-66`）冲突。
- 要求：定义只读 preview（只返回 assumptions/rejections，不构造 Authorization、不 apply ledger），或从 `action.draft` 删除 SAP 预览；补“治理 crate 不依赖 poie-exec/Gate、不构造 Authorization”的结构测试。

### B3 审批 token 无表、无调用绑定、无状态机
- PLATFORM:152 要求所有 token 持久化，但 B:237 只列 `agent_runs`；无 approvals 表、列、唯一约束、过期清理。
- PLATFORM:137-147 的 `ApprovalToken` 缺 `call_id`/`args_sha256`/批准时间/拒绝原因；B:154 却要求绑定“运行 ID、工具调用 ID、参数摘要、会话、过期”。`used` 无法区分 approved/rejected/expired/consumed，B:219-221 的 `agent_approve` 与 `agent_resume` 职责因此不确定。
- 要求：`approval_tokens` 表（id/session/run/call/tool/tool_version/subject_digest/policy_version/expires/state/created/decided），`UPDATE ... WHERE state='pending' AND expires_at>?` 原子消费；`session` 为随机 id（R9）。

### B4 RunState 只存哈希，恢复/审批展示与当前 DB 锁模型互相矛盾
- B:226-228 的 turns 只有 `argsSha256/resultSha256`，`pendingApproval` 只有 tokenId/tool/subjectSha256/expiresAt；没有参数、结果、影响、citations。恢复时无法重建 provider 上下文，UI 也拿不到 B:292 要求的“操作/影响/参数/引用”。
- `lib.rs:117-128` 现在在 `spawn_blocking` 内先 `db.lock()` 再做 30s HTTP；多步 Agent 若照搬，`agent_status/agent_approve` 会被同一个 Mutex 锁死，审批暂停无法实现。
- 要求：二选一并写入设计——(a) 内存 runtime registry + 通道/condvar，SQLite 只存状态/审计，锁只在读写瞬间持有；或 (b) 持久化有界、脱敏的对话/工具结果并定义恢复算法；同时定义 cancel 在 provider 调用中途的语义（cancel 后不得落结果/审计成功）。

### B5 审批发生在执行之后；`plan.propose` 的“批准”没有对象
- B:202-210 的顺序是 3 校验→4 执行工具→5 若需审批再暂停；B:151 又说 operator_confirm 要“暂停运行，UI 显示参数/影响/引用，确认后继续”。按文字实现会先执行后审批。
- `plan.propose` 被标 `operator_confirm`（B:97），但产物（计划）在执行后才存在；批准摘要只含 `goal/horizon/deadline/...`（B:176），不含 planFingerprint，操作员批的是“请求”而非计划。
- 要求：固定顺序为“审批→签发 capability→执行”；明确 `plan.propose` 是纯计算（auto）还是“先算后批”（批准 plan digest，再允许 `action.draft`/导出）；若保持 confirm，必须定义可展示的 impact 与 plan digest 绑定。

### B6 最终回答的引用协议未定义
- B:121-123 要求事实性回答带 citation、未知引用 `citation_missing`；B:355 却把“事实性 vs 建议”留作开放问题；真实 provider 只返回文本，B:257 的 `final/citations` 只是 ScriptedProvider 的内部形状。
- 没有 `answer.emit` 工具、JSON 输出格式或 citation 语法；运行时无法判断“哪些断言是事实”，验收 #2（B:337）不可测。
- 要求：定义 final 的机器可读协议（推荐只读 `answer.emit{claims[],citations[]}` 或受约束 JSON），逐条 claim 校验 citation 存在且来自本轮结果；无/未知 citation 稳定拒绝；明确“建议/假设”的标注方式。

### B7 审计哈希链/日志格式不 concrete，迁移不可执行
- B:318 只有散文；未定义记录字段全集、canonical 哈希输入（是否含 `prev_hash`、是否用 `poie.audit/0.1`）、genesis、anchor 文件命名/链、单写者/fsync/尾部残行、重建入口。DIGEST:18-26/44-50 没有 audit domain；SCHEMAS 也未登记。
- `db.rs:58-89` 每次启动 `DROP TABLE events/notes`，没有 `PRAGMA user_version`；`audit` 表无 `turn_id/args_sha256/...`，历史行无法回溯成链。
- 要求：冻结 `poie.agent-audit/0.1` 记录（event_id/at/actor/session/run/call/tool@version/decision/reason/args_sha256/result_sha256/prev_hash/hash），复用 ADR-0003 的追加/fsync/尾部规则；`user_version` 迁移 + legacy 段 genesis；anchor 链（anchor_n 含前一日 hash）并明确“本地 anchor 被整体替换不可防”的测试边界；补从日志重建 SQLite 的入口。

## 3. REQUIRED-BEFORE-B

- **R1 错误包络/错误码**：`lib.rs` 全部命令仍 `Result<T,String>`，与 PLATFORM:39-64 的 `{code,message,details}` 不符；`citation_missing`/`not_implemented`/`provider_*`/`run_*`/`invalid_tool_arguments` 未登记。`agent_*` 必须先落 `CommandError` 与码表，UI 才可能显示稳定 reason code。
- **R2 profile 与 Rule of Two**：B:190-193 的 analyst 禁止“生成可派发动作”，但 B:132 对两个 profile 都给 `read+propose`，`plan.propose/action.draft` 正是可派发提案；B1 又没有外部文档工具，profile 不可观测。要求按 profile 给工具白名单（analyst 只读；dispatcher 可 propose）并测试，或明确 B1 两个 profile 都只读。
- **R3 provider 循环与 HTTP 限制**：`http.rs:18-32` 无响应体上限；`inference.rs:146-151` 固定 body。要求 `tools/tool_choice`、tool_calls 适配（并行调用、坏 JSON、缺参、`finish_reason`）、`timeout_ms`/`max_response_chars`/`max_context_chars`；解决 `max_prompt_chars=4000` 与 `max_tool_result_chars=20000` 的计量口径；同一 session 最多一个活跃 run；定义取消检查点。
- **R4 Citation/EvidenceRef 统一**：PLATFORM:121 的 EvidenceRef（kind 含 `experiment/report`，必须有 `source` 与 fingerprint|sha256）与 B:117 的 Citation（kind 只有 object/action/plan/evidence/contract，`digest` 可选）是两套结构，而 PLATFORM:122 声称 B/C/D 共用。要求统一（Citation 内嵌 EvidenceRef 或补齐字段），并在 SCHEMAS 登记 `poie.agent-run/0.1`、`poie.agent-eval/0.1`、`poie.agent-audit/0.1`。
- **R5 Evals 可执行性**：45 例/覆盖矩阵有意向，但 fixture 缺 `schema`、`preregistered_at`、审批脚本（approve/reject/expiry）、final/citation 断言；“真实工具或固定 fixture executor”二选一会让工具契约用例失真；注入指标应拆成“运行时 containment（确定性必过）”与“模型是否被说服（仅现场）”。要求 fixture schema + 覆盖矩阵表 + 计数检查进 CI。
- **R6 UI/legacy 路径**：`App.tsx:1021` 仍走 `inference_complete`；B 要删掉自由聊天入口，或把它限制为 Settings 连接测试（Settings 已用 `inference_test`，`inference_complete` 无其他消费者）。同时定义 `agent://state` 负载（全量/增量）、重连/轮询、审批卡字段（args/impact/citations/subject）、过期/取消，以及 Playwright 用例。
- **R7 依赖/流程门禁**：PLATFORM:165 的依赖表缺 `toml`（fixture 解析）与 `poie-ontology`（`plan.propose` 的 Model/Goal）。要求补依赖表/ADR（编号）、根 members、两处 Cargo.lock、test-matrix 行与 status-snapshot、desktop workflow 的 `crates/poie-governance/**` 路径、`evals` CI 步骤。`SCHEMAS:3/45` 声称的 `scripts/check-schema-registry.py` 实际不存在（`scripts/` 无此文件），A0 遗留的登记门禁必须补上或改文档。
- **R8 范围/文档漂移**：B:315 的 `execution_gate_set` 旧句应删；PLATFORM:68 明确 agent 不是独立窗口，B:315 的“Agent 窗口”改为“main 窗口 + 命令 scope”。B:101 的“B2 待审批派发请求”超出 B 的退出条件（README 要求模型工具中 dispatch=0），建议删除。B:99/279 一边说 `sim.run` B1 不启用，一边把它算进 45 例的“五工具”；B1 不应向模型暴露 `sim.run`。
- **R9 capability 生命周期**：B:159-167 要求单次、≤300s、不得超会话、登出/接管/降级撤销；但未说签发时机（应在审批后）、与 `Session.expires_at` 取 min、撤销事件、128-bit 随机来源（desktop 已有 getrandom）。补实现约束与测试。

## 4. OPTIONAL

- O1 设置页“最近 eval 结果”无数据源：由 CI 生成 `evals/agent/REPORT.json` 并随包，或删掉该 UI 项。
- O2 `policyVersion` 常量、`runId` 生成、`asOf` 语义、工具结果截断算法（保持 JSON 可解析、citation 不丢）。
- O3 `ToolResult`/`RunState` 的 schema 快照测试；描述性 JSON Schema 与 serde struct 的一致性测试（避免模型提示与校验器漂移）。
- O4 desktop CI 加 `--locked`；audit UI 按 run 过滤；审计文件 0600/保留策略。
- O5 单 run 队列/并发上限（若 R3 未覆盖）。

## 5. 通过项（不需要架构重写）

- 无 dispatch 工具：B:99-103 明确六工具、封闭目录、`sim.run` 不伪造；符合 README 退出条件。
- 权限/审批分类/scope 的骨架正确；DIGEST 的 `poie.approval/0.1` 与 per-tool 字段方向正确（需按 B3/B5 收紧）。
- T0-T3 与“工具输出不进指令位”的意图正确；Rule of Two 的 profile 概念正确（需 R2 落地）。
- 确定性 eval 与现场 pass^k 分离（B:270-271）是诚实的；45 例/先提交 fixture 的纪律正确。
- RunState 持久化 + 宿主重启标 `interrupted`、不自动恢复模型调用（B:240-243）避免幽灵运行。
- 审计“只追加 + SQLite 仅索引 + 不宣称防篡改/不伪造远程时间戳”（B:318）方向正确；补格式即可。
- 复用 A 的 `poie-objects`/`poie-digest`/SDK 与现有 UI inspector，避免第二套本体；无 Tauri/网络依赖的边界正确。
- 威胁模型表（B:299-311）与测试清单覆盖面好；验收 #1/#3/#5 可直接转成测试（#2/#4 需 B6/R5）。

## 6. 进入 B 前的勾选

- [ ] B1–B7 关闭；B0 四前置（session id/approval 表/audit 格式/gate 鉴权）有实现与测试
- [ ] `agent_*` 的 CommandError/错误码、run 状态机、审批状态机、原子消费
- [ ] 六工具逐项契约 + 只读 SAP preview（无 Authorization）
- [ ] 审计迁移/链/锚/重建；历史行 legacy 处理
- [ ] 45 例 + 覆盖矩阵 + CI 计数；live pass^k 独立
- [ ] UI 审批/取消/legacy 移除 + Playwright
- [ ] members/locks/test-matrix/status/cargo-deny/ADR/check-docs
- [ ] `execution_gate_set` 旧句与“Agent 窗口”旧句修正

## 7. 方法与边界

- 只读了指定文档与源码，未跑全量测试；未验证真实 provider、Tauri 运行时与 SQLite 并发。
- 结论只对 `892d018` 负责；A0/A1 的既有漂移（checker 缺失等）只在影响 B 时列为 R7，不作为 B 的架构否决。
- 本报告不要求重写 Phase B；七项阻断均为契约/状态机/迁移层面的局部修订。

VERDICT: CHANGES_REQUIRED
