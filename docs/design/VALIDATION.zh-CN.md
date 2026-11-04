# Poieweave Phase A–D 设计独立校验（最终）

- 校验对象：[README](README.zh-CN.md)、[Phase A](phase-a-ontology-product.zh-CN.md)、[Phase B](phase-b-agent-governance.zh-CN.md)、[Phase C](phase-c-what-if.zh-CN.md)、[Phase D](phase-d-site-delivery.zh-CN.md)、[Digest](DIGEST.md)、[平台契约](PLATFORM.zh-CN.md)、[Schema 注册表](SCHEMAS.md)，以及 [Palantir](research/palantir-patterns.zh-CN.md)、[Agent 治理](research/agent-governance.zh-CN.md)、[差距审计](research/repo-gap-audit.zh-CN.md)。
- 代码基线：分支 `design/phase-a-d`，HEAD `55bd48d`；设计文档已入库，工作区仅有本校验文件被修改。
- 文档快照：2026-11-04 23:08（本地）。校验期间文档经历多轮并发修订；本报告按 `55bd48d` 的行号，最终结论应在下一次 commit 后重跑。
- 方法：逐条对照 PO/SAP/VEC、`crates/poie-*`、`apps/desktop/src-tauri`、`apps/desktop/src`、根 workspace、CI、`scripts/` 与 ADR；`python3 scripts/check-docs.py` 通过。未运行完整 `cargo test`，未修改应用源码、设计文档或 git 状态，仅重写本文件。

## 0. 总判定

**PASS WITH REQUIRED CHANGES。** 本轮修订基本闭合了前几轮的主要阻塞：A0 宿主鉴权前置、`DIGEST.md`、`PLATFORM.zh-CN.md`、`SCHEMAS.md`、只读 `preview_plan_through_sap`、B 的 `agent_*`/`RunState`/per-tool approval digest/审计日志、C 的 `OutcomeKind`/`plan_to_proposed`/静态演示快照、D 的“不导入模型/不做 DMG”/`host_info.build`/原子写都已落地。

剩余缺陷数量少但性质明确：A0 的 synthetic preview 授权仍把 `scope` 写成数组，与 `Authorization.scope: String` 不符；C 的 `evidenceSha256` 没有来源，且 `plan_to_proposed` 之后没有把仿真分支/模型写入 `Authorization.basis`（SAP §7.4）；README P5 仍要求“live digest 不变”，与 C 的静态演示快照不一致；B 的 `untrusted` 字段/依赖还有小漂移。修复这些文档级问题后，可进入 Phase A。

## 1. 逐文档校验

### 1.1 `README.zh-CN.md`

**成立之处**

- 跨阶段不变式、阶段退出条件与 PO/SAP/VEC 一致；P2 已限定为“物理/运营变更都是动作，配置/交付写入受 ApprovalToken/HostPolicy 约束”（README:67），不再与 D 的配置写冲突。
- A 的退出条件已改为“桌面内消费者仅凭 Catalog/SDK”（README:80），与 A:370-372 一致；R1 也改为“以分支 HEAD 作为校验快照”（README:103），不再写固定旧 commit。
- README:41、99-120 的 R1–R18 修订记录让读者能追踪每项设计修正。

**缺陷**

1. **【阻塞】P5 的门禁仍要求“variant 前后 live digest 不变”**（README:70）。C 已明确 MVP 只针对静态 Base-01 演示快照、没有可变 live state，并删除了“live 不变”的假阳性断言（C:66、C:264）。P5 应改为“C 不写 ActionLedger/execution/federation；SourceModel digest 不变”，否则 C 的验收无法按 P5 执行。
2. **【需修改】P6 仍用“bundle 绑定 model/contracts/evidence”**，而 D 已明确 bundle 是“元数据、不含模型文件、不替换 app/模型”（D:20、D:35、D:82-83）。应写成“manifest 记录 model/contracts/evidence digest，导入只更新交付清单/策略/证据引用”，避免现场把它理解为模型包。
3. **【需修改】“四份设计文档”**（README:124）已过时；现在还有 DIGEST/PLATFORM/SCHEMAS 与三份研究备忘，应更新清单。

### 1.2 `phase-a-ontology-product.zh-CN.md` 与 `DIGEST.md`

**成立之处**

- A0 已写入正文第 4 节并成为实施第 0 步（A:36-46、A:380），覆盖 gate/sandbox 鉴权、命令 scope、错误包络、demo 授权替换；A:43 已把 `execution.rs` 与 `federation_view.rs` 两处硬编码授权纳入替换。
- `OntologyCatalog` 现在有 `catalog_digest`（A:217），`ObjectPage`/host 使用同名（A:226、A:288）；`SapPreview` 统一为 `rejections`（A:242、A:270）；`plan.preview_sap` 输入已改为“Plan object（host 持有）或 planRef”（A:125）。
- `ActionType`、`FunctionType`、`Predicate`、`ObjectType`、`ObjectInstance`、`ObjectDetail`、`EvidenceRef` 等 DTO 已给出字段（A:175-253）；A:270-273 的只读 `preview_plan_through_sap(&Plan, &AssumedContext)` 明确不写 ledger、不创建 Authorization、不产生 transition，并保留 `assumed/failed/unknown` 逐项状态。
- `DIGEST.md` 已把 `-0.0` 归一化、fingerprint/SHA-256 分离、manifest 排除 integrity 写清（DIGEST:27-50、:58、:86-92）；`ModelCanonical` 已删除 `Asset.description`（DIGEST:71-72），与运行时 `Asset` 字段一致。

**缺陷**

1. **【阻塞】A0 的 synthetic preview 授权类型不匹配。** A:43 写 `scope=["preview"]`（数组），但 `Authorization.scope` 是 `String`（[action.rs](../../crates/poie-state/src/action.rs):213-218）。实现时要么改为 `scope="preview"`，要么扩展 `Authorization` 把 scope 设为集合（后者是 SAP 契约变更，需 ADR + SAP-T 用例）。这是 Phase A 的第一个编译级阻塞。
2. **【需修改】A0 的测试预期未写全。** A:43 要求替换 demo 授权并断言“不能 dispatch”，但 `execution.rs` 的测试仍断言 `review_status=="ready"`、0 rejections、全部 `authorized`（[execution.rs](../../apps/desktop/src-tauri/src/execution.rs):181-190）；synthetic preview 替换后必须同步修改这些断言，并新增“未登录不生成授权”“synthetic 不可 dispatch”的负例，否则 A0 的验收无法通过。
3. **【需修改】对象图的布局/边派生仍不具体。** A:314 要求 `CapabilityGraph` 节点来自 Catalog，A:360 已加“删除 `GRAPH_NODES` 后仍能渲染”的测试；但现有组件用硬编码 x/y 与索引边（[App.tsx](../../apps/desktop/src/App.tsx):80-99、:532-536），Catalog 只给类型/链接，没有布局函数。至少应给出确定性布局规则（或保留字段坐标的映射），否则 A:360 的测试只能证明“能渲染”，不能证明“来自 Catalog”。
4. **【需修改】`PropertyValue.value` 与 `PropertyDef.kind` 的边界未定义。** A:175-185 允许 `serde_json::Value`，但未规定金额/数量/枚举的编码、`unit` 与 quantity 的关系，以及 `source=model|declared|observed` 与 `asOf` 的强制组合；SDK 生成器无法据此生成强类型。
5. **【需修改】A 的错误码表未完全引用 PLATFORM。** A:290 已列出完整集合，但应与 PLATFORM:56-71 保持单一来源，避免新增 `approval_*`/`bundle_*` 时再漂移。

**测试缺口**

- 缺 Catalog 逐字段黄金 fixture（属性、单位、端点类型、排序）、`catalog_digest` 排除自身字段/键序/浮点测试、cursor/limit/未知 schema 的 host 边界测试。
- 缺 `SapPreview` 的 assumed/failed/unknown 三分支、`planRef` 不存在、synthetic preview 不可 dispatch 的测试。
- 缺浏览器预览 mock Catalog 与 host JSON 快照（A:383 只写策略，未给 fixture）。

### 1.3 `phase-b-agent-governance.zh-CN.md`

**成立之处**

- Host API 已完整：`agent_run/status/approve/reject/cancel/resume`，`RunState` 持久化到 `agent_runs.state_json`，重启标记 `interrupted`，`user_version` 迁移（B:212-233）；上一版“暂停后无法继续”的阻塞已解决。
- per-tool approval digest 已按 `plan.propose/action.draft/sim.run/pending dispatch` 定义字段并引用 DIGEST（B:171-181）；`ApprovalToken` 走 PLATFORM §6，删除双人审批（B:182）。
- ToolResult 增加 `evidenceRefs/asOf/sourceClass/trustProfile`（B:110-115），T0–T3 信任级与“工具输出不进入指令位”已写成不变量；不做通用 JSON Schema 求值器（B:103），避免新依赖。
- B1 已拆成 45 个确定性用例（五工具，`sim.run` 标 `not_implemented`），C 阶段补到 60+（B:269）；pass^k 只用于现场真实 provider（B:260-261）；审计已定义为 `app_data/audit/agent.log` + `anchor.json` + SQLite 索引（B:318）。

**缺陷**

1. **【阻塞】`untrusted` 字段/规则仍有漂移。** ToolResult 结构（B:110-112）没有 `untrusted` 字段，但威胁表仍写“工具输出标记 untrusted”（B:303）。语义上可用 `sourceClass=T2/T3` 表达，但 SDK/测试会寻找字段；应删除该句或把 `untrusted` 作为派生布尔加入 DTO。
2. **【需修改】`poie-governance` 的依赖树没有 `poie-digest`。** B:56-60 只列 objects/plan/ontology/state；但 B:171-181 的 approval digest 使用 DIGEST。PLATFORM:147 已列 `poie-digest`，B 必须同步，否则实现时会临时加依赖。
3. **【需修改】`agent_run` 的 `policyProfile` 未定义。** B:216 输入 `prompt, policyProfile`，但没有取值（analyst/dispatcher? 自定义?）与 `HostPolicy.agentScopes` 的关系；应在 B 或 PLATFORM 定义枚举/默认值。
4. **【需修改】`RunState` 只有表结构，没有字段级 schema。** B:227 定义 `state_json`，B:223 说 `poie.agent-run/0.1`，但未给 `RunState` 的字段（messages/toolCalls/pendingApproval/budget/step）。PLATFORM:20 只登记 schema 名，SDK 无法生成。
5. **【需修改】`agent_resume` 与审批 token 的绑定未写清。** B:221 说“仅 approved 后”，但 `ApprovalToken.used`、`run_id`、`agent_resume(runId, approvalTokenId)` 的校验顺序、过期/重放路径没有测试矩阵；应在 B:225-233 补状态机。

**测试缺口**

- 缺 capability 签发/过期/参数变更/跨会话复用测试；缺 `ApprovalToken` 字段逐一变更后的 `approval_mismatch` 测试；缺暂停→重启→`interrupted`→取消/重试测试。
- 缺 provider 超大响应/非 JSON/超时/畸形 `tool_calls`/输出截断测试；缺 audit 断链/锚点篡改/按行重建测试。
- 缺 `RunState` schema round-trip 与 45+60 用例覆盖矩阵。

### 1.4 `phase-c-what-if.zh-CN.md`

**成立之处**

- Snapshot 已覆盖 `modelSha256/evidenceSha256/stateId?/nowS/seed/horizonS/maxDepth/maxActions/assetTimelines/stockTimelines/storageLimits`（C:53-63）；C:66 明确 MVP 只针对静态 Base-01 演示快照，不再声称可变 live/stale；C:264 改为“无新增 ledger/execution/federation 记录”。
- `PatchDto` 已补齐 `assetDegraded/assetCapabilityVerification`（C:75-79）；`OutcomeKind` 分离 structural/timed 并带 caveat（C:100-106）；`resultSha256` 使用 DIGEST `poie.variant/0.1`（C:118）；`sapPreview` 与 A 的 `rejections` 对齐（C:113）。
- `plan_to_proposed` 明确只创建 `Proposed`、不写 Authorization、不 dispatch（C:191）；timed/storage 使用 `startMode=scenario-start`，完整 checkpoint 降为 C2（C:93、C:199-201）；依赖已补 `poie-state`/`poie-digest`（C:127-128）。
- `assetDelayS` 已有明确公式：`AssetTimeline` 状态变更 `at += N`，N≥0 且 ≤horizon，不改变库存时间线（C:95）；`stateId` 作为 DTO 字符串（C:58）。

**缺陷**

1. **【阻塞】`evidenceSha256` 没有来源。** C:58/145/153 把它作为 stale/promote 的比较键，但 PLATFORM §5 只定义 `SourceModel` 与 `model_digest`，没有定义 evidence digest 的输入（是 `experiments/vec-base01/report.toml`、`scenarios/base-01-timed/digests.toml`，还是 manifest.evidence？），D:72 又用 `fingerprint+sha256` 两种形式。必须定义：由 host 从哪个文件/字段计算、算法、基准时间；否则 `stale_snapshot` 不可测试。
2. **【阻塞】`plan_to_proposed` 之后没有把仿真分支/模型写入 `Authorization.basis`。** PLATFORM:133-139 要求从 What-if 产生的 action 在 `Authorization.basis` 编码 `simulation=<branchId>;model=<modelSha256>`（或结构化字段），缺失时 `not_promotable`；C:191-192 只说 `plan_to_proposed` 后交给现有 SAP 验证命令，没有说明由谁写 basis、`ProposedActions` 是否携带 branch/model。B/C 的 traceability 依赖这一点，必须补到 C 的 PromotionResult/`plan_to_proposed` 契约。
3. **【需修改】`variant_run` 的 `mode` 与 `OutcomeKind` 对应关系只写了类型，没有写允许域。** C:100-106 定义 structural/timed 可空，C:164 的 `mode` 有 structure/plan/timed；应明确 plan 模式是否必须同时有 structural 和 timed（或 plan 的 `timed=null`），避免 UI 对 `null` 的解释不一致。
4. **【需修改】`whatif_variant_delete` 未要求审计/ApprovalToken。** C:167 只写 `ok`，但 C:283 说“清理只能显式操作并留审计”；应把 delete 纳入 ApprovalToken/审计，或明确仅删定义、不删结果。
5. **【需修改】`evidenceRefs` 的填充规则未定义。** C:114 有 `evidenceRefs[]`，但没有说明每个 variant 结果应引用哪些（snapshot、plan fingerprint、SAP preview、VEC report）；D 的 manifest.evidence 需要同一结构，B 的 citation 也复用；应给出最小集合与去重/排序规则。

**测试缺口**

- 缺 `evidenceSha256` 来源/变化→`stale_snapshot` 测试；缺 `plan_to_proposed` 后 basis 编码/解析/缺失 `not_promotable` 测试。
- 缺 snapshot 全字段 round-trip、`PatchDto->Patch` 逐字段转换、storage 上下界、compare 排序稳定性、ApprovalToken 一次性测试。
- 缺 Playwright：浏览器预览 vs host 的 Sim 模式测试策略（C:256 只写“未登录/浏览器预览禁用”）。

### 1.5 `phase-d-site-delivery.zh-CN.md`、`PLATFORM.zh-CN.md` 与 `SCHEMAS.md`

**成立之处**

- D 已明确不做模型/二进制导入（D:20、D:35、D:112 与 PLATFORM:112 一致），bundle 是 JSON 元数据，无 `files[]`/路径遍历面（D:82-83）；artifact 改为实际 `.app.zip`（D:75、D:189）；`manifestSha256` 引用 DIGEST 的排除规则（D:84）。
- `host_info` 增加编译期 `build: { releaseId, commit, target, channel? }`（D:87），解决“manifest 声称的 build 与运行二进制不一致”；tag gate 明确 `workflow_run`/`needs: [build, verify]`（D:202）；`write_atomic` 定义临时文件/fsync/rename/父目录 fsync（D:182-188）。
- `DeliveryState` 区分 manifest 来源与本地订阅（D:105）；`DeliveryPlan` 用 `adoptManifest` 避免“自动升级”误解（D:121）；`recall` 本地列表+审计（D:122）；`HostPolicy` 单 loader（D:193-197）。
- `SCHEMAS.md` 已建立 schema 表与 `parse/compatible` 规则（SCHEMAS:1-40）；PLATFORM 的窗口标签已改为实际 `poie-view-*`（PLATFORM:77-90）；`ApprovalToken` 统一 B/C/D（PLATFORM:141-155）。

**缺陷**

1. **【需修改】`SCHEMAS.md` 声明的检查器不存在。** SCHEMAS:1-3 与 PLATFORM:9 说 `scripts/check-schema-registry.py` 在 A0 加入 CI，但 `scripts/` 中没有该文件；A0 的实施步骤 A:380 也没有列出它。应在 A0 加入脚本与失败用例，或删除“实现时可由检查器校验”的承诺。
2. **【需修改】`DeliveryPlan` 的 `actions[].requiresApproval` 与 `ApprovalToken` 的关系未定义。** D:112-117 有 `requiresApproval`，D:164-177 的命令都接收 `approvalTokenId`；但没有说明 `requiresApproval=true` 时 UI 如何创建/展示 token、哪些 action 可自动（`adoptManifest`?）。
3. **【需修改】`diagnostics.redactions` 的规则仍偏字段名。** D:139 只按字段名含 `key/token/secret/password/authorization` 替换；对 endpoint 中嵌入的凭据、prompt 片段、路径 basename 没有值级规则。应补长度/熵阈值或明确“只做字段级白名单，不做内容扫描”，避免过度承诺。
4. **【需修改】`delivery_bundle_export` 上限与 sandbox 现有 1 MiB 文本限制未对齐。** D:168 说 bundle JSON 上限 256 KiB、总大小 ≤1 MiB（D:188），但现有 `sandbox::write` 只接受 1 MiB UTF-8 文本（[sandbox.rs](../../apps/desktop/src-tauri/src/sandbox.rs):9、38-47）；`write_atomic` 是新增 API，必须同步确认 sandbox 路径/大小检查复用它，而不是继续用旧 `sandbox_write`。

**测试缺口**

- 缺 manifest 自哈希/篡改/unknown major/minor/artifact-target 不匹配/contracts-evidence 缺失测试。
- 缺 `delivery_*` 状态机、ApprovalToken 过期/重用、`HostPolicy` 强制、DB `user_version` 迁移/回滚、`write_atomic` 崩溃/磁盘满测试。
- 缺 `host_info.build` 与 manifest 的交叉核对测试；缺 tag gate/PR paths 的 workflow 测试。

### 1.6 三份研究备忘

**Palantir 模式**

- 强项：事实/推断分离、采纳/改造/拒绝、P1–P9 与“像 Palantir 的演示、丢掉物理契约”的风险仍是最好的取舍框架。
- 问题：P2/P5 已部分过时（README 已改 P2；P5 仍写 live digest）；A 的“外部消费者”已改为桌面内，但备忘 :91 仍写外部；外部事实（Apollo air-gapped 等）未在本仓库独立验证，只能作方向依据。

**Agent 治理**

- 强项：G1–G7、威胁表、AGT-T1–T13、Rule of Two/τ-bench/PROV-O 引用扎实；T0–T3 与 `as_of/trust/evidenceRefs` 已被 B 采纳。
- 问题：备忘仍主张七个工具并含 `dispatch.request`（:98），B 已决定六个且无 dispatch（B:94-101）；备忘的 effect/approval 词表与 B 的两列需要映射表；`Gate::dispatch` 实际是 `ExecutionGate::dispatch`（[gate.rs](../../crates/poie-exec/src/gate.rs):149）。应标注“研究提案”，不要与冻结决定并列。

**仓库差距审计**

- 强项：路径/行号最具体，正确指出 `execution_gate_set` 缺 auth、`db.rs` 启动 DROP、测试矩阵计数、DMG/PR paths。
- 问题：仍写 `poie-agent`/`auto_read`（:104/:113/:116），与 B 的 `poie-governance`/`auto` 不一致；:175 的 checkpoint 旧字段、:177 的 `validate_plan_through_sap -> proposed`、:181 的 `ensure_live_dispatch` 作为写入门、:283-284 的测试计数均已过时，应更新或标注“以 55bd48d 为准”。

## 2. 跨阶段问题

1. **【阻塞】README P5 与 C 的静态演示快照冲突。** README:70 要求“variant 前后 live digest 不变”，C:66/264 已明确没有可变 live state；二者必须统一为“C 不写 ledger/execution/federation，SourceModel digest 不变”。
2. **【阻塞】SAP §7.4 的仿真来源在 C 的 promote 路径中断。** PLATFORM:133-139 选择 `Authorization.basis` 编码 `simulation=<branchId>;model=<modelSha256>`，但 C:191-192 的 `plan_to_proposed`/PromotionResult 没有携带 branch/model，也没有测试：解析 basis、缺失 `not_promotable`。B 的 Agent 提案同样需要该字段；应在 PLATFORM 之外补 C/B 的落点。
3. **【阻塞】`evidenceSha256` 无来源。** C:58/145/153 依赖它做 stale/promote；PLATFORM 5 只定义 `model_digest`，D:72 的 evidence 用 `fingerprint+sha256`。应定义 evidence digest 的输入文件/算法/基准时间，并与 D 的 manifest.evidence 对齐。
4. **【需修改】A0 synthetic preview 的 `scope` 类型。** A:43 的 `scope=["preview"]` 与 `Authorization.scope: String`（action.rs:213-218）不匹配；修复后才能实现 A0。
5. **【需修改】B `untrusted` 字段/依赖小漂移。** ToolResult 无 `untrusted`，B:303 仍写“标记 untrusted”；B 依赖树无 `poie-digest`，但 approval digest 使用 DIGEST。应在 B 内同步。
6. **【需修改】浏览器预览与 host-only 测试策略。** A:383 已写“Catalog mock/host E2E 策略”，B/C/D 的 Playwright 仍只在浏览器预览（playwright.config.ts:4-6）；需要一份统一的 fixture/E2E 表，明确哪些验收在 host 侧运行。
7. **【需修改】`SCHEMAS.md` 检查器尚未实现。** SCHEMAS:1-3 的 `check-schema-registry.py` 不存在，A0 步骤未列；应在 A0 加脚本/CI 用例，或删除该承诺。
8. **【需修改】新 crate/CI 门禁的落地仍分散。** A:379-384 有 `poie-digest`/members/Cargo.lock/test-matrix；B:341/C:274?/D:276 也有类似步骤，但 PLATFORM:153-154 的统一要求没有逐阶段复制；建议在每阶段首个 PR 模板中加固定清单。
9. **【需修改】代码注释与契约词表漂移。** `poie-plan/src/sap.rs:1-8,87` 写“six dispatch checks”，实际 `review_dispatch` 是七项（review.rs:1-12）；`poie-state/src/lib.rs:31` 仍说 M12 未实现。设计文档说七项是对的，但代码注释会误导 B/D。

## 3. REQUIRED（开发前必须修复）与 OPTIONAL

### REQUIRED（按优先级）

| 编号 | 必须修改 | 涉及 |
| --- | --- | --- |
| R1 | A0：把 synthetic preview 的 `scope` 改为 `String` 或 ADR 扩展 `Authorization`；同步 `execution.rs` 的 `ready/authorized` 断言并加“synthetic 不可 dispatch/未登录不生成授权”负例 | A |
| R2 | C：定义 `evidenceSha256` 的来源/算法/基准时间，并与 D 的 manifest.evidence 对齐；给 `plan_to_proposed`/PromotionResult 增加 `simulation=<branch>;model=<sha>` 的 basis 编码与“缺失 not_promotable”测试 | C |
| R3 | README：把 P5 的“live digest 不变”改为“C 不写 ledger/execution/federation；SourceModel digest 不变”；P6 改为“manifest 记录 digest，bundle 不含模型/二进制” | README |
| R4 | B：删除“工具输出标记 untrusted”或给 ToolResult 加 `untrusted` 派生字段；B 依赖树补 `poie-digest`；定义 `policyProfile` 与 `RunState` 字段 | B |
| R5 | SCHEMAS/A0：实现 `scripts/check-schema-registry.py` 或在 A0 删除该承诺；把 `SCHEMAS.md` 的注册表/兼容函数接入 CI | PLATFORM/A |
| R6 | A/C/D：给出对象图确定性布局或字段坐标映射；统一 Playwright fixture/E2E 表（mock vs host） | A/B/C/D |
| R7 | 各阶段：把 PLATFORM:153-154 的 fmt/clippy/test/doc/cargo-deny/check-test-matrix/check-docs/ADR 清单写入首个 PR 模板 | 全部 |
| R8 | 研究备忘：更新 `poie-agent`/`auto_read`/七工具/`dispatch.request`/`ensure_live_dispatch`/测试计数等陈旧表述，标注“以 55bd48d 为准” | 研究 |

### OPTIONAL（可延后/可降级）

- O1 `pass^k` 已限定现场真实 provider；CI 继续用确定性用例与覆盖矩阵（已采纳，保持）。
- O2 SBOM/attestation、Ed25519 签名、自动更新器延后；SHA-256 + 明确“未认证”是可接受的 MVP。
- O3 DMG/portable exe：D 已改为 `.app.zip`，保持“不生成 DMG”即可。
- O4 对象图确定性布局算法可先用稳定布局；不必现在设计图数据库。
- O5 VEC 指标、跨站点 what-if、bundle 压缩/差分、多租户留到后续。
- O6 生成 TS 之外再生成 JSON Schema/OpenAPI，可后续做。
- O7 `recentErrors` 先复用 audit 事件（D 已如此），保持。
- O8 capability 用不可伪造的内部句柄 + 参数摘要；不追求跨进程签名。
- O9 代码注释 “six dispatch checks” 与 `poie-state/src/lib.rs:31` 的 M12 过时描述，可随 A 的清理 PR 一起修。
- O10 `diagnostics.redactions` 可先做字段级白名单，但需在 UI/文档说明“不做内容级扫描”。

## 4. “可以开始 Phase A 了吗？”检查清单

- [ ] **R1** A0 synthetic preview `scope` 类型已定；`execution.rs` 测试断言更新与 synthetic 不可 dispatch 负例已定。
- [ ] **R3/R4** README P5/P6 与 C/B 的静态演示/审计字段已对齐。
- [ ] **R5** `SCHEMAS.md` 检查器已实现或在 A0 删除承诺。
- [ ] **R6** 对象图布局/Playwright fixture 已定（至少 mock Catalog + host JSON 快照）。
- [ ] **R7** `poie-digest`/`poie-objects`/`poie-sdkgen` 进根 members、Cargo.lock、test-matrix、cargo-deny/ADR 已写入首个 PR。
- [ ] `OntologyCatalog.catalog_digest`、`SapPreview.rejections`、`plan.preview_sap` 输入、`ActionType/FunctionType/Predicate` 字段已定（当前 A:214-253 已基本满足）。
- [ ] A0 的 gate/sandbox/inference 鉴权、scope 矩阵、错误包络已实现并有拒绝测试。
- [ ] 六个旧视图、Playwright smoke、i18n、`check-docs`/`check-test-matrix` 的验收命令已写入 PR。

**结论：当前为“否”，但已接近。** R1 是 Phase A 的直接阻塞；R2/R3 是 B/C/D 的开工前置；R4–R8 是文档一致性/测试门禁。完成 R1–R3 后可重跑本校验，预计可进入 Phase A。

## 5. 研究遗漏与过度声明

- **过度声明 1（README）**：P5/P6 仍把 C/D 描述成“live digest/模型 bundle”，与 C 的静态演示、D 的元数据 bundle 不一致；应改文案。
- **过度声明 2（D）**：D:26-27 的 DeliveryPlan“可更新/不可更新”已改为 `adoptManifest`（D:121），但 README 的退出条件仍写“离线导入可启动”；应改为“可检查/可采纳 manifest”，不要暗示安装。
- **遗漏 1（C）**：`evidenceSha256` 没有来源；C 的 stale 检查因此无法真正实现。
- **遗漏 2（C/B）**：SAP §7.4 的仿真来源只在 PLATFORM 定义，C 的 `plan_to_proposed`/B 的 Agent 提案没有落点。
- **遗漏 3（A）**：对象图布局仍依赖手写坐标；Catalog 只给类型/链接，无法独立渲染图。
- **过程问题**：文档在本次校验期间经历多轮并发修订；本报告按 `55bd48d` 快照。若继续修改文档，必须提交新 commit 后重跑，不能引用本报告的旧行号。

VERDICT: CHANGES_REQUIRED
