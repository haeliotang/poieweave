# Poieweave Phase A–D 设计独立校验（最终）

- 校验对象：[README](README.zh-CN.md)、[Phase A](phase-a-ontology-product.zh-CN.md)、[Phase B](phase-b-agent-governance.zh-CN.md)、[Phase C](phase-c-what-if.zh-CN.md)、[Phase D](phase-d-site-delivery.zh-CN.md)、[Digest 规范](DIGEST.md)，以及 [Palantir](research/palantir-patterns.zh-CN.md)、[Agent 治理](research/agent-governance.zh-CN.md)、[差距审计](research/repo-gap-audit.zh-CN.md) 三份备忘。
- 代码基线：`bee82b7`（`main`）；`git status` 显示 `?? docs/design/`，全部设计/校验文档尚未入库。
- 文档快照：2026-11-04 22:56（本地）。校验期间文档被多轮并发改写（22:40–22:56），行号以本快照为准；结论必须先冻结 commit 才可引用。
- 方法：逐条对照 PO/SAP/VEC、`crates/poie-*`、`apps/desktop/src-tauri`、`apps/desktop/src`、根 workspace、CI、`scripts/` 与 ADR；`python3 scripts/check-docs.py` 通过。未运行完整 `cargo test`，未修改应用源码、设计文档或 git 状态，仅重写本文件。

## 0. 总判定

**PASS WITH REQUIRED CHANGES。** 四个阶段的方向正确，且本轮修订解决了上一版最明显的三个问题：A 删除了无模型来源的治理字段并增加了 A0 宿主鉴权前置；A/C/D 有了共享的 [Digest 规范](DIGEST.md)，区分 fingerprint 与 SHA-256；A 的预览改为 host 侧只读 `SapPreview`，不再由 `poie-objects` 伪造授权。没有发现必须推翻的架构错误。

但当前仍不能直接开工。剩余问题集中在“规范已写、实现契约未闭合”：`OntologyCatalog` 仍没有 `catalog_digest` 字段；A 的 `SapPreview`/`ActionPreview` 缺类型与错误包络；B 的六工具/审批暂停/审计权威仍不可实现；C 的 Snapshot/checkpoint/verdict/promote 仍缺字段；D 的模型导入、digest 来源、host_info 与 tag gate 仍无落点。按 README 的设计门禁，应先完成 §3 的 REQUIRED 修改并重新校验。

## 1. 逐文档校验

### 1.1 `README.zh-CN.md`

**成立之处**

- 跨阶段不变式（SAP 唯一执行语义、正式契约唯一真源、模拟隔离、证据绑定、默认保守、可重复）与 PO/SAP/VEC 一致，并给了可测门禁。
- 新增 Digest 交叉规范入口（README:43-45），并把 A0/A1 的安全前置写入“已知修订”（README:41），比上一版更可执行。
- 主动列出实施前必须清理的漂移（README:84-96），把 384/388、DMG、tag gate 等旧声明纳入范围。

**缺陷**

1. **顺序与依赖自相矛盾。** README:14 要求“严格 A → B → C → D”，但差距审计建议 `A1→A2→B1→C1→B2→C2→D1→D2` 且允许 B1/C1 并行（repo-gap-audit:18）。C 的 promote 依赖 B 的审批（C:131-132、C:235），却又排在 B 后；应发布“阶段切片 DAG”，说明可重叠切片与不可跳过的安全前置。
2. **P2 “无 CRUD/写接口”过宽。** README:66 与 Phase D 的 `delivery_channel_set`、`delivery_policy_set`、`bundle_import`（D:159-167）冲突。应限定为“不得有绕过 SAP 的领域对象/执行写接口；配置/交付状态写入必须审计并受 auth/scope 约束”。
3. **P5 “promote 只产生 proposed”**（README:69）仍与现有 SAP 预览行为不符：`validate_plan_through_sap` 会把动作推进到 `authorized` 并写入 `Authorization`（[sap.rs](../../crates/poie-plan/src/sap.rs):77-82、:126-146）。必须由 C 的新 proposed-only API 解决，不能只在文档里重复承诺。
4. **A 的退出条件“外部消费者仅凭 Catalog/SDK”**（README:79）超出 MVP 的 Tauri IPC 边界（A:315 自认仅 IPC）；应改为“桌面内消费者”，外部服务化另立阶段。
5. **门禁产物未入库且 README 预设 PASS。** README:30 要求最终报告末行为 `VERDICT: PASS`，但 README:101-102 仍写“最终校验待执行”；整个 `docs/design/` 未跟踪。本报告只能给出 `CHANGES_REQUIRED`，因此在 R1 完成前门禁不成立。

### 1.2 `phase-a-ontology-product.zh-CN.md` 与 `DIGEST.md`

**成立之处**

- A0 把 `execution_gate_set`、`sandbox_read/write` 的鉴权/scope 前置写成开发第一步（A:36-45），并明确“没有它 Phase A 不开始”；这是本轮最重要的修正。
- `lot` 已明确“MVP 不暴露未进入 `Lot` 结构的 `location/evidence`”（A:67），与运行时 `Lot` 只有 `id/resource/si/quality`（[model.rs](../../crates/poie-ontology/src/model.rs):290-295）一致；不再要求投影不存在的字段。
- `ActionType` 只保留模型可导出字段，治理字段由 Phase B 按 ID 关联（A:100-108）；`SapPreview` 由 host 生成并显式标注假定项（A:186-200），`poie-objects` 不依赖 plan、不伪造授权（A:160-163）。这是正确的边界。
- Digest 规范给出 SHA-256 + canonical JSON + domain 分隔 + fingerprint/SHA-256 区分 + manifest 排除 integrity（DIGEST:10-50、:86-90），足以作为 A/C/D 的共同规范。

**缺陷**

1. **【阻塞】`OntologyCatalog` 仍没有 `catalog_digest` 字段。** A:170-177 只有 `model_digest`，但 A:179-184 的 `ObjectPage.catalog_digest`、A:218 的“返回 catalogDigest”以及 DIGEST:21 的 `poie.catalog/0.1` 都要求它。消费方只能自行哈希整个响应，无法验证“同一个 Catalog”。必须给 `OntologyCatalog` 加 `catalog_digest`（按 DIGEST:21 的 payload 不含 digest 字段），并定义 host 返回顺序/排除规则。
2. **【阻塞】Digest 的模型 canonical 与 validated `Model` 字段不完全对应。** DIGEST:71-72 要求资产含 `description`，但运行时 `Asset` 没有该字段（model.rs:266-288）；`Model.doc` 才有。DIGEST:64 又说“只从已验证的 `Model` 字段构建”。必须二选一：把 `description` 加入 `Asset`（改 PO 实现/版本），或在 canonical 中删除并说明；否则实现者只能读 `Model.doc`，绕过验证。`DIGEST:70` 的 resource 字段同样要核对 `Resource` 是否有 `description`。
3. **【阻塞】`SapPreview`/`ActionPreview` 仍缺类型与错误契约。** `CheckPreview`、`Predicate`、`ObjectType`、`ObjectInstance`、`LinkType`、`LinkInstance`、`ObjectDetail`、`FunctionType` 均无字段定义；A:220 只有四个错误码，缺 `invalid_cursor`、`limit_exceeded`、`model_mismatch`、`unauthorized`、`preview_unavailable`。host 命令返回裸 `Result<T,String>`，UI 无法稳定分支；必须定义 `{code,message,details}` 包络和每条命令的 JSON 快照。
4. **【阻塞】`can_submit` 仍没有 submit 命令。** A:198 说 `can_submit` “仅表示可进入 proposed”，A:250 说“提交只进入 SAP proposed/validated”，但 A:205-213 的命令表没有 submit；A 只有 preview。应增加明确的 `ontology_action_submit`（只写 proposed，不写 authorized），或删除 `can_submit`，否则该字段永远无法验证。
5. **【阻塞】`plan.preview_sap` 的输入仍不可实现。** A:121 只接收“plan digest, assumed context”，但 `validate_plan_through_sap` 需要 `&Plan` 与非空 `state_version`（sap.rs:88-95）。没有 plan registry/存储，无法从 digest 还原 Plan；应改为接收 Plan 对象，或把该函数移到 host 并明确不进入 SDK。
6. **【阻塞】A0 未进入实施顺序，且移除授权会破坏现有执行视图。** A:302-309 的步骤从 `poie-objects` 开始，没有 A0；A0 只是正文第 4 节。A:251 要求移除 `execution.rs` 的 `Authorization{by:"maintainer", basis:"issue #105..."}`（[execution.rs](../../apps/desktop/src-tauri/src/execution.rs):96-101），但该文件同时硬编码 `policy_version/state_version/predicates`，测试还断言 `review_status=="ready"`、0 rejections、全部 `authorized`（execution.rs:181-190）。移除授权后 `review_dispatch` 必然报 `missing_authorization`，`dispatch_allowed=false`。必须定义替代授权来源并同步改测试；此外 `federation_view.rs` 还有第二处 `Authorization{by:"maintainer", basis:"preview"}`（:85-89），A0/A 未列入清理。
7. **【阻塞】SDK 生成路径仍自相矛盾。** 架构图写 `scripts/check-ontology-sdk.py`（A:139），§7.3 写 `cargo run -p poie-sdkgen -- --check`（A:230）；`scripts/` 中不存在前者。还缺：由哪个 bin 产出输入 Catalog JSON、生成器依赖（ts-rs/Schemars/自写，需 ADR 与 `cargo-deny`）、黄金文件、失败退出码，以及 `poie-sdkgen` 是否计入 `check-test-matrix.py` 的 `poie-*` 包计数。
8. **【需修改】对象图/Playwright 的可行路径未定义。** A:244 要求 `CapabilityGraph` 节点来自 Catalog，但现有组件用硬编码 `GRAPH_NODES`（含 x/y）与索引边（[App.tsx](../../apps/desktop/src/App.tsx):80-99、:532-536）；没有布局/边派生规则。Playwright 只跑浏览器预览（[playwright.config.ts](../../apps/desktop/playwright.config.ts):4-6），不能调用 Tauri host；A:290 的对象详情/动作预览测试需要 mock Catalog 或 host E2E，当前未定义。
9. **【需修改】“外部开发者通过 JSON API”仍是过度声明。** A:53 与 A:296 超过 A:315 的 IPC 边界；应改文案或明确未来阶段。
10. **【需修改】新 crate/CI/测试矩阵未落到步骤。** `poie-objects`、`poie-sdkgen`、`poie-digest` 必须加入根 `members`（[Cargo.toml](../../Cargo.toml):3-4），根/桌面 `Cargo.lock` 要更新；`check-test-matrix.py` 会枚举所有 `poie-*` 并校验 [test-matrix](../status/test-matrix.zh-CN.md) 与 [status-snapshot](../status/status-snapshot.zh-CN.md) 计数（当前 372+16=388）。A:309 只在最后一步提“测试矩阵”，没有把其列为每个 PR 的硬门禁；新增 `sha2`/JSON Schema/codegen 依赖也需 ADR。
11. **【需修改】Digest 规范仍有边界缺口。** canonical JSON 的数字只写“用 `serde_json` 稳定序列化”（DIGEST:58），没有规定 `-0.0`、指数/精度、跨版本行为；数组“集合语义按 ID 排序”（DIGEST:56）对 `requires_any` 这类字符串数组没有统一规则；`poie.evidence/0.1`（DIGEST:47）的 evidence registry 尚不存在。A1 前应补固定测试向量（含浮点、负零、重复字符串、键序）。

**测试缺口**

- 缺 Catalog 逐字段黄金 fixture（属性、单位、端点类型、排序），只测计数不足以证明“不是第二套本体”。
- 缺 `catalog_digest` 的规范化/跨版本/排除自身字段测试、cursor/limit/未知 schema 的 host 边界测试、`model_mismatch`、`asOf/declared` 属性测试、`can_submit=false` 测试。
- 缺“对象图节点集合等于 Catalog 子集”的断言；否则 `GRAPH_NODES` 可以继续存在。
- 缺浏览器预览 mock Catalog 的 smoke 与 host JSON 快照。

### 1.3 `phase-b-agent-governance.zh-CN.md`

**成立之处**

- 工具目录封闭、六工具均有 effect/approval 两列（B:92-101），并明确没有 `dispatch.*`/`shell.*`/`http.*`/`secrets.*`；本轮已删除 §9 的 `dispatch.request`，与 README 不变式 5 一致。
- capability 已改为不透明随机 id + 宿主内存映射（B:154-163），不再虚构签名；单次使用、≤300s、登出/接管撤销的方向正确。
- 正确识别两个既有缺口：`execution_gate_set` 未登录可改 gate，Tauri 自定义命令被所有窗口共享（B:275）；并要求先修。
- 审计迁移、Rule of Two、pass^k、60 个 fixture、注入 canary、预算/取消/引用缺失等测试项已列出（B:228-239、B:281-301）。

**缺陷**

1. **【阻塞】六工具/60 用例的时间线不成立。** 六工具包含 Phase C 才启用的 `sim.run`（B:99），但 B 的 MVP 验收要求 60 个用例覆盖六个工具（B:239）且 B 先于 C。B1 应只测五个工具，`sim.run` 返回 `not_implemented` 或标为 B2/C 启用；60 例应拆成 B1 覆盖矩阵（工具×场景×威胁×预算）。
2. **【阻塞】审批暂停/恢复没有 host API。** 生命周期有 `waiting_approval -> running`（B:189-192），但只提到 `agent_run`（B:276/307），没有 `agent_approve`/`agent_reject`/`agent_cancel`/`agent_status`/`agent_resume`，也没有 run_id、pending 查询、审批过期/取消路径。Tauri 单次 command 不能“暂停后继续”；必须定义持久化 `RunState` + 事件回传，或明确重启即取消并给出 UI 语义。
3. **【阻塞】运行状态存储三处矛盾。** B:200 说“持久化 pending 状态并暂停”，B:204 说“只保存在宿主内存 + 审计/可选 SQLite”，B:315 说“MVP 运行存内存，重启即取消”。三者不能同时成立；必须选一种并写清恢复/审计语义。
4. **【阻塞】`approval_digest` 不可实现。** B:168 的 `H(actionId || target || command || validFrom || validUntil || policyVersion || planDigest || argsSha256)` 没有算法/编码/字段顺序；`plan.propose` 没有 `actionId/command/validFrom/validUntil`，`action.draft` 没有 `planDigest`。必须按工具定义 subject schema（哪些字段必填、缺失时如何规范化为 `null`）、哈希算法/域分隔，并定义 `ApprovalToken` 类型（绑定 run/call/session/expiry/一次性）；DIGEST:46 的 `poie.approval/0.1` 可复用，但 B 必须显式引用。
5. **【阻塞】capability 的 `sessionId` 在现有会话里不存在。** `auth::Session` 只有 `display_name` 与 `expires_at_ms`（[auth.rs](../../apps/desktop/src-tauri/src/auth.rs):23-26、:141-150），没有 session id/epoch；B:157 的 `sessionId` 无法签发/撤销。应给 Session 加不可猜测 id 与 auth epoch，并定义登出/接管/降级时 capability 的撤销路径；B:196 还写“校验会话/签名”，但 capability 已无签名，词表需同步。
6. **【阻塞】审计与 SAP 记录没有落点。** B:278 说“权威事件仍落 SAP 记录”，但 SAP 只有九类 `RecordKind`（[record.rs](../../crates/poie-state/src/record.rs):125-135），没有 tool/approval/agent-run 类型；只读工具调用没有 SAP 动作。必须选择：新增记录类型（SAP v0.3 契约变更 + ADR + SAP-T 用例），或明确 Agent 审计以 append-only log/SQLite 哈希链为权威并降低“SAP 记录”声明。另需 `PRAGMA user_version` 迁移，不能继续启动时 `DROP TABLE IF EXISTS events/notes`（[db.rs](../../apps/desktop/src-tauri/src/db.rs):58-78）。
7. **【阻塞】`Authorization` 没有承载审批摘要/policy 的字段。** `Authorization` 只有 `by/basis/scope/valid_until`（[action.rs](../../crates/poie-state/src/action.rs):213-218），`policy_version` 在 `Action` 上，不在 `Authorization`。B 说“Agent 来源写入 `Authorization.basis`、策略版本写入 `policy_version`”，但审批摘要放哪里没有定义；`scope` 当前是 `"dispatch"` 自由字符串。应定义字段扩展或明确的编码规则，否则审批无法从 SAP 记录重建。
8. **【需修改】`pass^5` 用在 ScriptedProvider 上没有信息量。** B:226 明确 CI 只跑确定性脚本，B:230 又要 pass^5；确定性脚本重复 5 次必然相同。pass^k 应只用于真实 provider 的现场评测（B:241-243），CI 报告确定性用例数/覆盖矩阵；否则指标是装饰。
9. **【需修改】Tool Catalog 与 Phase A Function Catalog 是两套。** A:116-124 定义 `query.service_viability/query.external_dependencies/plan.propose/plan.preview_sap`；B:92-99 定义 `catalog.describe/state.query/evidence.get/plan.propose/action.draft/sim.run`，只有 `plan.propose` 重合，且没有映射表。B:279 声称“工具 schema 来自 A Catalog”，实现上会另造静态注册表；应把 ToolType 定义为 Catalog 的投影，或合并命名并给出迁移。
10. **【需修改】旧自由聊天路径绕过治理。** B:276 保留 `inference_complete` 不变；现有 Inspector 仍直接调用它（[App.tsx](../../apps/desktop/src/App.tsx):981-987），不经过工具/引用/审批。必须把旧入口标为 legacy 或让它走同一策略，否则验收 5“不降级为自由聊天”只覆盖新命令。
11. **【需修改】JSON Schema/提示/HTTP 限制未申报。** B:101 要求 JSON Schema 2020-12、`additionalProperties:false`，仓库没有 `jsonschema` 依赖，手写完整 2020-12 不现实；按 CONTRIBUTING 需 ADR + `cargo-deny`。`max_prompt_chars=8000`（B:130）与现有 4000 上限冲突（[inference.rs](../../apps/desktop/src-tauri/src/inference.rs):135）；`http.rs` 只有 30s 超时且无响应体上限，多步调用会撞上。
12. **【需修改】命令/窗口 scope 只有原则，没有矩阵。** B:275 要求按命令/窗口检查 scope，但没有列出每个命令的 read/write/窗口/审批属性，也没有说明用 Tauri capability 还是手工 window-label 检查；`capabilities/default.json` 只有 `core:default`，所有自定义命令仍全局可达。
13. **【需修改】`two_person` 与 Phase D 冲突。** B:147 把双人审批留给 D，D:253 明确 MVP 单操作员；应删除预留或定义角色模型，不能同时承诺。
14. **【需修改】ToolResult/Citation 缺关键字段。** B:107-112 的 ToolResult 只有 citations/redactions/truncated/untrusted，没有 `asOf`/`trust`/`evidenceRefs`；Citation 的 `id` 没有语法、`digest` 可选。B:117 的“引用 ID 必须精确存在”无法在对象 ID 语法未定义时验证；应复用 A 的 canonical object ID 与 DIGEST 的 evidence domain。

**测试缺口**

- 缺 capability 签发/过期/参数变更/跨会话复用测试；缺 approval digest 字段逐一变更后的 `approval_mismatch` 测试；缺暂停→重启→恢复/取消测试；缺 gate/sandbox/inference 的窗口/scope 拒绝测试。
- 缺 provider 超大响应/非 JSON/超时/畸形 `tool_calls`/输出截断测试；缺 audit 哈希链、每日锚定、SAP 关联、DB 迁移测试。
- 缺 60 例的覆盖矩阵与 fixture schema 版本；`pass^5` 与确定性 CI 的关系未定义。

### 1.4 `phase-c-what-if.zh-CN.md`

**成立之处**

- 正确把 `Patch`（模型层）与 `Perturbation`（时间/储能/目标）分开（C:66-85），并承认 `poie-sim` 没有 checkpoint（C:89）；`storage_discharge_w` 确实在 `TimedScenario` 上而非 `Patch` 上（[timed.rs](../../crates/poie-sim/src/timed.rs):225-235），这个判断准确。
- 新增 `assetDelayS`（C:78、C:91），把“泵延迟修复”从无字段变成有明确承载；并规定 timed 未实现时 UI 标为 `structure/plan only`，不能显示时间结论。这是本轮有效修正。
- 正确引用 `Namespace::Simulation` 与 `ensure_live_dispatch`（C:137；[record.rs](../../crates/poie-state/src/record.rs):83-104、[isolation.rs](../../crates/poie-state/src/isolation.rs):20-29），并声明 UI 的 `Sim` 不是安全边界。
- 固定 `now_s`、拒绝 stale、不自动 rebase、比较必须列出全部差异、`SIMULATION` 双编码，都是正确方向（C:106、C:139、C:142、C:179）。

**缺陷**

1. **【阻塞】Snapshot 字段不足以重放。** C:53-60 只有 assetStatus/lots/storageLimits/evidenceVersion；缺 `nowS/seed/horizonS/maxDepth/maxActions/StateId/assetTimelines/stockTimelines/evidenceValidity`，以及 storage 的 capacity/initial SoC。`TimedScenario` 需要这些（timed.rs:225-235）；`assetDelayS` 需要构造 `AssetTimeline`，但 Snapshot 没有时间线。缺了无法从 snapshot 重放 `TimedRunner` 的结果；`StateId` 虽存在但不可序列化（[projection.rs](../../crates/poie-state/src/projection.rs):60-66）。
2. **【阻塞】桌面没有“live 状态”，stale 检查是空的。** `domain_overview` 返回启动时计算的静态 `DomainOverview`（[lib.rs](../../apps/desktop/src-tauri/src/lib.rs):175-177、:241-252），`execution`/`federation` 也在启动时算好；没有可变模型/证据版本。C:139 的“live 模型/证据/StateId 改变则拒绝”永远不成立，“模拟前后 live 不变”的验收（C:201）也是平凡测试。必须定义 live 状态源（当前只是 Base-01 demo），或把 C 限定为“静态 demo snapshot 上的 what-if”并删除强声明。
3. **【阻塞】`PatchEnvelope` 复制 `Patch` 且不可序列化。** C:71-75 与 `Patch`（model.rs:813-846）逐项重复，但 `Patch`/`AssetStatus`/`LotQuality`/`Verification` 只有 `Deserialize`（model.rs:33-58）。C 需要持久化/返回 snapshot/variant/digest，必须给领域类型加 `Serialize` 或定义 DTO 并做双向转换，并明确 `PatchEnvelope -> Patch` 的唯一映射；当前还漏了 `assetDegraded`/`assetCapabilityVerification`。
4. **【阻塞】verdict 词表冲突且缺映射。** C:98 把 `candidate` 与 `too_late/power_limited` 放在一个枚举，C:142 才说归一化为 `VerdictKind`；但 `VerdictKind` 已是 VEC 的六类结论名（[verdict.rs](../../crates/poie-viability/src/verdict.rs):11-20），结构结论是 `candidate/blocked/unknown/search_limit`（[graph.rs](../../crates/poie-ontology/src/graph.rs):40-45），时序结论是 `feasible/blocked/.../power_limited`（timed.rs:159-168）。必须定义独立类型（如 `OutcomeKind { structural, timed }`）与 `candidate <-> feasible` 映射、每种 mode 的结论域；否则 UI 会把结构 candidate 误读成时序可行。
5. **【阻塞】`sapPreview {reviewStatus, rejections[], checks[]}` 当前 API 不提供。** `validate_plan_through_sap` 只返回 `authorized_actions/review_checks` 计数或错误字符串（sap.rs:21-35、:146）；`review_dispatch` 虽返回拒绝项，但被封装在错误里。必须新增包含每项检查/拒绝详情的 preview 结果类型（可复用 A 的 `SapPreview`），否则 C:101 和 A/B 的“逐项原因”都无法实现。
6. **【阻塞】promote 说“只创建 proposed”，实际只有“授权”API。** C:131/C:204 要求 promote 创建 proposed actions，但现有 `validate_plan_through_sap` 直接到 `authorized`（sap.rs:126-146）；`Action` 也没有仿真分支/模型版本字段（[action.rs](../../crates/poie-state/src/action.rs):222-242），无法满足 SAP §7.4“仿真结果生成的现场动作必须引用仿真分支与模型版本”。必须新增 proposed-only API，并决定如何携带 `simulation_branch/model_version`（字段扩展或 ADR）。
7. **【阻塞】timed/storage 构造与 checkpoint 不足。** `TimedScenario::from_parts` 把 `storage_discharge_w` 置空且无 setter（timed.rs:349-362），直接写 pub 字段会绕过单位/非负校验；`TimedRunner::run` 只接受 `CaseDoc`（timed.rs:404），C 的任意 goal/deadline 没有入口。C:89 提议的 `Checkpoint {time_s, assets, stocks, batteries, external_flows, digest}` 缺 `initial_stocks`、`external_in_kg/out_kg/into_assets_kg`、`rules/fired_rules`、`queue/next_seq`、`log/violations` 与 model digest；按其字段恢复会破坏质量核算/重放。必须定义完整 checkpoint 或明确只做固定起点重跑。
8. **【阻塞】隔离声明不可执行。** `ActionLedger` 没有 namespace 字段；`Namespace` 只在 `Envelope`/记录层（record.rs:83-104），`ensure_live_dispatch` 只在派发时检查（isolation.rs:20-29），不约束 variant 的 plan/ledger/记录写入。若 variant 根本不写 SAP 记录，C:137“所有变体记录使用 Simulation namespace”是空话；若要写，需要记录级 API 与投影排除测试。必须二选一并补测试。
9. **【阻塞】依赖漏了 `poie-state` 与 `poie-digest`。** C:110-116 的 `poie-whatif` 依赖树没有 `poie-state`，但 C 要用 `Namespace`、`StateId`、`isolation`；也没有 `poie-digest`，但 C:106 的 digest 必须走共享规范。必须加入并说明是否允许 `poie-whatif` 依赖 state（当前依赖方向 `state -> ontology`，不会成环）。
10. **【需修改】预算/比较/持久化/审批 token 未定义。** `budget_exceeded`（C:157）没有具体数值；`CompareResult` 排序键/升降序/并列规则未定义；C:232 说结果存内存、定义/摘要存 DB，但 `db.rs` 没有 snapshot/variant/run 表与迁移；`whatif_variant_promote(variantId, approval)` 的 `approval` 若是布尔值就是审批绕过，必须定义绑定 snapshot/variant/plan digest 的一次性 token。
11. **【需修改】`Mode::Sim` 在 host 不存在。** 前端只有 TS 联合类型与 `MODE_ENABLED.sim=false`（App.tsx:51、:69），没有 Rust `Mode`；“Sim 下所有写操作被禁止”在 host 没有对应状态，`execution_gate_set` 仍可调用（lib.rs:184-205）。需要 host 侧模式或明确只约束 `whatif_*`。
12. **【需修改】`GoalDoc` 只 Deserialize。** `poie-ontology::scenario::GoalDoc` 是 `Deserialize`（[scenario.rs](../../crates/poie-ontology/src/scenario.rs):29-37），C 的 API 返回/持久化 goal 需要 Serialize 或独立 DTO；`Verification` 同样。
13. **【需修改】`assetDelayS` 的语义仍不完整。** C:78/91 说“整体推迟”，但没有定义它作用于资产可用时间、修复完成时间还是时间线起点；`TimedScenario` 的 `asset_timelines` 是状态变更序列，偏移应作用于哪个字段、是否影响库存时间线，均未定义。应给出转换公式与测试（含负值/超过 horizon）。

**测试缺口**

- 缺 snapshot 全字段 round-trip、StateId stale 检测、per-mode verdict 域、timed 从 snapshot 重放、storage 上下界、compare 排序稳定性、promotion token、`proposed` 状态断言。
- “live 不变”没有可变 live 可比；需要可注入的 live 状态，或明确删除该测试，否则验收 2 是假阳性。
- 缺并发/取消/超时、DB 迁移、审计、namespace 泄漏测试。

### 1.5 `phase-d-site-delivery.zh-CN.md`

**成立之处**

- Manifest 覆盖 app/core/contracts/model/scenarios/evidence/policy/artifacts/SBOM/files/integrity（D:54-77），方向完整；并明确 SHA-256 只做完整性、不冒充签名（D:85）。
- 通道是“订阅”而非目标版本、DeliveryPlan/constraints/blockedReasons、recall/rollback（D:91-115），比“push 版本”正确。
- 诊断字段有脱敏白名单（D:119-131），默认关闭遥测/自动更新（D:107、:198），与 ADR-0001 I3/P7 一致。
- 新增“版本单一事实源”（D:173）：以 `apps/desktop/package.json` 为产品版本，core 保持自身语义版本，避免再次出现 core/desktop/tag 三套版本。

**缺陷**

1. **【阻塞】导入的模型没有消费方。** 桌面模型是编译期 `include_str!("../../../../scenarios/base-01/model.toml")`（[domain.rs](../../apps/desktop/src-tauri/src/domain.rs):12-13），`AppState` 只保存算好的 `DomainOverview`（lib.rs:37、:241-252），没有 model store/loader。D:75 的 `files = [{path="models/base-01.toml"}]` 与“携带新 bundle 导入并显式切换”（D:45）在运行时不会生效；验收 1/3（离线导入可启动、模型不匹配拒绝）不可验证。必须新增模型/证据存储、解析优先级（内置 fallback vs bundle）与启动加载，或把 D 明确降级为“元数据/策略交付”，不宣称模型切换。
2. **【阻塞】所有 digest 来源未定义。** D 需要 `model.digest`、`contractsDigest`、`evidence.digest`、`executionDigest`、`federationDigest`、`manifest_sha256`；代码没有 model digest，`ExecutionSnapshot`/`FederationSnapshot` 没有 digest（[execution.rs](../../apps/desktop/src-tauri/src/execution.rs):34-43、[federation_view.rs](../../apps/desktop/src-tauri/src/federation_view.rs):35-45），contracts 版本只存在于契约首行散文（[PO](../../specifications/physical-ontology.zh-CN.md):3），没有机器可读 registry。DIGEST 已给出 domain，但 D 必须补 `contracts.toml`/registry 与“从当前运行计算”的 API。
3. **【阻塞】`manifest_sha256` 自引用需在 D 内显式引用 DIGEST。** DIGEST:89 已定义 `canonical_json(manifest_without_integrity)`，但 D:76 仍只写 `integrity.manifest_sha256`，没有说明排除规则/字段名；单独读 D 会得到循环定义。D 必须引用 DIGEST:86-90，并定义 `manifest_without_integrity` 的精确 schema（是否排除整个 integrity 对象、是否包含 `files/artifacts`），再加固定测试向量。
4. **【阻塞】`host_info` 的编译期 `channel`/`manifestDigest` 不可实现。** D:169 要求从编译期 `release.json` 读取 `releaseId`、`channel`、`manifestDigest`；但 `channel` 又存在 SQLite 且可由 `delivery_channel_set` 修改（D:103-108、:162），manifest digest 又依赖构建后 artifact（D:73）。同一字段不能既是编译期常量又是运行时可变状态；manifest digest 也不能在二进制构建时嵌入它自己覆盖的 artifact。应拆成：编译期 `releaseId/build channel`、运行时 `effective channel`（`delivery_status`）、旁路 `manifestDigest`。
5. **【阻塞】`scenarios.digests = "sha256:..."` 与现有证据格式不符。** 现有时间版 digest 是规范化字符串（如 `feasible|candidate|10800.000|`，[digests.toml](../../scenarios/base-01-timed/digests.toml):3-5），VEC 报告的 input hash 是 fnv1a 16 位（[report.toml](../../experiments/vec-base01/report.toml):17-20），不是 SHA-256；DIGEST:27 又规定输出不带 `sha256:` 前缀。必须区分 `fingerprint`（规范文本，用于复现）与 `sha256`（字节承诺，用于完整性/证据），或明确对 digest 文件本身做 SHA-256 并写清字段。
6. **【阻塞】构件清单与流水线不一致。** D:73 示例是 `Poieweave_0.1.1_universal.dmg`，但 [desktop-build.yml](../../.github/workflows/desktop-build.yml):135-148 只构建 `.app` 后打 `.app.zip`，没有 `hdiutil`/DMG；README 仍宣称 DMG/portable exe（README:27、:82，README:94 已承认是手动补的）。必须改 manifest/README，或增加 DMG 构建；否则站点按 manifest 找不到 artifact。
7. **【阻塞】现有 sandbox 不能承担导入。** `sandbox::resolve` 只拒绝非 Normal 组件，不检查符号链接；`write` 用 `std::fs::write` 跟随符号链接、没有临时目录/原子 rename/fsync，只支持 UTF-8 文本、单文件 1 MiB（[sandbox.rs](../../apps/desktop/src-tauri/src/sandbox.rs):9-48）。D:81-82 的“临时目录 + 原子发布 + 拒绝符号链接”需要全新 API 与 no-follow/同文件系统 rename/目录权限/大小上限测试，不能把现有函数当基线。
8. **【阻塞】“所有写操作要求登录”当前是假的。** `sandbox_read/write`、`domain_overview`、`execution_gate_set` 都未强制 `auth::require`（lib.rs:175-221、:184-205）；`inference_complete` 不读站点策略；`capabilities/default.json` 只有 `core:default`，所有自定义命令仍全局可达。D:209“所有 host/Agent 路径读取同一策略”没有 loader/强制点，`remote_inference_allowed=false` 也无法阻止现有命令。必须先列每命令 auth/scope/policy 矩阵并修 gate/sandbox。
9. **【阻塞】命令/错误/数据模型缺失。** D:159-167 没有 `delivery_bundle_inspect`（UI 要求导入前预览）、`delivery_recall`（§5.3 定义 recall）、`delivery_plan`（UI 要解释“为什么没有更新”）；`DeliveryPlan`、`Diagnostics` 的部分字段无来源；错误只说“稳定 reason code”，没有错误码表/包络。`recentErrors` 没有结构化错误存储（只有 audit 字符串），`executionDigest`/`federationDigest` 无函数。
10. **【阻塞】DB 迁移/回滚没有底座。** D:115 要求“数据库迁移可逆或先备份”，但 `db.rs` 没有 `user_version`、没有迁移框架，启动还会 `DROP TABLE IF EXISTS events/notes`（db.rs:58-78）。D 需要独立迁移/备份/恢复设计，否则“回滚可测”无法实现。
11. **【需修改】通道/策略/recall 语义未决。** `channel` 同时出现在 manifest（D:60）与 SQLite DeliveryState（D:95-100），“通道是订阅”与 manifest 的 `channel="stable"` 冲突；`site` 通道与站点固定策略、`app.version` 不兼容拒绝与“导入新 bundle 升级”互相矛盾；bundle 的 `policy` 与本地 SQLite 策略谁优先、导入能否放宽 agent scopes/remote inference/auto-update，均未定义。recall 在断网站点如何获知（没有网络/中心）也未定义。
12. **【需修改】发布门禁只有要求，没有落点。** [ci.yml](../../.github/workflows/ci.yml):5-7 只在 `push.branches=[main]` 与 PR 上运行，tag push 不会触发根 CI；[desktop-build.yml](../../.github/workflows/desktop-build.yml):183-186 的 release job 只 `needs: build`，PR paths 仍缺 `apps/desktop/tests/**` 与 `playwright.config.ts`（:20-31）。D:171 要求“tag release 必须依赖根 verify-all”，必须具体到 workflow trigger/needs/workflow_run，否则这条只是口号。
13. **【需修改】SBOM/attestation 未申报。** D:74、D:245 要求 SPDX JSON/SBOM/attestation，但没有工具（如 cargo-cyclonedx）、CI 权限、失败策略与依赖 ADR；`app.target` 也需构建期注入，不能运行时猜。
14. **【需修改】诊断包“扫描禁止字段”没有算法。** D:208/D:131 只列白名单，没有键名递归扫描、值熵/长度检测、负例测试；“绝对路径之外的敏感信息”表述本身也不可执行。

**测试缺口**

- 缺 manifest 自哈希（排除 integrity）、规范化、篡改任一字节、重复路径/大小写冲突/符号链接/硬链接、跨设备 rename、导入中崩溃、磁盘满/权限、bundle 超限测试。
- 缺“导入后实际使用导入模型”的测试（当前不可能）；缺 channel/manifest 冲突、recall、DeliveryPlan 解释、policy 强制、远程推理关闭、gate/sandbox 认证测试。
- 缺 diagnostics 字段逐一来源断言与 redactions 负例；缺 DB 迁移失败/回滚备份测试；缺 DMG/SBOM/attestation 与流水线一致性检查。

### 1.6 三份研究备忘

**Palantir 模式**

- 强项：严格区分文档事实与推断（:5、:10），把“可借鉴/改造/拒绝”落成 P1–P9 与阶段门禁（:143-164），并明确警告“像 Palantir 的演示、丢掉物理契约”是最大风险（:171）。这为 A–D 提供了正确的取舍框架。
- 问题：P2“无 CRUD/写接口”（:148）与 D 的配置写入冲突（同 README）；P5“promote 只产生 proposed”（:151）与现有 SAP 预览授权冲突；A 的“外部消费者仅凭 Catalog/SDK”（:91）超出 Tauri IPC；对 Palantir beta 语义的时效性已标注，但“Apollo 支持 air-gapped”等外部事实未在本仓库独立验证，只能作为方向依据，不能作为验收证据。

**Agent 治理**

- 强项：来源分级、G1–G7 不变量、威胁表、AGT-T1–T13、Rule of Two/τ-bench/PROV-O 的引用都较扎实（:71-82、:190-216）；“prompt injection 无可靠防御，只能缩小爆炸半径”（:15）应成为 Phase B 的基调。
- 问题：它与 Phase B 的决定已经分叉：备忘主张七个工具并含 `dispatch.request`（:98），Phase B/README 决定六个且模型不可见（B:92-101）；备忘的审批词表是效果类 `read_only/plan/simulate/dispatch`（:107-116），Phase B 又有 `auto/operator_confirm/two_person/forbidden`；备忘的 `poie-governance`/`poie-eval`（:83、:125）与 B 的 `poie-governance` 也未完全对齐。`Gate::dispatch`（:76）实际是 `ExecutionGate::dispatch`（[gate.rs](../../crates/poie-exec/src/gate.rs):149），属命名小错。备忘要求 ToolResult 带 `as_of/trust/evidence_refs`（:120），Phase B 的 ToolResult 只有 `citations/redactions/truncated/untrusted`（B:107-110），这是能力回退。
- 建议：把备忘中的“决定”与“提案”分开；凡 Phase B 未采纳的（RunState 序列化、T0–T3 信任级、`as_of`、七个工具）必须在 B 中显式写“未采纳/后续”，不能留下两套规范。

**仓库差距审计**

- 强项：路径/行号最具体，正确指出 `execution_gate_set` 缺 auth、`db.rs` 启动 DROP、测试矩阵计数、DMG 与 PR paths 等硬事实（:85-99、:294-300），是本次校验最有用的证据源。
- 问题：内部已陈旧/自相矛盾：:9 决定 `poie-governance`，但 :104/:113 仍写 `poie-agent` 且说“设计文档用 poie-agent”；:9 用 `auto` 词表，:116 又用 `auto_read`；:175 的 checkpoint 与 C:89 一样缺质量核算/队列字段；:177 的 `validate_plan_through_sap -> proposed` 与实际授权行为不符；:181 把 `ensure_live_dispatch` 当成写入门（它只是派发门）；:283-284 的测试计数本身已过时。备忘应标注“以冻结设计为准”，并删除/修正上述条目。

## 2. 跨阶段问题

1. **【阻塞】顺序 DAG 不一致。** README:14 的严格串行与审计 :18 的切片并行冲突；C 的 promote 依赖 B 审批（C:131-132、C:235），但 README 又把 C 放 B 后；B 的站点策略留 D（B:316），D 的 `agentScopes` 又要 B 执行（D:99、:167），形成策略依赖环。必须发布一张“切片 DAG + 安全前置”表，明确 A1/A2/B1/B2/C1/C2/D1/D2 的可并行关系与硬前置。
2. **【阻塞】Digest 规范已建立，但没有接进所有消费方。** A 的 `OntologyCatalog` 缺 `catalog_digest`（A:170-177）；C 的依赖树没有 `poie-digest`（C:110-116）；D 仍用 `sha256:` 前缀且未引用 DIGEST 的 manifest 规则（D:69、:76）；`poie-digest` 尚未进根 `members` 与测试矩阵。规范本身是进步，但“规范存在”不等于“接口闭合”。
3. **【阻塞】对象 ID/引用语法不统一。** A 用 `typeId + id`（A:210-211），B 的 fixture 用 `service:potable-water`（B:217-222），C 用 `pump-p1`（C:42），D 的 evidence 用自由 `id`（D:70）。需要统一的 canonical object ID（是否带类型前缀、大小写、转义）与 Citation/EvidenceRef 语法，否则 B 的 citation 校验、C 的 evidenceRefs、D 的 manifest 无法互相解析。
4. **【阻塞】证据引用有四种形状。** A 的 evidence 对象（kind/digest/validity/source）、B 的 `Citation{kind,id,label?,digest?}`、C 的 `evidenceRefs[]`、D 的 `evidence[{id,digest,kind}]` 没有共同定义；应先在 A 定义唯一 `EvidenceRef`（id/kind/digest/asOf/validity/source），B/C/D 复用；DIGEST:47 的 `poie.evidence/0.1` 可作哈希域，但没有类型定义。
5. **【阻塞】审批/策略类型重复且无 owner。** A 已删除 `ApprovalClass`，但 B 的 effect/approval 两列、C 的 `approval` 入参、D 的 `agentScopes/remoteInference/autoUpdate` 各自为政。应抽出一个共享 policy/approval 原语（放 B 或独立小 crate），A 只引用、C/D 复用；否则 C/D 会用布尔确认绕过 B。
6. **【阻塞】现有命令面击穿所有阶段的安全声明。** `execution_gate_set` 未认证（lib.rs:184-205）、`sandbox_write` 未认证且跟随符号链接（lib.rs:218-221、sandbox.rs:38-48）、`inference_complete` 绕过策略、所有窗口共享自定义命令（capabilities/default.json 仅 `core:default`）。B 的“操作员确认”、C 的“Sim 禁止写”、D 的“所有写操作要求登录”都依赖先修这些；A0 已把 gate/sandbox 纳入，但尚未列出 `inference_configure`/`auth_*` 的命令矩阵与 window 检查方式。
7. **【阻塞】SAP §7.4 的仿真来源无法写入 Action。** SAP §7.4 要求从仿真结果生成的现场动作引用仿真分支与模型版本；`Action` 只有 `plan_ref/policy_version` 等字段（action.rs:222-242），没有 `simulation_branch/model_version`。C 的 promote 和 B 的 Agent 提案都受影响；需字段扩展 + ADR + SAP-T 用例，或用可机器解析的编码规则。
8. **【阻塞】“proposed 只写”与现有 SAP 预览冲突贯穿 A/B/C。** 三份文档都承诺只生成 proposed/validated，但 `validate_plan_through_sap` 直接到 authorized 并伪造 Authorization（sap.rs:77-82、:126-146）。A 的 `SapPreview` 已避开了这条路径，但 C 的 promote 仍写“SAP 校验后创建 proposed”，B 的 propose 也没有 proposed-only API；必须在共享层定义 preview/propose 语义。
9. **【需修改】浏览器预览与 host-only 的测试策略缺失。** Phase A/B/C/D 的 Playwright 验收都要求 host 行为，但 Playwright 只跑浏览器预览（playwright.config.ts:4-6），现有 `domain.ts` 有 mock/fallback（domain.ts:189）。需要明确：哪些测试用 mock fixture，哪些用 Tauri host 集成测试（Rust）或真正的 E2E；否则验收是假绿。
10. **【需修改】新 crate/依赖/CI/文档门禁未进各阶段步骤。** 根 `members`/`Cargo.lock`、`check-test-matrix` 计数、`cargo-deny`、`rustdoc -D warnings`、ADR（新运行时依赖/契约变更）、`check-docs` 都只在阶段末尾/研究备忘出现；新增 `poie-objects/poie-sdkgen/poie-digest/poie-governance/poie-whatif/poie-delivery` 会立即使现有计数检查失败。每个阶段的第一个 PR 就应包含这些门禁，而不是最后补。
11. **【需修改】P2“无写接口”与 D 的配置写入。** README:66 的绝对表述应限定为“无领域对象/执行写接口”；配置/交付状态写入必须审计、受 auth/scope 约束，否则 D 的命令表与 P2 冲突。
12. **【需修改】代码注释与契约词表漂移。** `poie-plan/src/sap.rs:1-8,87` 写“six dispatch checks”，实际 `review_dispatch` 是七项（review.rs:1-12）；设计文档说七项是对的，但代码注释应同步。`poie-state/src/lib.rs:31` 仍说 M12 未实现（审计已指出），会误导 B/D 的依赖判断。

## 3. REQUIRED（开发前必须修复）与 OPTIONAL

### REQUIRED（按优先级）

| 编号 | 必须修改 | 涉及 |
| --- | --- | --- |
| R1 | 冻结设计文档到 git，记录 commit/摘要；本报告按冻结版重跑行号 | 全部 |
| R2 | 完成 Digest 接入：给 `OntologyCatalog` 加 `catalog_digest`；修正 `ModelCanonical` 与 `Asset.description` 等字段来源；定义浮点/数组/`-0.0` 规则与固定测试向量；把 `poie-digest` 加入根 members/测试矩阵/Cargo.lock/ADR | A/C/D |
| R3 | 定义 A 的完整 DTO（ObjectType/ObjectInstance/LinkType/LinkInstance/ObjectDetail/Predicate/CheckPreview/FunctionType）与统一错误包络/错误码；补 `Serialize` 或 DTO 策略 | A/B/C |
| R4 | A：新增 `ontology_action_submit`（只写 proposed）或删除 `can_submit`；把 `plan.preview_sap` 改为接收 Plan/host 预计算；把 A0 写入实施顺序第 0 步；定义 `SapPreview` 的 checks 类型/假定项 | A |
| R5 | A：单一 SDK 生成路径（`poie-sdkgen`），明确输入/输出/CI/黄金文件/失败码/依赖 ADR/测试矩阵；删除外部 JSON API 声明 | A |
| R6 | A：移除 `execution.rs`/`federation_view.rs` 的 demo 授权与硬编码 policy/state/predicates，定义操作员会话+policy/approval 替代来源并更新测试 | A |
| R7 | B：B1 只测五工具（`sim.run` 标 C/not_implemented）；定义 `agent_run` + approve/reject/cancel/status + RunState 持久化/恢复；统一 A Function/Tool Catalog | B |
| R8 | B：定义 per-tool `approval_digest`、`ApprovalToken`、capability 的 sessionId/epoch/撤销；明确审计权威（SAP 新记录类型或 append-only log）与 `user_version` 迁移、每日锚定 | B |
| R9 | C：补全 Snapshot（now/seed/horizon/maxDepth/maxActions/StateId/timelines/storage capacity+SoC/evidence validity）；定义 per-mode `OutcomeKind` 与 `candidate<->feasible` 映射；去重 PatchEnvelope；补 timed/storage API 与完整 checkpoint | C |
| R10 | C：定义 Compare/Promotion/Approval schema；新增 proposed-only SAP API；把仿真分支/模型版本写入 Action（字段扩展或 ADR）；落实 namespace/投影隔离测试；加入 `poie-state`/`poie-digest` 依赖 | C |
| R11 | D：定义 model/contracts/evidence/execution/federation digest 来源与机器可读 registry；定义 manifest 自哈希排除规则并引用 DIGEST；拆分编译期 releaseId 与运行时 channel/manifestDigest | D |
| R12 | D：新增 no-follow/临时目录/原子发布的导入 API；DB 迁移/备份/回滚；policy loader 与逐命令强制；补 bundle_inspect/recall/delivery_plan 与错误码；修正 DMG/SBOM/PR paths/tag gate | D |
| R13 | 跨阶段：发布切片 DAG；统一 object ID/EvidenceRef/policy-approval owner；修 gate/sandbox/inference 的 auth/scope；把 workspace/Cargo.lock/test-matrix/cargo-deny/ADR/rustdoc/docs 检查纳入每个阶段首个 PR | 全部 |

### OPTIONAL（可延后/可降级）

- O1 `pass^k` 只用于真实 provider 的现场评测；CI 继续用确定性用例与覆盖矩阵。
- O2 SBOM/attestation、Ed25519 签名、自动更新器延后；SHA-256 + 明确“未认证”是可接受的 MVP。
- O3 DMG/portable exe：补构建或改 README/manifest 示例，不阻塞 A–C。
- O4 对象图确定性布局算法；MVP 可用稳定布局，不必现在设计图数据库。
- O5 VEC 指标、跨站点 what-if、bundle 压缩/差分、多租户留到后续。
- O6 生成 TS 之外再生成 JSON Schema/OpenAPI，可后续做。
- O7 `two_person` 若不在 B 实现，删除预留并在 D 再定义。
- O8 capability 可先用不可伪造的内部句柄 + 参数摘要；不追求跨进程签名。
- O9 `recentErrors` 可先复用 audit 事件，但必须删掉“结构化错误”的承诺或补存储。
- O10 代码注释 “six dispatch checks” 与 `poie-state/src/lib.rs:31` 的 M12 过时描述，可随 A 的清理 PR 一起修。

## 4. “可以开始 Phase A 了吗？”检查清单

- [ ] **R1** 设计文档已提交、快照冻结；本报告基于固定 commit 重跑行号。
- [ ] **R2** `catalog_digest` 字段、`ModelCanonical` 字段来源、浮点/数组规则、固定测试向量已定；`poie-digest` 进 members/测试矩阵。
- [ ] **R3** Catalog DTO、`Serialize`/DTO 策略、错误包络、错误码、`schemaVersion` 已定。
- [ ] **R4** `ontology_action_submit` 或删除 `can_submit`；`SapPreview`/`CheckPreview`/假定项/`plan.preview_sap` 输入已定；A0 已成为实施第 0 步。
- [ ] **R5** `poie-sdkgen` 唯一生成路径、CI 检查、黄金文件、测试矩阵、`Cargo.lock`、依赖 ADR 已定。
- [ ] **R6** `execution.rs`/`federation_view.rs` 的 demo 授权、`policy_version`、`state_version`、`predicates` 的替代来源已定，相关测试更新计划已定。
- [ ] **R7** 浏览器预览 mock Catalog / host JSON 快照 / 六个旧视图不回归的测试策略已定。
- [ ] **R8** 对象图节点/边从 Catalog 派生（含确定性布局）已定；`GRAPH_NODES` 的删除/替换路径已定。
- [ ] **R9** 新 crate 进根 `members`，根/桌面 `Cargo.lock` 更新计划、`check-test-matrix` 计数、`cargo-deny` 许可结论已定。
- [ ] **R10** A 的外部消费者/JSON API 文案已改为“桌面内 SDK”，或明确另立阶段。

**结论：当前为“否”。** R1–R6 与 R8/R9 未完成前，Phase A 不应进入编码；R7/R10 以及 R11–R13 是 B/C/D 的开工前置。建议先做一个“A0 设计冻结 PR”：只改文档与 CI 门禁，不写业务代码，再重新跑本校验。

## 5. 研究遗漏与过度声明

- **过度声明 1（A）**：外部开发者通过 JSON API/TS SDK 接入（A:53、:296），但 MVP 只有 Tauri IPC（A:315）；应改为桌面内 SDK。
- **过度声明 2（D）**：断网导入并“切换模型”（D:45、D:75），但模型编译期 `include_str!`（domain.rs:12-13）且无 loader；在新增 model store 前，验收 1/3 不可验证。
- **过度声明 3（D）**：`.dmg` artifact（D:73）与现有流水线不符（desktop-build.yml:135-148 只有 `.app.zip`/NSIS/deb）；README 的 DMG/portable 文案也应同步修正。
- **过度声明 4（B）**：“权威事件仍落 SAP 记录”（B:278），但只读工具调用没有 SAP 记录类型；必须新增记录类型或降低声明。
- **遗漏 1（B）**：研究备忘的 T0–T3 信任级与 `as_of/trust/evidence_refs`（agent-governance:120、:149）没有进入 Phase B 的 ToolResult/策略；`untrusted=true` 只是标记，不是数据分类。
- **遗漏 2（C）**：VEC 明确“blocked 只相对当前快照/有界求解，不是不可行证明”；C 的 `blocked/candidate` 与 UI 没有携带该限定，可能把结构 blocked 宣传成物理不可行。
- **遗漏 3（D）**：没有安装器/DB 迁移/磁盘权限/操作员培训/支持流程等站点前置条件；D 的验收只覆盖 happy path，“现场交付”仍缺失败手册。
- **过程问题**：设计文档在本次校验期间被并发改写（mtime 22:44–22:56），且全部未跟踪；本报告只对 22:56 快照负责。在 R1 完成前，不应引用任何“已校验通过”。

VERDICT: CHANGES_REQUIRED
