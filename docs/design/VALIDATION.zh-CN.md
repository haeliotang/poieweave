# Poieweave Phase A–D 设计独立校验

- 校验对象：`docs/design/README.zh-CN.md`、Phase A/B/C/D、`DIGEST.md`、`VALIDATION-FAST.zh-CN.md` 与 `docs/design/research/` 三份研究备忘。
- 代码基线：`bee82b7`（`main`）；运行 `python3 scripts/check-docs.py` 通过（exit 0）。
- 文档状态：校验期间设计文档多轮并发修改（22:40–22:52），因此**设计断言按章节号引用，源码按行号引用**；文档目前仍未提交（`git status` 显示 `?? docs/design/`），结论无法绑定到 git 修订。
- 方法：逐条对照设计断言与 `crates/poie-*`、`apps/desktop/src-tauri`、`apps/desktop/src`、根 workspace、CI、`scripts/`、契约正文；未修改应用源码、设计文档或 git 状态，仅新增本文件。
- 结论：方向与现有 PO/SAP/VEC 契约一致，**没有发现必须推翻的架构错误**；A 已修复上一轮的大部分字段/预览问题，但仍有多项“规范已写、实现 API 未定”的阻塞点和两个过程门禁问题，当前不可直接开工。

## 0. 总判定

**PASS WITH REQUIRED CHANGES。** Phase A 的只读投影、A0 安全前置、`DIGEST.md` 的指纹/哈希分离都是实质性改进；Phase B 的治理设计也更接近可执行。但剩余问题集中在三类：**动作参数/只读 dry-run 仍无确定 API**、**审计/审批/运行恢复仍无契约级落点**、**Phase D 的模型解析与沙箱/数据库迁移仍被现有 `include_str!`/`db.rs` 架构挡住**。此外 README 的“最终校验必须 PASS”与独立校验原则冲突，必须先改。按 README 的设计门禁，完成 §3 的 REQUIRED 修改并重新校验前，不开始 Phase A 编码。

## 1. 逐文档校验

### 1.1 Phase A（Ontology 产品层）

**成立之处**

- 只读 Catalog、稳定对象/链接/动作/函数类型、生成式 TS 类型，确实能消除 `apps/desktop/src/domain.ts` 与 `App.tsx` 的手写快照漂移。
- A0 把 `execution_gate_set`/`sandbox_*` 的鉴权缺口提为 Phase A 第一步（A §4），方向正确；这比把缺口留给 B/C 更诚实。
- 动作目录改为“只含模型可导出字段”，并把 `risk_class`/`approval_class`/`budget_template` 等移到 Phase B 治理层（A §6.3），避免了在 `poie-objects` 中硬编码第二套策略。
- `DIGEST.md` + A §7.0 已正式定义 canonical JSON、SHA-256 域分隔、`model/catalog/plan/simulation/variant/manifest` 六类摘要，并明确 `Plan::digest()`/`Simulation::digest()` 是可读指纹而非哈希；这是本轮最重要的修复。
- `lot.location/evidence` 明确不进 MVP 对象属性（A §6.1），与验证后的 `Lot` 结构一致。

**缺陷与缺口**

1. **【阻塞】`parameters` 的派生规则仍未定义。** A §6.3 只说参数“从 inputs/outputs/occupies/restores/duration 派生，含单位与维度”，但没有说明：目标资产是固定还是可选、`runs`/数量/批次如何暴露、输入流是否可由调用者改变、duration 是只读还是参数、哪些字段必填。`Process` 本身没有参数结构（`crates/poie-ontology/src/model.rs:303-316`），因此实现者仍要自造映射表。必须给出确定性派生表，或从 MVP 删除“参数”并同步删除 §11 的“参数来自模型”验收项。
2. **【阻塞】只读 dry-run 仍无 API。** A §7.1/§7.2 现在把 `SapPreview` 交给 host，方向正确；但 host 没有可用的只读函数：现有 `validate_plan_through_sap` 会构造 `Authorization{by:"planning-validation"}` 并推进到 `authorized`（`crates/poie-plan/src/sap.rs:77-82`、`:139-146`）。必须新增 `preview_plan_through_sap`（不创建/不保存授权、不产生 ledger 状态）并定义 `assumedContext`、`assumed[]`、`CheckPreview` 的字段；否则 `SapPreview` 只能靠 host 伪造“7/7 已通过”。
3. **【阻塞】“当前模型”与 `DomainOverview` 可能不一致。** `domain.rs` 启动时对 Base-01 打 `pump-p1=degraded` 补丁再生成快照（`apps/desktop/src-tauri/src/domain.rs:99-104`），A 未说明 Catalog 用原始模型还是补丁后的模型，也未要求 host 持有 `Model`（当前 `AppState` 只有 `DomainOverview`，`lib.rs:37`）。若 Catalog 读原始模型，同一 UI 会显示两个 `pump-p1` 状态。必须定义唯一“当前模型”并在 host 启动时计算 digest。
4. **【需修改】属性证据的 `asOf` 没有数据源。** A §7.1 要求每个属性带 `source`/`asOf`/`verification`，但 `status_evidence` 只是字符串、`Lot.evidence` 无时间戳；必须定义 `asOf` 来自快照时间、证据记录，还是标为 `unknown`。`evidence` 对象的 `validity/source` 也没有账本（A §6.1 只返回引用）。
5. **【需修改】API 结构仍只给名字。** `ObjectType`、`ObjectInstance`、`LinkType`、`ObjectDetail`、`Predicate`、`CheckPreview` 均无字段；错误码仍只有四个，缺 `invalid_cursor`、`limit_exceeded`、`model_mismatch`、`unauthorized`、`preview_unavailable`。host 命令仍以 `Result<T,String>` 为主（如 `sandbox_read`、`execution_snapshot`），没有 `{code,message,details}` 包络。
6. **【需修改】SDK 生成路径自相矛盾。** A §7 架构图仍写 `scripts/check-ontology-sdk.py`（A:139），A §7.3 写 `cargo run -p poie-sdkgen -- --check`（A:230）；`scripts/` 中不存在前者。必须二选一，并明确输入 JSON 由哪个 bin 产生、生成物路径、失败退出码、黄金文件、`poie-sdkgen` 是否计入 `scripts/check-test-matrix.py`。
7. **【需修改】“外部开发者/JSON API”仍是过度声明。** A §5 写外部开发者通过 JSON API/TS SDK 接入，A §13 又承认仅 Tauri IPC。MVP 的 SDK 只能桌面内使用；README 的 A 退出条件“外部消费者仅凭 Catalog/SDK 完成枚举”应改为“桌面内消费者”，或明确未来服务化。
8. **【需修改】`can_submit` 仍没有提交命令。** A §7.1 增加该字段，A §7.2 说“提交只进入 SAP proposed/validated”，但命令表没有 submit，只有 preview。要么增加明确的 submit 命令（写 proposed，不写 authorized），要么删除该字段，避免永远为 `false` 的死状态。
9. **【需修改】A0 的 scope 矩阵未定义。** A §4 要求“按窗口/命令 scope”，但没有列出命令 × 窗口 × 登录 × 只读/写矩阵，也没有说明 Tauri 自定义命令如何做窗口身份识别（`capabilities/default.json` 目前只有 `core:default`）。B/C/D 会各自解释 scope，必须在 A0 冻结。

**测试/验收缺口**

- 只测“计数稳定”，没有逐字段对比 `Model`（属性、单位、端点类型、排序）与 Catalog 的黄金 fixture；没有 `preview_plan_through_sap` 不产生 `Authorized`/`Authorization` 的断言。
- 没有未知 schema 的 host 边界测试、cursor/limit 测试、`model_mismatch` 测试、属性 provenance 测试、`can_submit=false` 测试。
- “对象图节点来自 Catalog”没有测试能阻止继续使用 `GRAPH_NODES`（`apps/desktop/src/App.tsx:80`、`:532-536`）；需要断言节点集合来自 Catalog。
- `DIGEST.md` 的 canonical JSON 规则很好，但没有固定测试向量文件；`serde_json` 默认按结构体字段声明顺序输出，不会自动按 key 排序，必须由 `poie-digest` 实现并测试键序不变性。

### 1.2 Phase B（Agent 治理与评测）

**成立之处**

- 工具表把 scope/effect/approval 分开，六个工具全部为 read/propose/simulate，模型侧没有 dispatch；`dispatch.request` 从模型工具目录中删除（B §5.2）。
- capability 改为 opaque 128-bit、不新增加密库（B §5.5），比上一版“自签名”更诚实；审批仍绑定参数摘要与一次一用。
- 正确要求先修 `execution_gate_set` 的未登录改 gate 缺口，并要求窗口/命令 scope（B §9）；引用了 A0。
- 审计迁移方向正确：新增 `turn_id`/`args_sha256`/`sap_record_id`/`prev_hash`/`hash`，并明确 SQLite 不是防篡改来源（B §9）。
- 评测补上 pass^k、60 个 fixture、注入 canary、审批/越权/预算指标（B §6），方向对。

**缺陷与缺口**

1. **【阻塞】审批暂停/恢复仍无 host API。** B §5.7 写 `waiting_approval -> running`，但命令只有 `agent_run`（B §9）；没有 `agent_approve`/`agent_reject`/`agent_cancel`/`agent_status`/`agent_resume`，也没有 `run_id`、pending 查询、审批过期与取消路径。Tauri 单次 command 无法在暂停后继续；需要持久化 `RunState` + 事件回传，或明确“暂停即取消”。B §5.7 说“持久化 pending”，下一段说“只保存在宿主内存”，B §12 又说“MVP 存内存”，三处矛盾。
2. **【阻塞】`approval_digest` 仍不可实现。** B §5.5 的 `H(actionId || target || command || validFrom || validUntil || policyVersion || planDigest || argsSha256)` 没有算法/编码/字段顺序；`actionId` 对 `plan.propose` 不存在，`planDigest` 对 `action.draft` 不存在，`command` 又不得由模型生成。必须按工具分别定义 digest 输入、可选字段、会话/run/call/过期绑定；`DIGEST.md` 已可作为算法基础，但 B 尚未引用它。
3. **【阻塞】审计与 SAP 记录没有落点。** B §9 说“权威事件仍落 SAP 记录”，但 SAP 只有九类 `RecordKind`（`crates/poie-state/src/record.rs:123-135`），没有 tool/approval/agent-run 类型；塞进 `Verification`/`Observation` 会污染语义。必须选择：新增记录类型（SAP v0.3 契约变更 + ADR + SAP-T 用例）、或明确只写 append-only log + SQLite 索引并降低“权威”声明。另需 `PRAGMA user_version` 迁移，不能继续启动时 `DROP TABLE IF EXISTS events/notes`（`db.rs:58-88`）。
4. **【阻塞】工具目录与 Phase A 的函数目录仍是两套。** A §6.4 定义 `query.service_viability`/`query.external_dependencies`/`plan.propose`/`plan.preview_sap`；B §5.2 定义 `catalog.describe`/`state.query`/`evidence.get`/`plan.propose`/`action.draft`/`sim.run`，只有 `plan.propose` 重合，且没有映射。B 声称“工具 schema 来自 A Catalog”（B §9），实现上会另造静态注册表，违反 README 不变式 2。必须把 ToolType 定义为 Catalog 的投影，或合并名称。
5. **【阻塞】JSON Schema 验证器是未申报依赖。** B §5.2 要求 JSON Schema 2020-12、`additionalProperties:false`；仓库没有 `jsonschema` 或等价 crate。手写完整 2020-12 不现实；引入 `jsonschema` 属于新运行时依赖，按 `CONTRIBUTING.md` 与 `docs/DEVELOPMENT.zh-CN.md` §1.5 需要 ADR 与 `cargo-deny` 评估。另 `max_prompt_chars=8000`（B §5.4）与 `inference.rs` 的 4000 字符上限（`inference.rs:135`）冲突；`http.rs` 只有 30s 超时且无响应体大小上限（`http.rs:20-24`）。
6. **【需修改】B2 的“待审批派发请求”只有一句话。** B §5.2 说宿主可在 B2 增加 UI 命令，但不暴露给模型；没有命令名、输入/输出、审批绑定、是否可被 webview 调用、与“无 dispatch 工具”不变量的关系。要么删除该承诺，要么定义完整 API；否则实现者会各自发挥。
7. **【需修改】`pass^5` 用在 ScriptedProvider 上没有信息量。** 确定性脚本重复 5 次结果必然相同；pass^k 应只用于真实 provider 的现场评测。CI 应报告确定性用例数/覆盖矩阵，现场报告才报告 pass^k 与方差。
8. **【需修改】旧自由聊天路径绕过治理。** B §9 保留 `inference_complete` 不变；现有 Inspector 的 agent composer（`apps/desktop/src/App.tsx:1116-1146`）仍可把任意 prompt 发给 provider，不经过工具/引用/审批。必须把旧入口标记为 legacy 或让它走同一策略，否则“不降级为自由聊天”只覆盖新命令。
9. **【需修改】`two_person` 与 Phase D 冲突。** B §5.4 把双人审批留给 D；D §11 明确 MVP 单操作员。要么在 B 实现角色模型，要么删除双人预留，不能同时承诺。

**测试/验收缺口**

- 缺 capability 签发/过期/参数变更/跨会话复用测试；缺 approval digest 字段逐一变更后的 `approval_mismatch` 测试；缺审批暂停→重启→恢复/取消测试；缺 `execution_gate_set`、`sandbox_write`、`inference_configure` 的窗口/scope 拒绝测试。
- 缺 provider 超大响应、非 JSON、超时、`tool_calls` 畸形、工具输出截断的测试；缺 audit 哈希链/重建/SAP 关联测试；缺 DB 迁移测试。
- “60 个用例”没有覆盖矩阵（工具 × 场景 × 威胁 × 预算），也没有 fixture schema 版本；`pass^5` 与确定性 CI 的关系未定义。

### 1.3 Phase C（What-if 工作区）

**成立之处**

- 正确把 `Patch` 与 timed/storage 扰动分开，承认 `poie-sim` 没有 checkpoint（C:88），并明确“不自动 rebase”“不自动 TTL 删除”“比较必须列出全部差异”。
- 正确引用 `Namespace::Simulation` 与 `ensure_live_dispatch`（`crates/poie-state/src/isolation.rs:20-29`），并指出 UI 的 `Sim` 标签不是安全边界。
- 正确要求固定 `now_s`、stale 拒绝、promote 只产生 proposed、不写 live。

**缺陷与缺口**

1. **【阻塞】Snapshot 字段不足以支撑 timed/plan 运行。** C §5.1 的 `baseState` 只有 assetStatus/lots/storageLimits/evidenceVersion，缺 `nowS`、`seed`、`horizonS`、`maxDepth`、`maxActions`、`StateId`、`assetTimelines`、`stockTimelines`、`evidenceValidity`、storage 的 capacity/initial SoC。`TimedScenario` 需要这些字段（`crates/poie-sim/src/timed.rs:225-235`）；缺了就无法从 snapshot 重放出与 `TimedRunner` 相同的结果。`StateId` 虽已存在但不可序列化（`crates/poie-state/src/projection.rs:60-66`），且桌面没有 record log/`ProjectedState` 可供比较。
2. **【阻塞】桌面没有“live 状态”，stale 检查是空的。** `domain_overview` 返回启动时计算的静态快照（`lib.rs:236`、`:175-177`），`execution`/`federation` 也是启动时计算；没有任何可变模型/证据版本。C 说“live 模型/证据/StateId 改变则拒绝”（C:136），但该条件永远不成立；“模拟前后 live 不变”的验收（C §10）也因此是平凡测试。必须定义 live 状态源（当前只是 Base-01 demo），或把 C 明确限制为“静态 demo snapshot 上的 what-if”，并删除 live 不变量的强声明。
3. **【阻塞】`PatchEnvelope` 复制 `Patch` 且不可序列化。** C §5.2 的字段与 `poie-ontology::Patch`（`model.rs:813-846`）逐项重复，但 `Patch` 只 derive `Deserialize`，没有 `Serialize`；`AssetStatus`/`LotQuality`/`Verification` 也没有 `Serialize`（`model.rs:33-58`）。C 需要持久化/返回 snapshot、variant、digest，必须给 `Patch` 加 `Serialize`（或定义 DTO 并做双向转换），并明确 `PatchEnvelope -> Patch` 的唯一转换与拒绝规则，否则两套字段会漂移。
4. **【阻塞】verdict 词表冲突。** C:95 仍把 `candidate` 与 `too_late`/`power_limited` 放在一个枚举，C:139 才说归一化为 `VerdictKind`。但 `VerdictKind` 已是 VEC 的六类结论名（`crates/poie-viability/src/verdict.rs:11-20`），且结构 `Verdict` 是 `candidate/blocked/unknown/search_limit`（`graph.rs:40-45`），时序是 `feasible/blocked/.../power_limited`（`timed.rs:159-168`）。必须定义独立类型（如 `OutcomeKind { structural, timed }`）并写明 `candidate <-> feasible` 映射、每种 mode 允许的结论域；否则 UI/API 会把“结构 candidate”误读成“时序可行”。
5. **【阻塞】promote/compare/approval 的 schema 缺失。** `PromotionResult`、`CompareResult`、`approval` 输入都没有字段；`whatif_variant_promote(variantId, approval)` 中的 `approval` 若是布尔值，就是审批绕过。必须定义 `ApprovalToken`（绑定 snapshot/variant/plan digest、操作员会话、过期、一次一用），并复用 B 的审批原语。更关键的是：现有 `validate_plan_through_sap` 会直接授权（`sap.rs:77-82`），C 需要一个**只创建 proposed** 的新 API，不能声称“SAP 校验后创建 proposed”却调用授权函数。
6. **【阻塞】timed/storage 变体不可按描述构造。** `TimedScenario::from_parts` 把 `storage_discharge_w` 置空且没有 setter（`timed.rs:349-367`）；字段虽 `pub`，直接写可绕过单位/非负校验。C 的 `storageDischargeW: map<resourceId,number>` 还缺 capacity/initial SoC，而 Base-01 timed storage 的容量与初始能量来自场景文件（`scenarios/base-01-timed/scenario.toml:93+`）。没有这些，验收“至少一个 `power_limited`”无法稳定构造。必须增加受校验的 `TimedScenario::with_storage`（或 checkpoint）并把 storage 状态纳入 snapshot。
7. **【阻塞】隔离声明不可执行。** `ActionLedger` 没有 namespace 字段，`Namespace` 只在 `Envelope`（`record.rs`）；`ensure_live_dispatch` 只在派发时检查（`isolation.rs:20-29`），不约束 variant 的 plan/ledger/记录。若 variant 根本不写 SAP 记录，则“所有变体记录使用 Simulation namespace”是空话；若要写，则需要记录级 API。必须二选一并补测试。
8. **【需修改】`Mode::Sim` 不存在。** 前端只有 TS 联合类型 `"model"|"sim"|"shadow"|"live"` 和 `MODE_ENABLED.sim=false`（`apps/desktop/src/App.tsx:51`、`:69`），没有 Rust `Mode`。C §8.1 应写成“前端 Mode 的 sim 项”；同时“Sim 下所有写操作被禁止”在 host 没有对应状态，`execution_gate_set` 仍可调用（`lib.rs:184-205`），需 host 侧模式或明确只约束 `whatif_*`。
9. **【需修改】digest、预算、排序、持久化未定义。** C:103 的 digest 没有算法/规范化；`budget_exceeded` 没有预算数值；`CompareResult` 的排序键/升降序/并列规则未定义；C §12 说结果只存内存、定义/摘要存 DB，但 `db.rs` 没有 snapshot/variant/run 表和迁移。所有这些都应先冻结。

**测试/验收缺口**

- 缺 snapshot 全字段 round-trip、`StateId` stale 检测、per-mode verdict 域、timed 从 snapshot 重放、storage 上下界、compare 排序稳定性、promotion token、`proposed` 状态断言（现有 SAP 路径会 authorized）。
- “live 不变”测试没有可变 live 可比；需要构造可注入的 live 状态或明确删除该测试；否则验收 2 是假阳性。
- 缺并发/取消/超时、DB 迁移、审计、namespace 泄漏测试。

### 1.4 Phase D（站点交付）

**成立之处**

- 比上一版完整：Release/Site manifest、DeliveryPlan、recall、SBOM/attestation、`host_info` 的 release/channel、tag release 依赖根验证、桌面 workflow PR paths 修正，都是可执行方向。
- 正确坚持“SHA-256 不等于签名”“bundle 不作为 live 备份”“默认关闭遥测/自动更新”，并引用现有 `SHA256SUMS` 发布链。
- 正确要求 `files[]` 拒绝绝对路径/`..`，以及未签名明确标记。

**缺陷与缺口**

1. **【阻塞】导入的模型没有消费方，D 的核心承诺不可实现。** 桌面模型是编译期 `include_str!("../../../../scenarios/base-01/model.toml")`（`apps/desktop/src-tauri/src/domain.rs:12`），`AppState` 只保存算好的 `DomainOverview`（`lib.rs:37`、`:241-252`），没有 model store/loader。D 的 `files[]`（D §5.1）写着 `models/base-01.toml`，场景又说“携带新 bundle 导入并显式切换”，但导入后运行的仍是编译进去的 Base-01。验收 1/3（离线导入可启动、模型不匹配拒绝）因此不可验证。必须新增“模型/证据存储 + 解析优先级（内置 fallback vs bundle）+ 启动时加载”，或把 D 明确降级为“元数据/策略交付”，不宣称模型切换。
2. **【阻塞】digest 规范已有，但 D 的来源仍未闭合。** `DIGEST.md` 已定义 `poie.model/0.1`、`poie.manifest/0.1` 等域和“排除 integrity”规则，这是进步；但 D 仍需 `contractsDigest`、`evidence.digest`、`executionDigest`、`federationDigest`，而 `ExecutionSnapshot`/`FederationSnapshot` 没有 digest（`execution.rs:48-56`、`federation_view.rs:38-48`），contracts 版本只在中文契约首行散文（`specifications/physical-ontology.zh-CN.md:1`），没有机器可读元数据；`evidence.digest` 也没有证据文件路径/哈希来源。D 必须列出每个 digest 的 payload 与来源，并让 `poie-delivery` 复用 `poie-digest`，不能各自实现。
3. **【阻塞】“复用 sandbox 路径校验”不成立。** 现有 `sandbox::resolve` 只拒绝非 Normal 组件，不检查符号链接；`write` 用 `std::fs::write` 跟随符号链接、没有临时目录/原子 rename/fsync，且只支持 UTF-8 文本、单文件 1 MiB（`sandbox.rs:9-48`）。D 的“导入使用临时目录 + 原子发布”“符号链接拒绝”需要全新 API 与测试（no-follow、同文件系统 rename、目录权限、大小上限），不能把现有函数当基线。
4. **【阻塞】“所有写操作要求登录”是假的。** 现有 `sandbox_write`、`sandbox_read`、`domain_overview`、`execution_gate_set` 都未强制 `auth::require`（`lib.rs:175-221`、`:184-205`）；`inference_complete` 只检查配置/会话，不读站点策略。D 的策略“所有 host/Agent 路径读取同一策略”（D §8）没有 loader/强制点；`remote_inference_allowed=false` 也无法阻止现有命令。A0 已把 gate/sandbox 的缺口列为 Phase A 前置，但 D 仍要补 `inference_*` 的 policy 检查，并列出每个命令的 auth/scope/policy；否则 D 的审批是装饰。
5. **【阻塞】命令/数据模型缺失。** D 的命令表（D §6.2）没有 `delivery_bundle_inspect`（但 UI 要求导入前预览）、`delivery_recall`（但 §5.3 定义 recall）、`delivery_plan`（但 UI 要解释“为什么没有更新”）、`delivery_channel_set` 的审批参数。`DeliveryPlan`、`Diagnostics` 的部分字段没有来源：`recentErrors` 没有错误存储（只有 audit 字符串），`executionDigest`/`federationDigest` 无函数。`channel` 是本地 SQLite 设置还是 manifest 字段，冲突时谁优先，未定义。
6. **【阻塞】数据库迁移/回滚没有底座。** D 要求回滚“数据库迁移可逆或先备份”（D §5.3），但 `db.rs` 没有 `user_version`、没有迁移框架，启动时还会 `DROP TABLE IF EXISTS events/notes`（`db.rs:58-88`）。D 需要独立的迁移/备份/恢复设计；否则验收 4“回滚可测”无法实现。
7. **【需修改】构件与流水线不一致。** D §5.1 的 `artifacts` 示例是 `.dmg`，但 `.github/workflows/desktop-build.yml` 只构建 `.app` 后打成 `.app.zip`（`:135-148`），全仓库没有 `hdiutil`/DMG 步骤；README 产品段仍宣称 macOS DMG，而其漂移段又承认“当前 Release 的 DMG 是手动补的”。必须让 manifest 只记录流水线实际产物，或补 DMG 构建并更新 CI。`sbom`/`attestation`（D §5.1、§10）没有工具（如 cargo-cyclonedx）与 CI 权限设计；`app.target` 作为运行时值也需要构建期注入，不能运行时猜测。
8. **【需修改】诊断包字段来源不足。** `auditSummary.eventCounts` 可从 `audit` 表算；`recentErrors` 没有结构化错误表，`executionDigest`/`federationDigest` 没有 digest；`redactions[]` 没有扫描器/白名单规则。导出前扫描“禁止字段”需要明确算法与测试（不仅是白名单检查表）。
9. **【需修改】通道语义与升级叙事混淆。** D §5.2 说通道是订阅，但 `site` 通道实际上是站点固定；`app.version` 不兼容拒绝与“导入新 bundle 升级”互相矛盾（新 bundle 的目的就是改变版本）。必须区分“同版本证据/策略重绑定”与“跨版本升级”，后者需要安装器更新，而 D 非目标又说不做自动更新。

**测试/验收缺口**

- 缺 manifest 自哈希（排除 integrity）、规范化、篡改任一字节、重复路径/大小写冲突/符号链接/硬链接、跨设备 rename、导入中崩溃、权限/磁盘满、bundle 超限测试。
- 缺“导入后实际使用导入模型”的测试（当前不可能）；缺 channel/manifest 冲突、recall、DeliveryPlan 解释、policy 强制、远程推理关闭、`execution_gate_set` 认证的测试。
- 缺 diagnostics 字段逐一来源断言与 `redactions` 负例；缺 DB 迁移失败/回滚备份测试；缺 DMG/SBOM/attestation 与流水线一致性检查。

## 2. 跨阶段问题

1. **【阻塞】B 与 D 存在策略依赖环。** B 说工具 scope 由 A Catalog 派生、站点策略留到 D（B §12）；D 的 `agentScopes` 又要由 B 的运行时执行（D §5.2、§6.2）。在 B 没有 policy loader、D 没有 enforcement 接口之前，两者互相等待。必须先定义**一个**宿主 `Policy` 类型/加载器（B 默认值 + D SQLite 覆盖），A/B/C/D 全部只读它。
2. **【阻塞】审批原语重复且未落地。** B 有 `approval_digest` 与 capability；C 有 `whatif_variant_promote(approval)`；D 有 `delivery_channel_set` 的“操作员确认”。三者没有共享 `ApprovalClass`/`ApprovalToken`/验证函数。必须在 A/B 之间抽出一个库级审批原语（或明确只由 B 提供），否则 C/D 会各自实现布尔确认，形成审批绕过。
3. **【阻塞】安全边界仍被现有命令击穿。** A §4 已正确把 `execution_gate_set`、`sandbox_*` 的鉴权列为 A0，但代码现状仍是：`execution_gate_set` 先改 gate 再可选审计（`lib.rs:184-205`），`sandbox_write` 无 auth 且跟随符号链接（`lib.rs:219-221`、`sandbox.rs:38-47`），`inference_complete` 不读站点策略，所有窗口共享命令（`capabilities/default.json` 只有 `core:default`）。B/C/D 不能把 A0 当“已完成”；A0 必须先合并，否则后续“审批/隔离/策略”都是 UI 层。
4. **【阻塞】digest 规范已有，但实现与跨文档引用未闭合。** `DIGEST.md`/A §7.0 已定义 SHA-256、域分隔与 canonical JSON，这是进步；但 `poie-digest` 不在 workspace，`DIGEST.md` 未给出 Rust `ModelCanonical` 类型或测试向量文件；B 的 `approval_digest` 没引用它；C 的 `VariantResult.digest` 没引用它；D 的 `manifest_sha256` 虽引用规则但未接入。另 `serde_json` 默认不按 key 排序，canonical 输出必须由实现强制；`Plan::digest()`/`Simulation::digest()` 是可读指纹（`plan.rs:64`、`timed.rs:210`），不能当 SHA-256 使用。
5. **【阻塞】缺少统一 schema/版本注册表。** A 的 `poie.ontology-catalog/0.1`、D 的 `poie.site-bundle/0.1`、sim 的 `poie.timed-scenario/0.2`/`timed-cases/0.1`、契约的 `0.3-draft` 各自为政；A/C/D 都说“未知版本拒绝”，但没有兼容矩阵、没有 `schemaVersion` 的统一字段、没有迁移规则。C 的 `Snapshot` 有 `schemaVersion`，`VariantResult`/`DeliveryState` 却没有。
6. **【阻塞】模型类型不能直接序列化。** `AssetStatus`、`LotQuality`、`Verification`、`Patch` 只有 `Deserialize`（`model.rs:33-58`、`:813`），而 A/C 的 JSON/DB/HTTP 投影需要 `Serialize`；`StateId` 也没有 serde（`projection.rs:60`）。必须明确：给领域类型加 serde（并冻结字段名）还是定义独立 DTO；两者不能混用，否则 camelCase/枚举拼写会漂移。
7. **【需修改】重复抽象清单。** `DomainOverview`/`EntityView`/`ServiceView`（`domain.rs:80-90`）与 A 的 `ObjectInstance`；A 的 `FunctionType` 与 B 的 Tool Catalog；C 的 `PatchEnvelope` 与 `Patch`；C 的 `VariantResult.sapPreview` 与 `ExecutionSnapshot.checks`（`execution.rs:48-56`）；A 的 `evidence` 对象与现有 `EvidencePolicy`/`projection`（`crates/poie-state/src/projection.rs:25`）；D 的 `Diagnostics` 与 `host_info`（`lib.rs:46-54`）。每一组都要指定唯一真源与迁移/删除路径。
8. **【需修改】README 不变式 P2 与 Phase D 冲突。** README 的 P2 写“一切变更都是动作”，但 D 的 channel/policy/import/rollback 是配置写命令，不是 SAP 动作。应把 P2 限定为“物理/运营状态变更”，配置写路径单独声明其 auth/audit/审批要求，避免为满足口号而把设置写成假动作。
9. **【需修改】桌面 crate/workspace 约束未落到设计。** 新 crate 必须进入根 `Cargo.toml` 的 `members`（当前 `exclude` 桌面，`Cargo.toml:3-4`），桌面必须新增 path 依赖并更新自己的 `Cargo.lock`；根 CI 用 `cargo test --locked`，需要同步根 `Cargo.lock`。`poie-sdkgen`、`poie-governance`、`poie-whatif`、`poie-delivery`、`poie-digest` 都会改变 `scripts/check-test-matrix.py` 的计数（它枚举所有 `poie-*` 包并校验 `docs/status/test-matrix.zh-CN.md` 与 `status-snapshot.zh-CN.md`），设计只在阶段末尾提“测试矩阵”，没有把这一步列为每个 PR 的硬门禁。`cargo-deny`、`rustdoc -D warnings`、ADR（新运行时依赖/契约变更）也未写进各阶段实施顺序。
10. **【需修改】新依赖未申报。** 至少涉及：JSON Schema 验证器（B）、`sha2`/哈希（`poie-digest`）、`serde` 序列化（A/C）、SBOM 工具（D）、可能的 TS 代码生成依赖（A）。按 `CONTRIBUTING.md` 与 `DEVELOPMENT` §1.5，新运行时依赖需要 ADR 与 `cargo-deny` 许可评估；目前没有依赖清单/许可证结论，无法判断可实施性。
11. **【阻塞】README 要求最终校验“必须 PASS”。** README 设计门禁现在写“最终独立校验报告……必须末行为 `VERDICT: PASS`”。这会把独立校验变成盖章：若存在 REQUIRED 修改，校验器只能违背事实或被迫改结论。门禁应只要求“给出 PASS/CHANGES_REQUIRED/FAIL 及证据”，不能预设结果；这本身是流程性阻塞，应先改 README。
12. **【需修改】文档自身不受版本控制。** 四份设计 + 研究备忘全部是未跟踪文件，校验期间还在变化（22:40→22:52）。设计门禁要求“校验通过才开发”，但没有可引用的 commit/摘要，PR 无法证明“实现对应哪版设计”。应先提交设计文档，冻结快照，再重新跑本校验。

## 3. REQUIRED 与 OPTIONAL

### REQUIRED（开发前必须修复）

| 编号 | 必须修改 | 涉及 |
| --- | --- | --- |
| R1 | 冻结设计文档到 git；删除 README“最终校验必须 PASS”的预设，改为“必须给出 PASS/CHANGES_REQUIRED/FAIL 及证据” | 全部 |
| R2 | 把 `DIGEST.md` 落成 `poie-digest` 实现：Rust `ModelCanonical` 类型、canonical JSON 的 key 排序/数字规则、域分隔、固定测试向量；B/C/D 统一引用它 | A/B/C/D |
| R3 | 统一 schema/version 注册表与兼容矩阵；`VariantResult`/`DeliveryState` 补 `schemaVersion` | A/C/D |
| R4 | 统一 DTO/序列化策略（领域类型加 `Serialize` 或独立 DTO）；补 `Patch`、`AssetStatus`、`LotQuality`、`Verification`、`StateId` | A/C |
| R5 | 定义统一错误包络 `{code,message,details}` 与每阶段错误码表；host 命令不再返回裸 `String` | A/B/C/D |
| R6 | 把 A0 落为代码：`execution_gate_set`/`sandbox_*`/`inference_*` 的 auth/scope/policy；命令 × 窗口 × 登录矩阵与测试 | A/B/C/D |
| R7 | 定义模型来源与解析：当前 Base-01（含 degraded patch）唯一来源；D 的 bundle 模型存储/加载/优先级；或明确 D 不做模型切换 | A/C/D |
| R8 | A：给出 `parameters` 的确定性派生表（目标/数量/runs/输入/时长）或删除参数声明；新增不授权的 `preview_plan_through_sap` 并定义 `SapPreview.assumed` | A |
| R9 | A：单一 SDK 生成路径（`poie-sdkgen`），输入/输出/CI/测试矩阵/`Cargo.lock`/ADR 一次写清；删除外部 JSON API 声明 | A |
| R10 | B：定义 `agent_run` + approve/reject/resume/cancel/status 命令、运行状态序列化/恢复、Tauri 事件；解决“持久化/内存”矛盾 | B |
| R11 | B：定义 per-tool `approval_digest`（引用 DIGEST）、capability 撤销/过期语义；决定审计用 SAP 记录（需契约变更）还是 append-only log；补 `user_version` 迁移 | B |
| R12 | B：把 Tool Catalog 定义为 A Catalog 的投影；删除/定义 B2“待审批派发请求”；引入 JSON Schema 验证器并走 ADR；统一 4000/8000 字符与响应体上限 | B |
| R13 | C：补全 Snapshot（now/seed/horizon/maxDepth/maxActions/StateId/timelines/storage capacity+SoC/evidence validity）；定义 per-mode `OutcomeKind` 与映射；`PatchEnvelope` 去重 | C |
| R14 | C：定义 Compare/Promotion/Approval schema；新增 proposed-only SAP API；定义 timed storage 构造与 checkpoint；落实 namespace 记录 | C |
| R15 | D：接入 `DIGEST.md` 的 model/contracts/execution/federation/evidence 摘要；机器可读 `contracts`；`manifest_sha256` 自哈希规则 | D |
| R16 | D：新增 no-follow/临时目录/原子发布的 sandbox import API；DB 迁移/备份/回滚；policy loader 与命令级强制 | D |
| R17 | D：补 `delivery_bundle_inspect`、`delivery_recall`、`delivery_plan`、channel 审批参数；明确 channel vs manifest 优先级；修正 DMG/`target`/SBOM 声明 | D |
| R18 | 各阶段实施顺序加入：workspace members、根/桌面 Cargo.lock、`check-test-matrix`、`cargo-deny`、ADR、rustdoc、文档链接检查 | 全部 |

### OPTIONAL（可延后/可降级）

- O1 pass^k 只用于真实 provider 的现场评测；CI 继续用确定性用例 + 覆盖矩阵。
- O2 SBOM/attestation、Ed25519 签名、自动更新器延后；当前 SHA-256 + 明确“未认证”是可接受的 MVP。
- O3 DMG 与 portable exe：要么补构建，要么改 README/manifest 示例，不阻塞 A–C。
- O4 `poie-sdkgen` 生成 TS 之外再生成 JSON Schema/OpenAPI，可后续做。
- O5 Catalog 对象图布局算法（确定性坐标）可先用稳定布局，不必现在设计图数据库。
- O6 VEC 指标、跨站点 what-if、bundle 压缩/差分、multi-tenant 留到后续。
- O7 capability 的 opaque token 可先用宿主内存映射；但需补撤销/清理测试，不要把它当跨进程安全凭证。
- O8 `recentErrors` 可先只复用 audit 事件；但必须删掉“结构化错误”的承诺或补存储。

## 4. “可以开始 Phase A 了吗？”检查清单

- [ ] R1 设计文档已提交、快照冻结；README 不再预设 PASS。
- [ ] R2 `poie-digest` 已实现并有固定测试向量；`ModelCanonical` 字段表与 canonical JSON 规则可执行。
- [ ] R3/R4/R5 Catalog DTO、`Serialize` 策略、`schemaVersion`、错误包络已定。
- [ ] R7 “当前模型”= Base-01 + degraded patch 的唯一来源已定；host 持有 `Model` 而非仅 `DomainOverview`。
- [ ] R8 `parameters` 派生表已定；`preview_plan_through_sap` 不产生 `Authorized`/`Authorization`。
- [ ] R9 `poie-sdkgen` 唯一生成路径、CI 检查、测试矩阵/`Cargo.lock`/ADR 已定。
- [ ] R6 A0 的 gate/sandbox/inference auth/scope 已修，且命令 × 窗口矩阵有测试。
- [ ] 属性 `asOf` 来源已定（否则标 unknown）；`evidence` 对象不伪造 validity。
- [ ] `can_submit` 的 submit 命令或删除决定已定；无死字段。
- [ ] Playwright/i18n/截图/`check-docs`/`check-test-matrix` 的验收命令已写入 PR 模板。

**结论：当前为“否”。** 至少 R1–R9 未完成前，Phase A 不应进入编码；R10–R18 是 B/C/D 的开工前置。

## 5. 研究遗漏与过度声明

- 三份研究备忘已纳入 README，这是进步；`DIGEST.md` 也吸收了上一轮“digest 不可复现”的批评。但备忘中的关键结论仍未全部变成**决定**：B 未采用“审批是可序列化中断”的具体实现（OpenAI HITL 的 `RunState` 语义），仍把 `pass^k` 用在确定性脚本上；B 的审批摘要也未引用 `DIGEST.md`。
- **过度声明 1**：A 的“外部开发者通过 JSON API/TS SDK”与“外部消费者仅凭 Catalog”超出 MVP 的 Tauri IPC 边界（A §5；A §13 自己承认）。应改为桌面内 SDK，外部接入另立阶段。
- **过度声明 2**：D 的“断网导入并启动/切换模型”超出当前编译期 `include_str!` 架构（`domain.rs:12`）；在新增 model store 之前，验收 1/3 不可验证。
- **过度声明 3**：D 的 `.dmg` artifact（D §5.1）与现有流水线不符（`desktop-build.yml:135-148` 只有 `.app.zip`/NSIS/deb）；README 已承认当前 Release 的 DMG 是手动补的，Phase D 必须让 manifest 只记录流水线实际产物，或补 DMG 构建。
- **过度声明 4**：README 现在要求最终校验“必须末行为 `VERDICT: PASS`”。这不是研究结论，而是把质量门禁变成预设结果；独立校验必须允许 CHANGES_REQUIRED/FAIL。该句应先删除。
- **遗漏 1**：研究备忘明确指出 MCP/OSDK 的 read-time enforcement 不等于下游安全、prompt injection 无可靠防御；B 仍以“系统提示 + untrusted 标记”作为主要控制，Rule of Two 只停留在画像表，没有对应的数据分类（哪些工具输出属于 T0/T1/T2/T3）与调度器。应把“工具结果信任级”写进 `ToolResult` 与策略，而不是只写 `untrusted=true`。
- **遗漏 2**：VEC 明确“blocked 只相对当前快照/求解器，不是不可行证明”；C 的 `blocked`/`candidate` 与 UI 比较没有携带该限定。应在 `VariantResult`/UI 中保留“相对快照”的限定，避免把结构 blocked 宣传成物理不可行。
- **遗漏 3**：没有现场可用性/成本/迁移研究；D 的“站点交付”没有安装器、DB 迁移、磁盘/权限、操作员培训与支持流程。作为产品阶段，至少应有一份“部署前置条件/失败手册”，否则 D 的验收只覆盖 happy path。
- **过程问题**：设计文档在本次校验期间被并发修改（研究备忘与四份文档的 mtime 均在 22:44–22:52），且全部未跟踪。校验结论只能对当前工作树负责；在 R1 完成前，任何“校验通过”都不应被引用。

VERDICT: CHANGES_REQUIRED
