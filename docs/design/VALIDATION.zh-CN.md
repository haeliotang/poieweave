# Poieweave Phase A–D 设计独立校验（最终）

- 校验对象：[README](README.zh-CN.md)、[Phase A](phase-a-ontology-product.zh-CN.md)、[Phase B](phase-b-agent-governance.zh-CN.md)、[Phase C](phase-c-what-if.zh-CN.md)、[Phase D](phase-d-site-delivery.zh-CN.md)、[Digest](DIGEST.md)、[平台契约](PLATFORM.zh-CN.md)，以及 [Palantir](research/palantir-patterns.zh-CN.md)、[Agent 治理](research/agent-governance.zh-CN.md)、[差距审计](research/repo-gap-audit.zh-CN.md)。
- 代码基线：分支 `design/phase-a-d`，HEAD `f75c31d`；`docs/design/` 已入库，`git status` 仅有本校验文件被修改。README:103 仍记录冻结快照 `4a90edc`，与实际 HEAD 不一致。
- 文档快照：2026-11-04 23:00（本地）。校验期间文档被多轮并发改写，行号以本快照为准；最终结论必须以提交后的快照重跑。
- 方法：逐条对照 PO/SAP/VEC、`crates/poie-*`、`apps/desktop/src-tauri`、`apps/desktop/src`、根 workspace、CI、`scripts/` 与 ADR；`python3 scripts/check-docs.py` 通过。未运行完整 `cargo test`，未修改应用源码、设计文档或 git 状态，仅重写本文件。

## 0. 总判定

**PASS WITH REQUIRED CHANGES。** 本轮修订质量明显提高：A0 宿主鉴权前置、`DIGEST.md`、`PLATFORM.zh-CN.md`、只读 `preview_plan_through_sap`、B 的 `agent_*`/`RunState`/per-tool approval digest、C 的 `OutcomeKind`/`plan_to_proposed`、D 的“不导入模型/不做 DMG”都已落地，上一轮多数阻塞项被实质性解决。

剩余问题集中在“跨文档最后一致性”和少数实现契约未闭合：A 的 `OntologyCatalog` 使用 `catalog_sha256`，但 `ObjectPage`/host 使用 `catalog_digest`，`SapPreview` 字段名也前后不一致；B 的持久化叙述与新的 `agent_runs` 设计冲突；C 的依赖树漏 `poie-state/poie-digest`，`PatchDto` 与 `Patch` 并非一一对应；D 与 PLATFORM 对 `ModelStore` 的取舍相反；PLATFORM 引用的 `SCHEMAS.md` 不存在，窗口标签与现有 `poie-view-*` 不符。这些是文档级修复，不需要推翻架构，但在修复前不应开始 Phase A 编码。

## 1. 逐文档校验

### 1.1 `README.zh-CN.md`

**成立之处**

- 跨阶段不变式、阶段退出条件、漂移清单仍与 PO/SAP/VEC 一致；README:41 明确列出 R1–R5 修订，并新增 [DIGEST](DIGEST.md)/[PLATFORM](PLATFORM.zh-CN.md) 两个交叉规范入口（README:43-46）。
- README:30 已改为“由 validator 独立给出 verdict”，不再预设 PASS；这解决了上一版门禁自相矛盾的问题。
- README:99-120 的修订记录逐条回应了前一轮 R1–R18，便于追踪。

**缺陷**

1. **退出条件仍写“外部消费者仅凭 Catalog/SDK”**（README:80），与 Phase A 已改为“桌面内开发者/Tauri IPC”（A:54、A:327）以及 README:41 的 R9“删除外部 JSON API 声明”冲突。必须改为“桌面内消费者”，否则 A 的验收标准无法与设计一致。
2. **冻结记录不准确。** README:103 写冻结快照 `4a90edc`，但当前分支 HEAD 是 `f75c31d`；设计门禁要求“冻结后重跑”，README 的冻结记录必须指向实际 commit。
3. **P2 仍写“无 CRUD/写接口”**（README:66），与 D 的 `delivery_*` 写命令（D:162-175）冲突。应限定为“不得有绕过 SAP 的领域对象/执行写接口；配置/交付状态写入必须审计、受 ApprovalToken/HostPolicy 约束”。
4. **阶段顺序仍只有严格串行**（README:14），而 PLATFORM:157-164 给出 A0→A1→A2、B1→B2、C1→C2、D1→D2 的切片。应把切片写进 README，至少说明 B1/C1 是否允许并行；否则团队会按两种节奏执行。
5. **“四份设计文档”**（README:101）已过时：现在还有 DIGEST、PLATFORM、三份备忘；应更新清单，避免遗漏交叉规范。

### 1.2 `phase-a-ontology-product.zh-CN.md` 与 `DIGEST.md`

**成立之处**

- A0 已写入正文第 4 节，明确 gate/sandbox 鉴权、命令 scope 矩阵、错误包络、demo 授权移除（A:36-46）；并引用 PLATFORM §2–§4 的 DTO/错误/scope。
- `lot` 明确不暴露未进入 `Lot` 的 `location/evidence`（A:68），与运行时 `Lot` 只有 `id/resource/si/quality`（[model.rs](../../crates/poie-ontology/src/model.rs):290-295）一致；A:110 对治理字段缺失采用 fail-closed，避免第二套策略。
- A:219-234 新增只读 `preview_plan_through_sap(&Plan, &AssumedContext) -> SapPreview`，明确不写 ledger、不创建 Authorization、不产生 transition，并给出 `assumed`/`failed` 的逐项状态；这解决了“预览伪造授权”的核心问题。
- A:173-215 给出了 `ObjectType/ObjectInstance/ObjectDetail/OntologyCatalog/ObjectPage/SapPreview/ActionPreview` 的字段；`DIGEST.md` 区分 fingerprint 与 SHA-256、定义 domain/canonical JSON/manifest 排除规则，方向正确。
- A 的 `ActionType` 已删除 `can_submit`/approval/risk，改为模型字段 + Phase B 治理元数据（A:104-110），并明确 Tool Catalog 是函数目录的投影（A:127）。

**缺陷**

1. **【阻塞】`OntologyCatalog` 的 catalog 摘要字段名与调用方不一致。** A:201-209 定义 `catalog_sha256`（注释：排除该字段后哈希），但 A:211-216 的 `ObjectPage.catalog_digest`、A:275 的“返回 catalogDigest”和 DIGEST:21 的 `poie.catalog/0.1` 都使用 `catalog_digest`/`catalogDigest`。同一摘要三个名字会让 SDK/UI 无法对接；必须统一为一个（建议 `catalog_sha256` 或 `catalog_digest`），并明确 payload 排除规则、host 序列化顺序与测试。
2. **【阻塞】`SapPreview` 字段名前后不一致。** A:201-205 定义 `rejection_reasons`，A:231 又写 `rejections`，C:113 也使用 `rejections`；同一结构两个名字会让 SDK/UI 漂移。必须统一（建议 `rejections`）并纳入 `CheckPreview` 快照。
3. **【阻塞】`plan.preview_sap` 的输入仍是“plan digest”。** A:123 写“plan digest, assumed context”，但 A:224-227 的新 API 接收 `&Plan`；没有 plan registry，digest 不能还原 Plan。应把函数目录输入改为“plan object（由 host 持有）”，或明确先实现 plan store。
4. **【阻塞】A0 的 synthetic preview 授权类型与 `Authorization` 不匹配。** A:43 写 `scope=["preview"]`（数组），但 `Authorization.scope` 是 `String`（[action.rs](../../crates/poie-state/src/action.rs):213-218）。应改为 `scope="preview"`，或明确扩展 `Authorization` 把 scope 设为集合（后者是 SAP 契约变更，需 ADR）。另需断言 synthetic preview 不能进入 `dispatch`，且未登录时 execution 视图不生成授权动作。
5. **【阻塞】A0 的测试计划仍未覆盖 `execution.rs`/`federation_view.rs` 的现有断言。** A:43 已把两处 demo 授权改为 synthetic preview，这是正确方向；但 `execution.rs` 仍硬编码 `policy_version/state_version/predicates`，测试断言 `review_status=="ready"`、0 rejections、全部 `authorized`（execution.rs:181-190）。改成 synthetic preview 后必须同步修改这些断言，并新增“synthetic 不可 dispatch”的负例；A0 只写“替换”，没有写测试更新/失败预期。
6. **【阻塞】A 的 DTO 仍缺关键类型。** A:175-215 已定义 `PropertyDef`、`PropertyValue`、`LinkType`、`LinkInstance`、`EvidenceRef`、`ObjectType`、`ObjectInstance`、`ObjectDetail`、`SapPreview`，但 `ActionType`、`FunctionType`、`Predicate` 仍只有引用没有字段；PLATFORM:40 又使用 `ObjectInstanceDto` 等不同命名。A1 前必须给出这些类型的字段/单位/可空性/排序，或明确由 PLATFORM 定义并统一命名。
7. **【需修改】Digest 规范仍有字段来源冲突。** DIGEST:71-72 要求资产 canonical 含 `description`，但运行时 `Asset` 没有该字段（model.rs:266-288）；DIGEST:64 又说“只从已验证的 `Model` 字段构建”。必须把 `description` 加入 `Asset`（改 PO 实现/版本）或从 canonical 删除；否则只能读 `Model.doc`，绕过验证。DIGEST:56/58 对字符串数组、`-0.0`、浮点精度的规则仍不足，需固定测试向量。
8. **【需修改】对象图的布局/边派生仍未定义。** A:314 要求 `CapabilityGraph` 节点来自 Catalog，A:360 已加“删除 `GRAPH_NODES` 后仍能渲染”的测试；但现有组件用硬编码 x/y 与索引边（[App.tsx](../../apps/desktop/src/App.tsx):80-99、:532-536），Catalog 只给类型/链接，没有布局函数。A:383 已写“Catalog mock/host E2E 策略”，但未定义具体 fixture/布局；MVP 至少应给出确定性布局或保留手工坐标的映射规则。

**测试缺口**

- 缺 Catalog 逐字段黄金 fixture、`catalog_digest` 规范化/排除自身字段测试、cursor/limit/未知 schema 的 host 边界测试、`model_mismatch`、`asOf/declared` 属性测试。
- 缺“对象图节点集合等于 Catalog 子集”的断言；否则 `GRAPH_NODES` 可以继续存在。
- 缺浏览器预览 mock Catalog 的 smoke 与 host JSON 快照；缺 `SapPreview` 的 assumed/failed/unknown 三分支测试。

### 1.3 `phase-b-agent-governance.zh-CN.md`

**成立之处**

- 新增完整的 Host API：`agent_run/status/approve/reject/cancel/resume`，`RunState` 持久化到 `agent_runs.state_json`，Tauri 事件 `agent://state`，重启标记 `interrupted`（B:212-233）；这解决了上一版“暂停后无法继续”的阻塞。
- per-tool approval digest 已按工具定义字段并引用 DIGEST: `plan.propose/action.draft/sim.run/pending dispatch`（B:171-181）；审批使用 PLATFORM §6 的 `ApprovalToken`，删除双人审批（B:182）。
- 明确不做通用 JSON Schema 求值器，改用 Rust 强类型 + `serde(deny_unknown_fields)`（B:103），避免引入 `jsonschema`；ToolResult 增加 `evidenceRefs/asOf/sourceClass/trustProfile`（B:110-115），比上一版更接近研究备忘。
- pass^k 只用于现场真实 provider，CI 只报确定性用例（B:260-261）；`inference_complete` 降为 Settings 兼容路径（B:306）；审计不新增 SAP RecordKind，改为 append-only 日志 + SQLite 索引（B:308）。这些都是正确修正。

**缺陷**

1. **【阻塞】`MVP 运行存内存，重启即取消` 与新的持久化设计冲突。** B:345 仍写“MVP 运行存内存，重启即取消”，但 B:223-233 已定义 `agent_runs` 持久化、`interrupted`、`agent_status` 查询。两者不能同时成立；必须删除 B:345 的旧表述，明确“RunState 持久化；模型调用不自动恢复”。
2. **【阻塞】六工具/60 用例的时间线仍不成立。** 六工具包含 Phase C 才启用的 `sim.run`（B:99），但 B 的 MVP 验收要求 60 个用例覆盖六个工具（B:269）且 B 先于 C。B1 应只测五个工具，`sim.run` 返回 `not_implemented` 或标为 B2/C 启用；60 例应拆成 B1 覆盖矩阵。
3. **【阻塞】ToolResult 字段与规则不一致。** B:110-112 的 ToolResult 没有 `untrusted` 字段，但 B:124 仍写“工具输出标记 `untrusted: true`”。应删除该句或把 `untrusted` 加回结构；否则 SDK 生成和测试会冲突。
4. **【阻塞】审计存储/锚定仍未定义。** B:308 要求“只追加本地审计日志（哈希链 + 每日锚定）”，但没有定义日志格式、文件位置、锚定目标（本地文件/SAP/外部）、`prev_hash` 迁移与 `user_version`；现有 `db.rs` 每次启动仍 `DROP TABLE IF EXISTS events/notes`（[db.rs](../../apps/desktop/src-tauri/src/db.rs):58-78）。这是 B 的安全证据基础，必须写清。
5. **【需修改】`Authorization.basis` 仍承载不了审批摘要。** `Authorization` 只有 `by/basis/scope/valid_until`（[action.rs](../../crates/poie-state/src/action.rs):213-218），`policy_version` 在 `Action` 上；B:307 说 Agent 来源写入 `Authorization.basis`、策略版本写入 `policy_version`，但 `ApprovalToken` 的 subject digest/ID 没有落到 SAP 记录。若审计日志是权威，应明确 token ID 与 `sap_record_id` 的关联方式；否则无法从 SAP 记录重建“谁批了什么”。
6. **【需修改】威胁表仍写“JSON Schema”。** B:297 的“工具参数注入 | JSON Schema + 固定白名单”与 B:103“不做 JSON Schema 求值器”矛盾；应改为“Rust 强类型 + deny_unknown_fields + 长度/枚举上限”。
7. **【需修改】`agent_run` 的 `policyProfile` 未定义。** B:216 输入 `prompt, policyProfile`，但没有 `policyProfile` 类型/取值/与 `HostPolicy` 的关系；`agent_status`/`agent_resume` 的 `RunState` 只有名字没有 schema 字段；应引用 PLATFORM 的 `poie.agent-run/0.1` 并给出字段。
8. **【需修改】Cargo/test-matrix 门禁未进 B 步骤。** B:335-340 没有 `poie-governance`/`poie-digest` 进根 `members`、`Cargo.lock`、`check-test-matrix`、`cargo-deny`、ADR 的步骤；PLATFORM:153-154 有要求，但 B 自己必须写。

**测试缺口**

- 缺 capability 签发/过期/参数变更/跨会话复用测试；缺 `ApprovalToken` 字段逐一变更后的 `approval_mismatch` 测试；缺暂停→重启→`interrupted`→取消/重试测试。
- 缺 provider 超大响应/非 JSON/超时/畸形 `tool_calls`/输出截断测试；缺 audit 哈希链、每日锚定、SAP 关联、DB 迁移测试。
- 缺 60 例的覆盖矩阵与 fixture schema 版本；缺 `sim.run` 在 B1 的 `not_implemented` 测试。

### 1.4 `phase-c-what-if.zh-CN.md`

**成立之处**

- Snapshot 已补 `nowS/seed/horizonS/maxDepth/maxActions/evidenceValidityS/assetTimelines/stockTimelines/storageLimits/stateId?`（C:53-63），并明确 C MVP 只针对静态 Base-01 演示快照，删除“live 不变”的假阳性断言（C:66、C:262）；这是本轮最好的修正。
- `PatchDto`（C:75-79）与领域 `Patch` 分离，`OutcomeKind` 分开 structural/timed 并加 caveat（C:100-106），`resultSha256` 使用 DIGEST `poie.variant/0.1`（C:118），`sapPreview` 形状与 A 对齐（C:113）。
- 新增 `plan_to_proposed`（C:184-190），明确不调用 `validate_plan_through_sap`、不写 Authorization、不 dispatch；`TimedScenario::with_storage` 与 `startMode=scenario-start`（C:194-199）解决了 timed/storage 的构造路径。
- 持久化、错误码、`ApprovalToken`、`Mode` host 状态都已补（C:182、C:203、C:209）。

**缺陷**

1. **【阻塞】依赖树仍漏 `poie-state` 与 `poie-digest`。** C:122-133 的 `poie-whatif` 依赖没有 `poie-state`（`plan_to_proposed` 需要 `ActionState`/`Action`），也没有 `poie-digest`（`resultSha256`）。PLATFORM:148 已列出 `state, viability, digest`，C 必须同步，否则实现时会临时加依赖/破坏 workspace。
2. **【阻塞】`PatchDto` 声称“与 Patch 一一对应”，但少了两个字段。** C:75-79 只有 11 项，而 `Patch` 有 13 项（含 `assetDegraded`、`assetCapabilityVerification`，[model.rs](../../crates/poie-ontology/src/model.rs):813-846）。应补齐或明确 MVP 不支持这两个字段并 fail-closed；否则“降级策略/能力验证”变体无法表达，且转换测试会漏字段。
3. **【阻塞】checkpoint 规格仍不完整，若按原文实现会破坏质量核算。** C:93 的 `Checkpoint {time_s, assets, stocks, batteries, external_flows, digest}` 缺 `initial_stocks`、`external_in_kg/out_kg/into_assets_kg`、`rules/fired_rules`、`queue/next_seq`、`log/violations` 与 model digest。C:199 允许“从固定场景起点重跑”，但 C:93 仍把 checkpoint 写成“必须先增加”的硬前提；应二选一：定义完整 checkpoint，或明确 MVP 只用 `startMode=scenario-start`，把 checkpoint 列为 C2/Optional。
4. **【阻塞】C:143 仍写“当前 live 模型与证据版本”。** C:66 已明确 C MVP 没有可变 live state，C:151 又改为 host model/evidence digest；C:143 的措辞会让实现者去找不存在的 live 投影。应统一为“当前 `SourceModel` 的 modelSha256/evidence digest”。
5. **【需修改】`evidenceVersion` 没有来源。** C:58 写入 `evidenceVersion`，但 D/PLATFORM 只定义 `modelSha256`/contracts/evidence manifest；C MVP 的静态证据版本从哪来（VEC report? scenario digests?）没有定义。应明确“MVP 固定为某个 digest/fingerprint，或删除该字段”。
6. **【需修改】`assetDelayS` 的语义不完整。** C:82/95 说“资产可用/修复整体推迟”，但没有定义作用于 `AssetTimeline` 的哪个字段、是否影响库存时间线、负值/超过 horizon 的行为。应给出转换公式与测试。
7. **【需修改】`stateId?` 的可序列化性。** C:58 允许 `stateId?`，但 `StateId` 只有 `Debug/PartialEq`（[projection.rs](../../crates/poie-state/src/projection.rs):60-66），没有 serde；应作为 DTO 字符串或删除。
8. **【需修改】C 的 Cargo/test-matrix 门禁未进步骤。** C:269-274 没有 `poie-whatif`/`poie-digest` 进 members、Cargo.lock、test-matrix、cargo-deny、ADR；PLATFORM:148/153 有要求。

**测试缺口**

- 缺 snapshot 全字段 round-trip、`StateId`/digest stale、per-mode `OutcomeKind` 域、`PatchDto->Patch` 逐字段转换、storage 上下界、compare 排序稳定性、`plan_to_proposed` 只产生 proposed、`ApprovalToken` 一次性测试。
- 缺 namespace/投影排除测试（C:149 说“未来需要仿真记录”才用 `Namespace::Simulation`，MVP 不写；测试应断言零记录，而不是只断言无“live”）。
- 缺并发/取消/超时、DB 迁移、审计、Playwright（浏览器预览 vs host）测试。

### 1.5 `phase-d-site-delivery.zh-CN.md`

**成立之处**

- D 已明确**不做模型/二进制导入**（D:20、D:35、D:181），把 bundle 限定为 JSON 元数据，删除 `files[]`/路径遍历面（D:82-83），并修正 artifact 为实际 `.app.zip`（D:75、D:189）；这是对上一版最危险的“导入模型但无消费方”问题的正确降级。
- `manifestSha256` 明确引用 DIGEST 的 `manifest without integrity`（D:84），`contracts.toml` 机器可读（D:88、D:182），`DeliveryState` 区分 manifest 来源与本地订阅（D:105），`DeliveryPlan/recall/rollback` 与 `approvalTokenId` 命令补齐（D:112-123、D:164-177）。
- `HostPolicy` 单一 loader、`remoteInference=false` 拒绝推理、`execution_gate_set/sandbox_*` 按 A0 修复（D:193-197）；诊断只用 audit 的 `recentErrors`（D:133），不再虚构错误表；SBOM/attestation 降为 O2（D:40、D:191）。
- 版本单一源、`release.json`、`check-release-manifest.py`、tag gate 写入 D:186-191。

**缺陷**

1. **【阻塞】D 与 PLATFORM 对 `ModelStore` 的取舍相反。** D:35 明确“不导入/切换模型文件；D 只记录 digest”，D:181 说 host 保存 `SourceModel`；但 PLATFORM:112-113 说“D 的 site bundle 增加 `ModelStore`：内置 fallback + imported bundle；优先级 imported > built-in；当前设计选择实现 ModelStore”。两者只能保留一个。若按 D，PLATFORM 必须删除 ModelStore/imported 优先级；若按 PLATFORM，D 必须恢复模型导入/重启加载的验收。当前状态无法实现。
2. **【阻塞】缺少运行中构建身份，manifest 的 app/commit/target 无法与二进制核对。** D 已不再要求 `host_info` 扩展；`host_info` 仍只有 `version/data_dir/db_path/sandbox_root/platform`（[lib.rs](../../apps/desktop/src-tauri/src/lib.rs):46-54）。D:203 的 channel/version/commit/target 来自 DeliveryState（SQLite），可被本地修改；站点无法证明“运行的是 manifest 声称的 build”。应增加 `host_info.build`（releaseId/commit/target/可选 build channel），或明确 DeliveryState 不是运行身份并降低验收。
3. **【阻塞】发布门禁仍没有落点。** D:190 说“tag release 依赖根 verify-all”，但 [ci.yml](../../.github/workflows/ci.yml):5-7 只在 `push.branches=[main]` 与 PR 上运行，tag push 不触发根 CI；[desktop-build.yml](../../.github/workflows/desktop-build.yml):183-186 的 release job 只 `needs: build`，PR paths 仍缺 `apps/desktop/tests/**` 与 `playwright.config.ts`（:20-31）。必须具体到 workflow trigger/needs/workflow_run，否则这条只是口号。
4. **【阻塞】`delivery_bundle_export/import` 的原子写/路径安全未落到 API。** D:168-177 只写“写入 sandbox/相对名、不跟随符号链接”，但现有 `sandbox::write` 用 `std::fs::write`、跟随符号链接、无临时目录/rename/fsync（[sandbox.rs](../../apps/desktop/src-tauri/src/sandbox.rs):9-48）。A0/PLATFORM:99 要求 no-follow + 原子写，但 D 没有定义新的 `sandbox_import`/`write_atomic` API 与失败回滚测试；D:237 断言“导入失败不改 DeliveryState”需要事务/临时文件实现。
5. **【需修改】`delivery_bundle_export` 输出是“bundle JSON（写入 sandbox）”，但 D 的 UI 还要“导出 bundle/诊断”到站点可携带介质。** 现有 `sandbox_write` 只允许 1 MiB UTF-8 文本（sandbox.rs:9、38-47），manifest+diagnostics 可能超限；也没有“选择导出目录/文件对话框”的 host 命令。应明确导出路径/大小上限，或 MVP 只导出到固定 sandbox 目录并提示操作员复制。
6. **【需修改】`redactions` 规则太粗。** D:139 只按字段名含 `key/token/secret/password/authorization` 替换；无法覆盖值级泄漏（如 endpoint 中的凭据、prompt 片段、路径）。“绝对路径细节”也没有定义 basename/相对化规则；应补值级扫描、长度/熵阈值与负例。
7. **【需修改】`delivery_plan` 的“可更新”与“不替换 app/模型”语义冲突。** D:26-27 要让 DeliveryPlan 解释“可更新/不可更新”，但 D:20/35 明确不安装/不替换 app/模型，唯一可“更新”的是 manifest/策略/证据引用。应在 UI/字段中把 `update` 改为 `adopt manifest`，避免操作员误以为会升级二进制。
8. **【需修改】Cargo/test-matrix/ADR 未进 D 步骤。** D:257-262 没有 `poie-delivery`/`poie-digest` 进 members、Cargo.lock、test-matrix、cargo-deny、ADR；PLATFORM:149/153 有要求。

**测试缺口**

- 缺 manifest 自哈希（排除 integrity）、规范化、篡改任一字段、unknown major/minor、artifact/target 不匹配、contracts/evidence 缺失测试。
- 缺 `delivery_*` 状态机、ApprovalToken 过期/重用、`HostPolicy` 强制、DB `user_version` 迁移/回滚、导入失败状态不变测试。
- 缺“运行身份 vs DeliveryState”测试（见缺陷 2）；缺 tag gate/PR paths 的 workflow 测试。

### 1.6 `PLATFORM.zh-CN.md`

**成立之处**

- 统一 schema 注册、`schemaVersion`、DTO/领域类型边界、错误包络、命令 scope、`SourceModel`、`ApprovalToken`、crate/依赖/工作区规则（PLATFORM:7-31、:33-44、:46-73、:75-100、:102-113、:115-136、:138-155），正是跨阶段最需要的“单一平台契约”。
- `ApprovalToken` 删除双人审批（PLATFORM:133），与 B/D 单操作员一致；`HostPolicy` 默认值（PLATFORM:100）为 D 提供强制点；`SourceModel` 固定 Base-01+degraded（PLATFORM:107）解决了 A/C/D 的“当前模型”歧义。

**缺陷**

1. **【阻塞】引用的 `docs/design/SCHEMAS.md` 不存在。** PLATFORM:9 说“统一注册表 `docs/design/SCHEMAS.md`（实现时可由 `scripts/check-schema-registry.py` 校验）”，但目录中没有 `SCHEMAS.md`，`scripts/` 中也没有 `check-schema-registry.py`。PLATFORM:15-24 只有一张内联表，缺少字段级 schema/兼容规则；A/B/C/D 的 `schemaVersion` 仍无唯一来源。必须创建该文件与检查器，或删除引用并以内联表为唯一注册表。
2. **【阻塞】窗口标签与实际代码不符。** PLATFORM:77 定义 `main`、`focus-federation`、`focus-execution`、`focus-evidence`、`settings`、`agent`；实际 `open_workspace_window` 创建的是 `poie-view-{view}`（[lib.rs](../../apps/desktop/src-tauri/src/lib.rs):120-140），没有 `focus-*`/`settings`/`agent` 窗口。scope 矩阵按这些标签实现会全部拒绝或全部放行；必须改为实际标签或明确新增窗口。
3. **【阻塞】`SCHEMAS.md` 的兼容规则与 A/C/D 的 `schemaVersion` 不一致。** PLATFORM:28 说“未知 major 拒绝、未知 minor 可接受”；A 的 `OntologyCatalog` 只有 `schema` 字符串，C 的 `VariantResult` 有 `schemaVersion`，D 的 manifest 用 `schema`；缺少统一的 major/minor 解析函数与测试。应在 PLATFORM 定义 `parse_schema`/`compatible`，并让各 DTO 复用。
4. **【需修改】DTO 命名与 A/C 实际结构不一致。** PLATFORM:40 使用 `ObjectInstanceDto`、`LinkInstanceDto`、`ActionTypeDto`、`SapPreviewDto`，A:173-215 使用 `ObjectInstance`、`LinkInstance`、`ActionType`、`SapPreview`；PLATFORM:41 使用 `PatchDto`，C:75 已用 `PatchDto`（一致），但 A 的命名不一致。SDK 生成器会按 A 的名字生成，PLATFORM 的“统一 DTO”名只是文档；应统一命名或明确 A 的类型就是 DTO。
5. **【需修改】`poie-governance` 依赖 `poie-plan`，但 B 的工具目录没有 `plan.preview_sap`。** PLATFORM:147 列 `poie-plan`；B:94-99 六工具只有 `plan.propose/action.draft`，`action.draft = plan.propose + plan.preview_sap`（B:309）。这不算错，但应在 B 的 Tool Catalog 中体现 `plan.preview_sap` 的只读调用，避免实现者直接调 `validate_plan_through_sap`。

### 1.7 三份研究备忘

**Palantir 模式**

- 强项：事实/推断分离（:5、:10）、采纳/改造/拒绝与 P1–P9（:143-164）、“像 Palantir 的演示、丢掉物理契约”的风险（:171）仍是最好的取舍框架。
- 问题：P2“无 CRUD/写接口”（:148）与 D 配置写入冲突；P5“promote 只产生 proposed”（:151）现在有 C 的 `plan_to_proposed` 支撑，但仍需在验收中引用；A 的“外部消费者仅凭 Catalog/SDK”（:91）已过时，应改为桌面内。外部事实（Apollo air-gapped 等）仍未在本仓库独立验证，只能作方向依据。

**Agent 治理**

- 强项：G1–G7、威胁表、AGT-T1–T13、Rule of Two/τ-bench/PROV-O 引用扎实（:71-82、:190-216）；T0–T3 信任级与 `as_of/trust/evidenceRefs` 已被 B 采纳（B:110-115），这是本轮最好的吸收。
- 问题：备忘仍主张七个工具并含 `dispatch.request`（:98），B 已决定六个且无 dispatch（B:94-101）；备忘的审批词表 `read_only/plan/simulate/dispatch`（:107-116）与 B 的 effect/approval 两列（B:92-99）需要一张映射表；`Gate::dispatch`（:76）实际是 `ExecutionGate::dispatch`（[gate.rs](../../crates/poie-exec/src/gate.rs):149）。应把备忘标为“研究提案”，不要与 B 的冻结决定并列为规范。

**仓库差距审计**

- 强项：路径/行号最具体，正确指出 `execution_gate_set` 缺 auth、`db.rs` 启动 DROP、测试矩阵计数、DMG 与 PR paths（:85-99、:294-300）。
- 问题：内部陈旧：:9 决定 `poie-governance`，但 :104/:113 仍写 `poie-agent`；:9 用 `auto` 词表，:116 又用 `auto_read`；:175 的 checkpoint 仍缺质量核算/队列字段；:177 的 `validate_plan_through_sap -> proposed` 与实际授权行为不符；:181 把 `ensure_live_dispatch` 当成写入门（它只是派发门）；:283-284 的测试计数已过时。应更新或标注“以冻结设计为准”。

## 2. 跨阶段问题

1. **【阻塞】冻结与验收不可追溯。** README:103 说快照 `4a90edc`，但 HEAD 是 `f75c31d`；本报告只能对 23:00 快照负责。设计门禁要求冻结后重跑，README 的冻结记录必须先对齐。
2. **【阻塞】Digest 只完成了规范，没有完成接口。** A 的 `OntologyCatalog` 使用 `catalog_sha256` 而调用方使用 `catalog_digest`；C 的依赖缺 `poie-digest`；D 的 manifest 已引用 DIGEST，但 A/C 的 DTO 没有统一 `sha256/fingerprint` 字段命名；`poie-digest` 未进根 `members`/测试矩阵。规范存在不等于可消费。
3. **【阻塞】“当前模型”有两个版本。** PLATFORM:107 定义 `SourceModel = Base-01 + degraded`；D:35/181 只记录 digest、不导入；PLATFORM:112-113 又要 D 实现 `ModelStore`（imported > built-in）。A/C 依赖 `SourceModel`，D 依赖 `ModelStore`，两者必须合并成一个决定。
4. **【阻塞】SAP §7.4 的仿真来源仍无法写入 Action。** C 的 `plan_to_proposed`（C:189）只创建 proposed，但 `Action` 没有 `simulation_branch/model_version` 字段（[action.rs](../../crates/poie-state/src/action.rs):222-242）。SAP §7.4 要求从仿真结果生成的现场动作引用仿真分支与模型版本；需字段扩展 + ADR + SAP-T 用例，或用可机器解析的编码规则。B 的 Agent 提案同样受影响。
5. **【阻塞】对象 ID/EvidenceRef 仍无统一语法。** A 用 `typeId + id`（A:241-242），B 的 fixture 用 `service:potable-water`（B:246-251），C 用 `pump-p1`（C:42），D 的 evidence 用 `id + fingerprint + sha256`（D:72）。需要统一 canonical object ID（类型前缀/大小写/转义）与 `EvidenceRef`（id/kind/fingerprint/sha256/asOf/source），否则 B 的 citation、C 的 evidenceRefs、D 的 manifest 无法互相解析。
6. **【阻塞】审批原语已抽象，但落点仍不完整。** PLATFORM:120-129 的 `ApprovalToken` 有 `tool_or_action`/`subject_digest`，B/C/D 都引用；但 `Authorization` 没有 token ID/subject digest 字段（action.rs:213-218），`sap_record_id` 只在 B 的审计里出现。需要明确 token 与 SAP 记录/审计记录的关联，否则审批不可重建。
7. **【需修改】浏览器预览 vs host-only 的测试策略仍缺失。** Phase A/B/C/D 的 Playwright 验收都要求 host 行为，但 Playwright 只跑浏览器预览（playwright.config.ts:4-6），现有 `domain.ts` 有 mock/fallback（domain.ts:189）。需要明确哪些用例用 mock，哪些用 Rust host 集成测试或 E2E；否则验收是假绿。
8. **【需修改】新 crate/CI 门禁仍未进各阶段首个 PR。** PLATFORM:153-154 要求每个 PR 跑 fmt/clippy/test/doc/cargo-deny/check-test-matrix/check-docs，并给新依赖写 ADR；A/B/C/D 的实施顺序都没有这一步，`poie-digest/poie-objects/poie-sdkgen/poie-governance/poie-whatif/poie-delivery` 会立即使 `check-test-matrix.py` 计数失败。
9. **【需修改】P2/P5 的措辞需要与 D/C 对齐。** P2“无 CRUD/写接口”与 D 配置写冲突；P5“promote 只产生 proposed”现在有 C 的 `plan_to_proposed`，但 README:70 仍用旧句；应改为“promote 只经 `plan_to_proposed` 产生 proposed，不 authorize/dispatch”。
10. **【需修改】代码注释与契约词表漂移。** `poie-plan/src/sap.rs:1-8,87` 写“six dispatch checks”，实际 `review_dispatch` 是七项（review.rs:1-12）；`poie-state/src/lib.rs:31` 仍说 M12 未实现。设计文档说七项是对的，但代码注释会误导 B/D。

## 3. REQUIRED（开发前必须修复）与 OPTIONAL

### REQUIRED（按优先级）

| 编号 | 必须修改 | 涉及 |
| --- | --- | --- |
| R1 | 冻结当前所有设计文档到单一 commit，更新 README:103 的 `4a90edc`；本报告按冻结版重跑行号 | 全部 |
| R2 | A：统一 `catalog_sha256`/`catalog_digest`/`catalogDigest` 为同一字段；统一 `SapPreview` 字段为 `rejections`；把 `plan.preview_sap` 输入改为 Plan/host 预计算；补齐 `ActionType`/`FunctionType`/`Predicate` 字段或明确 PLATFORM 定义 | A |
| R3 | A：把 A0 的 synthetic preview `scope` 改为 `String`（或 ADR 扩展 `Authorization`）；同步修改 `execution.rs` 的 `ready/authorized` 断言并增加 synthetic 不可 dispatch 负例；定义对象图布局与 Playwright mock 的具体 fixture | A |
| R4 | DIGEST：修正 `ModelCanonical` 的 `description` 等字段来源；补 `-0.0`/浮点/字符串数组/键序固定测试向量；把 `poie-digest` 进 members/测试矩阵/Cargo.lock | A/C/D |
| R5 | B：删除 B:345 的“存内存/重启即取消”，与 `agent_runs` 持久化对齐；B1 只测五工具、`sim.run` 标 C/not_implemented；修正 `untrusted` 与 JSON Schema 两处矛盾；定义审计日志格式/锚定/`user_version` | B |
| R6 | C：依赖树补 `poie-state`/`poie-digest`；`PatchDto` 补齐 `assetDegraded/assetCapabilityVerification` 或明确 fail-closed；checkpoint 定义完整或降级为 `scenario-start`；统一 C:143 的 live 措辞；定义 `assetDelayS`/`evidenceVersion` | C |
| R7 | D：解决 D:35 与 PLATFORM:112-113 的 ModelStore 矛盾；增加 `host_info.build`（releaseId/commit/target）或明确运行身份来源；把 tag gate/PR paths 写成具体 workflow；定义 `sandbox_import` 原子写/失败回滚与导出路径/大小上限 | D |
| R8 | PLATFORM：创建 `docs/design/SCHEMAS.md` 与 `scripts/check-schema-registry.py`（或删除引用）；把 `focus-*`/`settings`/`agent` 窗口标签改为实际 `poie-view-*` 或定义新窗口；统一 A 的 DTO 命名 | PLATFORM |
| R9 | 跨阶段：统一 canonical object ID 与 `EvidenceRef`；把 `ApprovalToken` 与 `Authorization`/`sap_record_id` 的关联写清；给 `Action` 增加 `simulation_branch/model_version` 或可机器解析的编码规则（SAP §7.4） | A/B/C/D |
| R10 | 更新三份研究备忘：`poie-agent`/`auto_read`/七工具/`dispatch.request`/`ensure_live_dispatch` 等陈旧表述；统一“研究提案 vs 冻结决定”；更新测试计数 | 研究 |
| R11 | 把新 crate/依赖/CI/文档门禁写入每个阶段首个 PR：根 `members`、Cargo.lock、`check-test-matrix`、`cargo-deny`、`rustdoc -D warnings`、ADR、`check-docs` | 全部 |

### OPTIONAL（可延后/可降级）

- O1 `pass^k` 已限定现场真实 provider；CI 继续用确定性用例与覆盖矩阵（已采纳，保持）。
- O2 SBOM/attestation、Ed25519 签名、自动更新器延后；SHA-256 + 明确“未认证”是可接受的 MVP。
- O3 DMG/portable exe：当前 D 已改为 `.app.zip`，保持“不生成 DMG”即可。
- O4 对象图确定性布局算法；MVP 可用稳定布局，不必现在设计图数据库。
- O5 VEC 指标、跨站点 what-if、bundle 压缩/差分、多租户留到后续。
- O6 生成 TS 之外再生成 JSON Schema/OpenAPI，可后续做。
- O7 `recentErrors` 先复用 audit 事件（D 已如此），保持。
- O8 capability 用不可伪造的内部句柄 + 参数摘要；不追求跨进程签名。
- O9 代码注释 “six dispatch checks” 与 `poie-state/src/lib.rs:31` 的 M12 过时描述，可随 A 的清理 PR 一起修。
- O10 若 C 不做 checkpoint，把 `TimedScenario::with_storage`/`startMode=scenario-start` 写成唯一路径即可。

## 4. “可以开始 Phase A 了吗？”检查清单

- [ ] **R1** 设计文档已提交到单一 commit；README:103 的冻结快照与实际 HEAD 一致。
- [ ] **R2** `catalog_sha256`/`catalog_digest`/`catalogDigest` 已统一；`SapPreview.rejections`、`plan.preview_sap` 输入、`ActionType`/`FunctionType`/`Predicate` 字段已定。
- [ ] **R3** A0 的 synthetic preview `scope` 类型已定；`execution.rs` 的 `ready/authorized` 断言更新与 synthetic 不可 dispatch 负例已定；对象图布局与 Playwright mock fixture 已定。
- [ ] **R4** `ModelCanonical` 字段来源、浮点/数组/负零规则、固定测试向量已定；`poie-digest` 进 members/测试矩阵。
- [ ] **R5** Phase A 的 Catalog/SapPreview/ActionPreview 全部有 JSON 快照；六个旧视图与 Playwright/浏览器预览策略已定。
- [ ] **R6** 对象图节点/边从 Catalog 派生（含确定性布局）已定；`GRAPH_NODES` 删除/替换路径已定。
- [ ] **R7** 外部消费者文案已改为“桌面内 SDK”；README:80 的退出条件同步。
- [ ] **R8** 错误包络/错误码/`schemaVersion`/`SCHEMAS.md` 已可执行（PLATFORM）。
- [ ] **R9** 新 crate 进根 `members`、根/桌面 `Cargo.lock` 更新计划、`check-test-matrix` 计数、`cargo-deny` 许可结论已定。
- [ ] **R10** A 的 `SourceModel`（Base-01+degraded）与 host `Arc<Model>`/digest 已写入 A0；C/D 不再各自定义“live”/ModelStore。

**结论：当前为“否”。** R1–R4 未完成前，Phase A 不应进入编码；R5–R11 是 B/C/D 的开工前置。建议先做一个“A0 设计冻结 PR”：只改文档、补 `SCHEMAS.md`/测试向量与 CI 门禁，不写业务代码，再重新跑本校验。

## 5. 研究遗漏与过度声明

- **过度声明 1（README/A）**：README:80 仍写“外部消费者仅凭 Catalog/SDK”，A 已改为桌面内（A:54、:327）；必须同步，否则 A 的验收标准与设计不一致。
- **过度声明 2（D）**：D:26-27 的 DeliveryPlan“可更新/不可更新”容易被读成升级 app/模型；D:20/35 明确不替换 app/模型。应把字段/文案改为“可采纳 manifest/策略”，避免现场误解。
- **遗漏 1（D）**：D 没有运行中构建身份（`host_info` 仍只有版本/路径），manifest 的 app/commit/target 无法与二进制核对；这是站点交付的信任根之一，不能只依赖可修改的 SQLite DeliveryState。
- **遗漏 2（B）**：B 的审计日志“哈希链 + 每日锚定”没有格式/存储/锚定目标；即使 UI 显示审计，也无法证明未被本地改写。应先定义日志文件/锚点，再宣称可重建。
- **遗漏 3（C）**：C 的 `evidenceVersion`/`stateId` 没有来源；静态 Base-01 演示快照的“证据版本”应是哪个文件/报告，未定义。
- **过程问题**：文档在本次校验期间被并发改写（22:40–23:00），当前 HEAD 为 `f75c31d`、README 冻结记录仍停在 `4a90edc`；本报告只对 23:00 快照负责。在 R1 完成前，不应引用任何“已校验通过”。

VERDICT: CHANGES_REQUIRED
