# Phase A 设计门禁校验（A0 + A1）

- 校验对象：[Phase A](phase-a-ontology-product.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)、[README](README.zh-CN.md)；代码 `crates/poie-{ontology,state,plan}`、`apps/desktop/src-tauri`、`apps/desktop/src`、根 `Cargo.toml`、`scripts/check-*.py`。
- 快照：分支 `design/phase-a-d`，**实际 HEAD `c4bc294`**。任务给的 `55bd48d` 已落后 2 个提交（`55bd48d → 91beea9 → c4bc294`）；`91beea9` 删除了 synthetic `Authorization`，`c4bc294` 修了命令矩阵/schema/preview DTO。本报告只对 `c4bc294` 负责。已入库/工作树的 `VALIDATION.zh-CN.md` 绑定 `91beea9`，`c4bc294` 已修其部分条目；本报告独立复核，不引用其行号。
- 方法：只读对照 PO/SAP 与源码；`python3 scripts/check-docs.py` 通过；未运行完整 `cargo test`；未修改既有源码/设计文档，仅新增本文件。工作树中 `docs/design/VALIDATION.zh-CN.md` 有并发未提交修改（非本 agent），本报告不修改它。

## 0. 结论

**CHANGES_REQUIRED。** A0 的“假授权”已经实质关闭，Catalog/Digest 方向正确，没有架构级否决项；但仍有 **6 组 A 级契约问题**会导致错误代码或不可测验收。完成 §3 的 1–6 后可重跑；当前不能开工。

## 1. 五个最强点

1. **A0 不再伪造授权。** A:43-46 明确不生成任何 `Authorization`，execution 只显示 proposed/validated 与 synthetic-preview，测试断言 `authorized`/`dispatched` 为 0、`dispatch_allowed=false`；`execution.rs` 与 `federation_view.rs` 的硬编码授权都纳入清理（[execution.rs](../../apps/desktop/src-tauri/src/execution.rs):96-101、[federation_view.rs](../../apps/desktop/src-tauri/src/federation_view.rs):85-90）。
2. **单一执行语义守得住。** `preview_plan_through_sap` 只读、不写 ledger/Authorization/transition（A:263-276）；C 的 `plan_to_proposed` 只产生 proposed；B 无 dispatch 工具。Catalog 不成为第二套执行路径。
3. **Digest 规范可复现。** fingerprint 与 SHA-256 分离、domain 分隔、`-0.0` 归一、manifest 排除 `integrity`、固定测试向量（DIGEST:20-40、58、89、96-102）；`model_digest`/`catalog_digest` 已在 Catalog 中（A:214-222）。
4. **Catalog/SDK 路径单一。** `catalog_digest`、`SapPreview.rejections`、`plan.preview_sap` 不接受 `planRef`、`ActionPreview/ObjectDetail` 有 `schema`、`PreviewError` 有映射（A:125、188-256、280-293）；`poie-sdkgen --check` 零 diff + 黄金文件（A:292-303）。
5. **UI 方向正确。** 图节点/边来自 Catalog、删除 `GRAPH_NODES`、确定性列布局、Playwright mock Catalog 与 Rust host 测试分工（A:317-319）；属性 `source/asOf/verification` 与 PO §3.1 的声明/观测分离目标一致（A:76、166）。

## 2. R1–R4 闭合表

| 项 | 判定 | 证据 | 残余 |
| --- | --- | --- | --- |
| R1 文档已提交/冻结 | **closed\*** | 全部设计文档在 `c4bc294` 入库；README 以“分支 HEAD”为快照（README:103） | 任务基线 `55bd48d` 已过期；工作树中 `VALIDATION.zh-CN.md` 仍有并发修改，冻结前需提交并以新 commit 重跑 |
| R2 catalog_digest / rejections / preview 输入 / DTO | **partial** | `catalog_digest` 统一（A:217、226）；`rejections` 统一（A:242）；`planRef` 已删除（A:125）；`Predicate` 已删除、`preconditions` 改 `Vec<String>`（A:252） | `EvidenceRef` 字段两套；`ObjectPage/ObjectDetail/ActionPreview` 缺 `modelDigest`；`ActionType.label/description`、`ObjectInstance.version` 等无模型来源 |
| R3 A0 第 0 步 / gate / sandbox / demo 授权 / workspace 门禁 | **partial** | A0 是实施第 0 步（A:383）；`execution_gate_set` 先鉴权（A:40）；demo 授权已删除并有测试断言（A:43-46）；根 `members`/`Cargo.lock`/测试矩阵/cargo-deny 已写（A:384、389） | `auth_*` 白名单自相矛盾；`sandbox_root` 漏列；A0 未包含 no-follow/原子写；ADR 未进步骤 |
| R4 DIGEST canonical / 浮点 / 固定向量 / poie-digest | **closed\*** | 删除 `Asset.description`（DIGEST:71-72）；`-0.0`、有限值、IEEE-754、固定向量（DIGEST:58、96-102）；`poie-digest` 进 members/测试矩阵（DIGEST:102、A:384） | 实际根 `Cargo.toml` 尚无该 crate（A1 才创建）；A1 首个 PR 必须补 `members`/lock，并写 `sha2`/属性测试依赖的 ADR |

> R5–R18 属 B/C/D 门禁；本报告只在它们与 A 契约相交处引用，不作为 A0/A1 阻断依据。

## 3. 剩余阻塞（按优先级）

### B1 A0 的命令矩阵仍不可直接实现（A0，阻塞）

- **`auth_*` 自相矛盾。** A:42 要求“未登录调用……`auth_*` 必须拒绝”，但 PLATFORM:74 把 `auth_status/register/sign_in/sign_out` 全部标为免登录；`auth_register`/`auth_sign_in` 是首启引导入口（[auth.rs](../../apps/desktop/src-tauri/src/auth.rs):81-110），按 A 字面实现会让新机永远无法注册/登录。**最小修复**：A:42 改为“除 `auth_status/register/sign_in` 三个引导命令外，未登录的 `auth_sign_out`/`audit_list` 等拒绝”，测试按命令逐项列，不用 `auth_*` 通配。
- **`sandbox_root` 漏列。** PLATFORM:79 只有 `sandbox_read/write`，但 host 还有 `sandbox_root`（[lib.rs](../../apps/desktop/src-tauri/src/lib.rs):224）并返回绝对路径。**最小修复**：矩阵补 `sandbox_root`（建议登录=是、main），或删除该命令并改前端。
- **A0 未覆盖 no-follow/原子写。** PLATFORM:90 要求 `sandbox_*` no-follow + 原子写，A:41 只写会话/路径/大小；现有 [sandbox.rs](../../apps/desktop/src-tauri/src/sandbox.rs):38-47 用 `std::fs::write`，会跟随根内符号链接且非原子。**最小修复**：A0 第 2 条补 no-follow + 临时文件/fsync/rename，并加“符号链接逃逸/部分写”测试。
- **测试预期未写全。** A:46 只列 `authorized/dispatched=0` 与 `dispatch_allowed=false`；[execution.rs](../../apps/desktop/src-tauri/src/execution.rs):181-197 仍断言 `review_status=="ready"`、0 rejections、全部 checks 通过、全部 authorized。去掉授权后 `review_dispatch` 会对每个动作报 `missing_authorization`，必须同步为 `blocked` 且 rejection 数等于动作数。**最小修复**：A:46 明确 `review_status=="blocked"`、authorization 检查失败、其余六项按实际状态，并保留“synthetic 不派发”负例。

### B2 `status_evidence` 的 source 极性写反（A1，阻塞）

A:76 写“`Asset.status` 有 `status_evidence` 时 `source=declared`……没有时为合成观测”，与代码/PO §3.1 相反：[model.rs](../../crates/poie-ontology/src/model.rs):270-272 明确 `None` 才是 declared/synthetic observation，`Some` 是证据支撑。按原文会把“有证据”标成最弱等级、把“无证据”标成观测，直接违反 A 的属性证据目标。**最小修复**：改为“有 `status_evidence` → `source=observed`（或新增 `evidence`），`asOf` 取证据时间（未知为 null）；无 `status_evidence` → `source=declared`、`asOf=null`、UI 显示声明/合成”，并为两分支各写 fixture。

### B3 EvidenceRef 与响应 digest 传播不完整（A1/A2，阻塞）

- **`EvidenceRef` 两套字段。** A:188 只有 `kind/id/fingerprint/sha256`；PLATFORM:121 还有 `asOf/source` 且要求 fingerprint 与 sha256“至少一个”。SDK 从 A 生成会缺字段，B 的 citation、C 的 evidenceRefs、D 的 manifest 无法互解析。**最小修复**：A 补齐 `asOf/source` 与“至少一个”约束，并加 DTO round-trip。
- **`ObjectDetail.evidence` 无填充规则。** 模型唯一证据是 `Asset.status_evidence: Option<String>`（model.rs:272），没有证据账本；若按 PLATFORM 的“至少一个 digest”，无法构造合法 `EvidenceRef`。**最小修复**：MVP 返回空 evidence + 把 `statusEvidence` 作为 declared 属性字符串；或定义 `{kind:"report", id:status_evidence, fingerprint:status_evidence, source:"model", asOf:null}` 并测试。
- **digest 未随所有 `ontology_*` 返回。** A:291 要求返回 `modelDigest` 与 `catalogDigest`，但 `ObjectPage` 只有 `catalog_digest`（A:224-229），`ObjectDetail`/`ActionPreview` 两者都没有。**最小修复**：给三者统一加 `modelDigest`/`catalogDigest`（或一个 `CatalogStamp`），并覆盖 `model_mismatch` 测试。

### B4 预览 API 的输入/归属仍未闭合（A2，阻塞）

- **`actionTypeId` 无法喂给 `preview_plan_through_sap(&Plan)`。** A:285 的 `ontology_action_preview(actionTypeId, params?, assumedContext?)` 与 A:266 的 `preview_plan_through_sap(&Plan, &AssumedContext)` 不匹配；`ActionType` 不含 target/runs/validity，host 也没有从 actionTypeId 构造 Plan 的规则；`ActionPreview.plan_ref`（A:251）也要随二选一保留或删除。**最小修复（二选一）**：新增 `preview_action_through_sap(actionTypeId, target, runs, validity, assumed)`，host 构造单个 `Action` 只跑七项检查；或改成 `ontology_plan_preview(planRef|goal, assumed)`，ActionPreview 只作行视图。
- **`params?` 与“无任意参数”冲突。** A:285 仍带 `params?`，A:112 明确不提供任意参数/字段、不包装 `runs`。**最小修复**：删除 `params?`；若确实需要计划变量，只允许显式 `target/runs/validity` 并逐项校验。
- **`SapPreview` 的 crate 归属未定。** A:239-243 把它列在 `poie-objects` API 中，A:266 的 `preview_plan_through_sap` 未写 crate 归属（本节上文是 `poie-plan::validate_plan_through_sap`）；若放在 `poie-plan`，PLATFORM:163 的 `poie-objects` 依赖没有 plan，A:276 又要求 `poie-objects` 不依赖 plan，两边成立时只能复制两份 DTO。**最小修复**：把 `SapPreview/CheckPreview/AssumedContext` 定义在 `poie-plan`（它拥有七项检查和 Plan 输入），host 组合 `ActionPreview`；或在 PLATFORM §7 显式增加 `poie-plan -> poie-objects` 并说明分层。
- **七项检查的状态映射不完整。** A:274 只把 ownership/reservations/authorization 标为 assumed；`AssumedContext`（A:231-234）没有 predicates/state_version/executor_dedup_window，preconditions 与 dedup 只能 unknown。**最小修复**：给出七项逐项表（observed/assumed/unknown），并测试 UI 不出现“7/7 通过”。

### B5 A1 的“模型派生字段”仍缺来源（A1，阻塞）

- `ActionType.label/description`（A:203）在已验证 `Process` 中不存在（[model.rs](../../crates/poie-ontology/src/model.rs):304-316 没有 description）；`ObjectInstance.version`（A:194）、`LinkType.cardinality`（A:186）、`ActionType.duration` 的格式（A:205）也没有派生规则。
- **最小修复**：给出确定性派生表：`version=model_digest`（或对象内容哈希）、`label=id` 或 `humanize(id)`、`description` 从 `Model.doc` 显式读取并说明非第二套、或置空，`cardinality` 按链接类型固定，`duration` 统一为 `"{:.3} s"`；加“每个 ActionType 字段逐项来自已验证 Model”的黄金 fixture。否则 Catalog 会变成第二套字段表。

### B6 过程门禁未落到 A1 首个 PR（process，阻塞）

- PLATFORM:172 要求新增依赖写 ADR（含 `sha2`），A:383-389 没有 ADR 步骤。**最小修复**：A1 第 1/6 步加 ADR（`poie-digest` 的 `sha2`、canonicalizer、属性测试依赖）。
- SCHEMAS:4/PLATFORM:9 说 `scripts/check-schema-registry.py` 在 A0 加入，但 A0 的 5 条与第 0 步（A:40-44、383）都没列。**最小修复**：A0 加脚本 + 失败用例，或把承诺改到 A1 并更新 SCHEMAS。
- A:342 说“limit 上限”，但没有默认值/最大值；`limit_exceeded` 无法触发。**最小修复**：定义 `default=50, max=200` 与 cursor 编码，并加边界测试。
- PLATFORM:171 要求更新桌面自己的 `Cargo.lock`，A:384 只写根 lock。**最小修复**：host 第 2 步补 `apps/desktop/src-tauri/Cargo.lock`。

## 4. 非阻塞（OPTIONAL，可进 A2 或后续）

- 图节点集合含 `capability`，但布局只有 service/process/asset/resource 三列（A:317-318）；补 capability 列或从节点集删除，并定义“当前服务可达”的函数。
- Playwright 的“mock Catalog fixture”只有名字（A:319），未给文件/生成方式；建议复用 `catalog.golden.json`，避免手写 fixture 漂移。
- `ObjectPage.next_cursor` 的编码、`FunctionType.input_schema/output_schema` 的具体 schema、`catalog_digest` 是否包含 `model_digest` 的措辞，可在 A1/A2 一并定稿。
- `federation_view.rs` 的 demo transition 仍写 `Authorized→Dispatched`；去掉授权后建议标为 synthetic 或改为 proposed-only 记录，避免 UI 误读（A0 清理 PR 可一并做）。
- P:181 的 “model store” 遗留词、`sap.rs:87` 的 “six dispatch checks”、`poie-state/src/lib.rs:31` 的 M12 旧注释，可随 A 的清理 PR 修。

## 5. Phase A 就绪清单

- [x] A0 已列为实施第 0 步；gate 先鉴权；demo 授权移除；`dispatch_allowed=false` 与 `authorized/dispatched=0` 写入验收（A:40-46、383）。
- [x] `catalog_digest`、`SapPreview.rejections`、`plan.preview_sap` 无 `planRef`、`ActionPreview/ObjectDetail.schema`、`PreviewError` 映射（A:125、188-256）。
- [x] DIGEST 删除 `Asset.description`、`-0.0`/浮点规则、固定向量、`poie-digest` 进 workspace 计划（DIGEST:58、71-72、96-102；A:384）。
- [x] `SourceModel = Base-01 + pump-p1 degraded`，host 持 `Arc<Model>` 的方向明确（PLATFORM:98-104；A:38）。
- [ ] **B1** `auth_*` 例外、`sandbox_root`、A0 no-follow/原子写、execution 测试预期已改。
- [ ] **B2** `status_evidence` 的 source/asOf 极性已修正，并有双分支 fixture。
- [ ] **B3** `EvidenceRef` 补齐 `asOf/source`；`ObjectDetail.evidence` 填充规则；所有 `ontology_*` 带 `modelDigest`/`catalogDigest`。
- [ ] **B4** Action 预览/Plan 预览二选一；删除 `params?`；`SapPreview` crate 归属；七项状态表。
- [ ] **B5** ActionType/ObjectInstance 的每个字段有确定性模型来源，黄金 fixture 覆盖。
- [ ] **B6** ADR、`check-schema-registry.py`、limit 上限、桌面 `Cargo.lock` 写入 A1 首 PR。
- [ ] 以上修改提交为单一 commit，以该 commit 重跑本报告；当前 `c4bc294` 不能作为开工基线。

**结论：当前为“否”，但没有架构级阻塞。** 先关闭 B1（A0 可直接产生错误鉴权/路径逃逸）与 B2/B3（A1 的数据语义），再关闭 B4–B6。B/C/D 的门禁项不在本报告阻断范围。

VERDICT: CHANGES_REQUIRED
