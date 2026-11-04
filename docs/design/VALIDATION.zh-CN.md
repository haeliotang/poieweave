# Poieweave Phase A–D 设计独立校验（门禁）

- 基线：任务指定 `45a2a93`；校验期间分支连续前进到 `f75c31d`、`55bd48d`、`91beea9`。本报告绑定 **`91beea9`**（工作树除本文件外干净），未修改源码、设计文档或 git 状态。
- 快照 SHA-256（前 10 位）：README `696a371b4a`、DIGEST `61ae5cd0d8`、PLATFORM `7d499342ab`、SCHEMAS `c8e53ff214`、A `7a587c069b`、B `175a797ec7`、C `574c2ff0f0`、D `8fc4edee92`。若在本报告后继续提交，必须按新快照重跑。
- 记法：[A](phase-a-ontology-product.zh-CN.md)、[B](phase-b-agent-governance.zh-CN.md)、[C](phase-c-what-if.zh-CN.md)、[D](phase-d-site-delivery.zh-CN.md)、[P](PLATFORM.zh-CN.md)、[G](DIGEST.md)、[S](SCHEMAS.md)；代码引用为 `path:line`（链接见 §5）。
- 方法：逐条对照 R1–R18、三份研究备忘、PO/SAP/VEC 与指定源码；`python3 scripts/check-docs.py` 通过；未运行完整 `cargo test`。

## 0. 总判定

**CHANGES_REQUIRED。** 本轮最重要的修复已经落地：A0 在 `91beea9` 删除了 synthetic `Authorization`，改为“不生成授权、`authorized`/`dispatched` 为 0”的只读预览（A:43-46），上一轮的“假授权”阻塞已关闭；A 的 Catalog/DTO/digest、B 的 RunState/审批/审计、C 的静态快照/OutcomeKind/`evidenceSha256`、D 的 metadata-only bundle 也都基本闭合。

没有发现架构级 BLOCKER。剩余是 **7 项 A 级契约修复**（scope 矩阵行、schema 单一来源与字段、`StateId` 描述、`Predicate` 映射、`planRef`、`PreviewError` 映射、`dispatch_allowed` 三态）加 1 项低优先的 label 来源，以及 B/C/D 的分阶段前置。完成 A 级 1-7 后，A0+A1 可以在“不造第二套本体、不绕过 SAP”的前提下开工；因此不是 PASS，也不是 FAIL。

## 1. 六个最强点

1. **A0 的授权语义已纠正。** A:43-46 明确 A0 不生成任何 `Authorization`，execution 视图只显示 proposed/validated 与 synthetic-preview，测试断言 `authorized`/`dispatched` 为 0；`execution.rs`/`federation_view.rs` 两处硬编码授权都纳入清理。这关闭了上一轮“预览即授权”的核心风险。
2. **单一执行语义贯穿四阶段。** `preview_plan_through_sap` 只读（A:263-275），C 的 `plan_to_proposed` 不 authorize/dispatch（C:194-195），B 无 dispatch 工具（B:92-101），审计不新增 SAP `RecordKind`（B:318）。
3. **Digest 规范可复现。** fingerprint 与 SHA-256 分离、domain 分隔、`-0.0` 归一、manifest 排除 `integrity`、固定测试向量（G:20-40、58、89、96-101）；`catalog_digest`/`model_digest` 已在 Catalog 中（A:215-217）。
4. **B 的治理边界可实现。** 六工具封闭、T0–T3、按工具定义审批 subject、`ApprovalToken`、append-only 日志 + `anchor.json`、`user_version` 迁移、B1 45 个确定性用例且 `sim.run` 返回 `not_implemented`（B:110-115、171-182、224-237、279、308）。
5. **C 的静态快照范围诚实。** 明确 C MVP 只针对 Base-01 演示快照、删除“live 不变”假阳性、`OutcomeKind` 分离结构/时序并带 caveat、`PatchDto` 13 字段、`evidenceSha256` 有新定义、`startMode=scenario-start`（C:53-66、75-79、100-118、93、60-62）。
6. **D 的降级正确。** bundle 只做 JSON 元数据、不导入模型/二进制、manifest 自哈希引用 DIGEST、`host_info.build`、`write_atomic`、导出上限、召回/回滚/通道命令、`.app.zip` 替代 DMG（D:35、82-88、162-190、199）。

## 2. R1–R18 闭合表

| R | 判定 | 证据 | 残余 |
| --- | --- | --- | --- |
| R1 冻结/可追溯 | **closed** | `git log`：4a90edc→45a2a93→f75c31d→55bd48d→91beea9；全部设计文档入库；README 改以“分支 HEAD”为快照 | 任务基线 45a2a93 已落后 4 个提交；门禁期间连续提交，报告只能绑定 91beea9；A0 修复与本次校验同批，独立性由本报告重建 |
| R2 fingerprint/SHA-256/domains/`poie-digest` | **closed** | G:20-40、58、89、96-101；A:215-217；P:169-178 | `poie-digest` 尚未进 `members`（实现前）；`poie.evidence/0.1` 的“source digest”与 `EvidenceRef.sha256` 命名待统一（低） |
| R3 schemaVersion 注册 | **partial** | S:7-19、32-43；P:9、15-24 | P 的内联表与 S 重复且漏 `poie.diagnostics/0.1`；`ObjectDetail`/`SapPreview` 未登记；`ActionPreview`/`ObjectDetail` 无 `schema` 字段；`check-schema-registry.py` 未进 A0 |
| R4 DTO/Serialize | **partial** | P:35-44；A:173-253；C:75-79 | `StateId` 被称为 Deserialize-only，实际无 serde（projection.rs:60-66）；`ActionPreview.preconditions: Vec<Predicate>` 无字符串谓词映射；A 的 DTO 缺 `schemaVersion` |
| R5 错误包络 | **partial** | P:48-73；A:290 | C 的 `unknown_snapshot`、B 的 `citation_missing`、`sim.run` 的 `not_implemented` 未登记；`PreviewError` 未映射到 `CommandError` |
| R6 命令 scope + A0 | **partial** | P:77-99；A:36-46、380；lib.rs:143、224 | A0 授权语义已闭合；矩阵仍写 `main/execution`、`main/planning`（实际 `poie-view-*`）；`sandbox_list` 不存在、`sandbox_root` 漏列 |
| R7 单一 SourceModel | **closed** | P:107-113；D:35、193；C:58、145 | P:190 的实现顺序仍写 “model store”（遗留词）；其余一致 |
| R8 ActionType 模型字段 + 只读预览 | **closed** | A:100-114、263-275 | `can_submit` 已删、预览只读；`label/description` 无模型来源（低） |
| R9 单一 SDK/无外部 JSON API | **closed** | A:54、279、370；README:80；A:144 | `poie-sdkgen` 唯一路径；外部网络 API 明确另立阶段 |
| R10 crate/工具映射 | **closed** | B:92-101、319；P:169-178；C:124-131 | 六工具与 A 函数映射齐全；`sim.run` B1 关闭；C 依赖已补 state/digest |
| R11 Agent run 状态 API | **closed** | B:212-237；P:20 | `RunState` schema、六命令、`agent_runs`、`interrupted` 均已定义 |
| R12 审批 digest/审计/DB 迁移 | **partial** | B:171-182、237、318；P:141-162；auth.rs:23-26 | `ApprovalToken.session_id` 需要的 `Session` id 不存在；SQLite approvals 表/迁移未定义；`Authorization.basis` 的 `approval:<tokenId>` 已给编码规则 |
| R13 Snapshot/OutcomeKind | **closed** | C:53-66、100-118；P:107 | `stateId` 为 DTO 字符串；`evidenceSha256` 已给出 `poie.evidence/0.1` 定义（C:60-62） |
| R14 Compare/Promotion/proposed-only/checkpoint | **partial** | C:70-95、171、194-195 | `ProposedActions`/`plan_to_proposed` 签名未定；promote 的审批 subject 未定义；“C 不写 SAP”与 proposed actions 的存放矛盾；仿真来源已用 `Authorization.basis` 编码 |
| R15 manifest digest/contracts | **partial** | D:57-88、186-200；P:5.1 | `contracts.toml` 只有 id/version/revision/sha256 一句，无格式/黄金样例；`release.json` 与 `RELEASE.json` 大小写不一致；tag gate 仍是要求，未落 workflow |
| R16 JSON bundle/policy/DB | **partial** | D:35、82-84、162-190；P:100 | 不导入模型/JSON bundle/原子写/导出上限已定；导入 policy 与本地策略的优先级、能否放宽 scope 未定；`HostPolicy.channel` 与 `DeliveryState.channel` 双源 |
| R17 命令/DMG 修正 | **closed** | D:164-175、189、199 | inspect/plan/recall/channel/policy 齐全；`.app.zip` 与 workflow 一致。根 `README.md:27,82` 仍写 DMG/portable，需在 D 前清理 |
| R18 workspace/deps/ADR/test-matrix | **partial** | P:181-182；A:380；B:345；D:271；C:272-279 | A/B/D 有 step 0；C §11 仍从 step 1 开始；SCHEMAS 检查器未进 A0；新 crate 未入 `members`（实现前） |

## 3. 剩余发现

### BLOCKER

**无架构/安全级 BLOCKER。** 上一轮的 A0 synthetic 授权已由 A:43-46 关闭；当前没有必须推翻的契约或执行语义。唯一的过程性要求是：以 `91beea9` 为门禁快照，Phase A 前不再改设计文档；若继续提交，必须重跑本校验（任务基线 45a2a93 已被 4 个提交取代，不能用旧行号引用）。

### REQUIRED-BEFORE-PHASE A

1. **Scope 矩阵行与标签不一致。** P:87 仍写 `main/execution`、P:92 仍写 `main/planning`，实际窗口是 `poie-view-execution`/`poie-view-planning`（lib.rs:143）；`sandbox_read/write/list`（P:88）中的 `list` 不存在，`sandbox_root`（lib.rs:224）未列。A0 的窗口拒绝测试若按文字实现，会在错误窗口上通过/失败。
2. **两个 schema 注册表不同步。** S:7-19 已有 `poie.diagnostics/0.1`，P:15-24 没有；S 是“唯一来源”却又在 P 重复。A 的 `ActionPreview`（A:246）与 `ObjectDetail`（A:197）没有 `schema` 字段，与 S:32“每个对外结构必须有 schema”冲突；`ObjectDetail` 也未登记。应删 P 的内联表，只留 S，并给每个命令输出加 `schema`/`schemaVersion`。
3. **`StateId` 的描述错误。** P:39 说 `AssetStatus`/`LotQuality`/`Verification`/`Patch`/`StateId` 保持 Deserialize-only；`StateId` 实际只有 `Clone/Debug/PartialEq/Ord/Hash`（projection.rs:60-66），没有任何 serde。C 已改用 DTO 字符串（C:58），P 应同步为“非 serde，DTO 字符串”。
4. **`Predicate` 无来源映射。** `ActionPreview.preconditions: Vec<Predicate>`（A:246-247），但计划动作的前置条件是 `Vec<String>`（`PlannedAction.preconditions`）和 `BTreeSet<String>`（`Action.precondition.predicates`）；`Predicate{field,op,value}` 无法从这些字符串稳定派生。应改为 `Vec<String>` 或定义谓词语法与转换测试。
5. **`plan.preview_sap` 的 “or planRef” 重新引入 plan store。** A:125 允许 `planRef`，但没有任何 plan registry；A 的 host 只持有当前 `SourceModel`。Phase A 应只接受 host 持有的 `&Plan`，删除 `planRef` 分支（或明确 planRef 由 host 内存映射并在 A0 定义）。
6. **`PreviewError` 未映射到错误包络。** A:244 定义了 `PreviewError{InvalidPlan,UnsupportedContext,NoPreviewAvailable}`，P:48-73 没有对应 code；`preview_unavailable` 只在 P:64。应给出 `PreviewError -> CommandError` 映射表。
7. **`ExecutionSnapshot.dispatch_allowed` 的预览语义未定义。** A0 只断言 `authorized`/`dispatched` 为 0，没有规定 `dispatch_allowed` 在无授权/合成预览下必须为 `false`；现有代码用 `review_rejections.is_empty()` 推导（execution.rs:158-160），而 `review_dispatch` 只检查授权存在/过期/撤销（review.rs:185-207），不检查预览标记。应把三态（无会话/合成预览/真实授权）写成状态表并测试。
8. **（低）`ActionType.label/description`、`ObjectType.label` 无模型字段。** `Process` 没有 `description`（model.rs:304-316），`Asset`/`Resource` 也没有；A:201-204 却要求这些字段。若采用“id 人类可读化”的确定性派生，必须写明算法并测试；否则删掉字段。

### REQUIRED-BEFORE-PHASE B

1. **`Session` 没有 id。** `auth::Session` 只有 `display_name`/`expires_at_ms`（auth.rs:23-26）；B 的 capability 需要 `sessionId`（B:161），P 的 `ApprovalToken` 需要 `session_id`（P:148）。必须先给 Session 加不可猜测 id 与 epoch，并定义登出/接管时 capability/token 的撤销。
2. **审批 token 没有表/迁移。** B:237 只定义 `agent_runs(run_id, schema_version, state_json, session_id, created_at, updated_at, expires_at)`；P:162 说“所有 token 持久化（SQLite）”。需要 `approvals` 表（token id/subject/run/expiry/used/sap_record_id）与 `user_version` 迁移；否则暂停/恢复只能靠内存。
3. **`citation_missing`/`not_implemented` 未登记。** B:122 使用 `citation_missing`，B:99 使用 `not_implemented`，P:56-71 没有；错误码表与实现会分叉。
4. **`http.rs` 没有响应体上限。** 现有 HTTP 传输只有 30s 超时（http.rs:23-25）；B 的多步工具调用会接收 provider 响应，必须定义最大字节/超时/截断测试，否则“预算”只覆盖步数不覆盖内存。

### REQUIRED-BEFORE-PHASE C

1. **`plan_to_proposed` 只有名字。** C:194 返回 `ProposedActions`，但类型、签名、每个 action 的字段、由谁写 ledger 都未定义。C:149 又说 C 不写 SAP 记录/ActionLedger；若 `plan_to_proposed` 只返回内存对象，后续“交给现有 SAP 验证命令”需要一个写入点，必须明确是 host 还是 SAP API。
2. **promote 的审批 subject 未定义。** `whatif_variant_promote(variantId, approvalTokenId)`（C:171）使用 P 的 `ApprovalToken`，但没有像 B:173-178 那样列出 subject 字段（snapshotSha256/variantResultSha256/planSha256/policyVersion/sessionId/expiresAt）。没有这些，审批无法绑定具体结果。
3. **未登记 `unknown_snapshot`，`delta` 无类型。** C:187 使用 `unknown_snapshot`；P:56-71 没有。C:177 的 `delta` 未定义结构，比较结果无法稳定序列化。
4. **C 的 step 0 缺失。** C:272-279 仍从 `poie-whatif` 开始，没有 A/B/D 已加的“进 members/Cargo.lock/ADR/测试矩阵”第 0 步；`poie-whatif`/`poie-digest` 的 CI 计数会在第一个 PR 失败。

### REQUIRED-BEFORE-PHASE D

1. **`contracts.toml` 只有一句。** D:88/194 要求 id/version/revision/sha256，但没有文件 schema、生成方式、黄金样例与“契约正文变更时如何更新 revision”的规则；D:68-71 的 manifest 示例又手写三个条目。
2. **`release.json` 与 `RELEASE.json` 大小写不一致。** D:87/152 用 `release.json`，D:199/275 用 `RELEASE.json`；构建期文件名必须唯一。
3. **召回不阻止继续使用。** D:28 目标写“召回阻止继续使用”，D:122 只写“已安装则显示召回+允许回滚”；没有启动/导入/派发任一执行点的拒绝规则。应明确“召回 release 的已安装 manifest 在启动时标记 degraded/禁止 promote”等可测行为。
4. **导入策略可放宽到什么程度未定。** D:173 的 `delivery_policy_set` 与导入 bundle 的 `policy` 可覆盖 `agentScopes/remoteInference`；D:268 只说“MVP 允许，需审批/审计”。这是安全边界，必须定义不可放宽的下限（例如 `remoteInference` 只能从 true→false）与测试。
5. **`HostPolicy.channel` 与 `DeliveryState.channel` 双源。** P:100 把 `channel` 放进 HostPolicy，D:96-105 又把 channel 放在 DeliveryState 并声明“本地订阅优先”；导入 manifest 时会出现三方冲突。应删除 HostPolicy.channel。
6. **根 README 仍宣称 DMG/portable。** `README.md:27,82` 与 D:35/199 的“不生成 DMG/便携 exe”冲突；D 的漂移清单只写了 design README。交付前必须改根 README 或恢复构建。

### OPTIONAL

- O1 `ActionType.label/description` 的人类可读派生可延后，但必须在 A1 前二选一（派生算法或删字段）。
- O2 `poie.evidence/0.1` 的 “source digest” 与 `EvidenceRef.sha256` 命名统一（低）。
- O3 P:190 的 “model store” 遗留词、P:39 的 `StateId` 描述、`sap.rs:87` 的 “six dispatch checks”、`poie-state/src/lib.rs:31` 的 M12 旧注释，可随 A 的清理 PR 一起修。
- O4 `ObjectDetail` 的 schema 登记与 `poie.object-detail/0.1` 可随 R3 一起补。
- O5 SBOM/attestation、Ed25519、自动更新器、VEC 指标、跨站点 what-if、`two_person` 继续延后；D 的 O2 已覆盖。
- O6 C 的 `whatif_variant_delete` 可加审计事件；不影响 MVP。

## 4. Phase A 就绪清单

- [x] **A0 授权语义**：不生成 `Authorization`，execution/federation 的 demo 授权移除，测试断言 authorized/dispatched 为 0（A:43-46）。
- [x] **单一模型**：`SourceModel = Base-01 + pump-p1 degraded`，host 持有 `Arc<Model>`（P:107-111；A:38）。
- [x] **单一执行路径**：Catalog 只读、`poie-objects` 不依赖 plan、预览不 authorize（A:164-168、263-275）。
- [x] **Digest**：`model_digest`/`catalog_digest` 命名一致，canonical/domain/manifest 规则齐全（A:215-217；G:20-40、89）。
- [x] **对象 ID/EvidenceRef**：`<type>:<id>` 与 `EvidenceRef` 统一（P:115-131）。
- [x] **A0 顺序**：A0 已是实施第 0 步（A:380）。
- [ ] **Scope 矩阵**：`poie-view-*` 行、`sandbox_root`/`sandbox_list` 修正（A 级 1）。
- [ ] **Schema 单一来源**：删 P 内联表，补 `ActionPreview`/`ObjectDetail`/`SapPreview` 的 `schema`，登记 `ObjectDetail`，检查器进 A0（A 级 2）。
- [ ] **A DTO 收尾**：`StateId` 描述、Predicate 映射、`planRef`、`PreviewError` 映射、`dispatch_allowed` 三态（A 级 3-7）。
- [ ] **快照冻结**：以 91beea9 为唯一基线；Phase A 前不再改设计文档。

**结论：当前为“否”，但没有架构级阻塞。** A0 的授权问题已关闭；完成 A 级 1-7 后，A0+A1 可以在不制造第二套本体、不绕过 SAP 的前提下开工。B/C/D 的前置项按 §3 分阶段处理；其中 B 的 Session id/approvals 表、C 的 `ProposedActions`/promote subject、D 的 policy 下限最值得优先。

## 5. 证据路径

- 设计：[README](README.zh-CN.md)、[A](phase-a-ontology-product.zh-CN.md)、[B](phase-b-agent-governance.zh-CN.md)、[C](phase-c-what-if.zh-CN.md)、[D](phase-d-site-delivery.zh-CN.md)、[P](PLATFORM.zh-CN.md)、[G](DIGEST.md)、[S](SCHEMAS.md)。
- 代码：[sap.rs](../../crates/poie-plan/src/sap.rs)、[review.rs](../../crates/poie-state/src/review.rs)、[action.rs](../../crates/poie-state/src/action.rs)、[projection.rs](../../crates/poie-state/src/projection.rs)、[model.rs](../../crates/poie-ontology/src/model.rs)、[execution.rs](../../apps/desktop/src-tauri/src/execution.rs)、[federation_view.rs](../../apps/desktop/src-tauri/src/federation_view.rs)、[lib.rs](../../apps/desktop/src-tauri/src/lib.rs)、[auth.rs](../../apps/desktop/src-tauri/src/auth.rs)、[db.rs](../../apps/desktop/src-tauri/src/db.rs)、[sandbox.rs](../../apps/desktop/src-tauri/src/sandbox.rs)、[http.rs](../../apps/desktop/src-tauri/src/http.rs)、[inference.rs](../../apps/desktop/src-tauri/src/inference.rs)、[App.tsx](../../apps/desktop/src/App.tsx)、[domain.rs](../../apps/desktop/src-tauri/src/domain.rs)、[ci.yml](../../.github/workflows/ci.yml)、[desktop-build.yml](../../.github/workflows/desktop-build.yml)。

VERDICT: CHANGES_REQUIRED
