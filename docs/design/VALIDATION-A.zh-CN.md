# Phase A 设计门禁校验（A0 + A1）

- 校验对象：[Phase A](phase-a-ontology-product.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)、[README](README.zh-CN.md)；代码 `crates/poie-{ontology,state,plan}`、`apps/desktop/src-tauri`、`apps/desktop/src`、根 `Cargo.toml`、`scripts/check-*.py`。
- 快照：分支 `design/phase-a-d`，**实际 HEAD `0527c7f`**。任务给的 `55bd48d` 已落后 4 个提交（`55bd48d → 91beea9 → c4bc294 → e16433d → 0527c7f`）；`91beea9` 删 synthetic `Authorization`，`c4bc294` 修命令矩阵/schema，`e16433d` 修 `auth_*`/`status_evidence`/Plan-based 预览，`0527c7f` 修 EvidenceRef/digest/字段派生/过程门禁。本报告只对 `0527c7f` 负责。
- 方法：只读对照 PO/SAP 与源码；`python3 scripts/check-docs.py` 通过；未运行完整 `cargo test`；未修改既有源码/设计文档，仅更新本文件。`VALIDATION.zh-CN.md` 绑定更早的 `c4bc294`，本报告独立复核，不引用其行号。

## 0. 结论

**PASS（A0 + A1 可开工）。** A0 的宿主鉴权/scope、demo 授权清理、synthetic preview 语义已闭合；A1 的 Catalog/DTO/digest/EvidenceRef 字段与派生规则已闭合；R1–R4 没有剩余 REQUIRED-BEFORE-A。下列 §3 均为 OPTIONAL/A2 细化，不阻断 A0/A1 实施。

## 0.1 六项指定检查

| # | 指定检查 | 判定 | 证据 |
| --- | --- | --- | --- |
| 1 | R1 文档已提交并冻结、修订引用一致 | **PASS** | 设计文档在 `0527c7f` 入库；README 以分支 HEAD 为快照（README:103）；本报告绑定该 commit |
| 2 | R2 `catalog_digest`、统一 `rejections`、`plan.preview_sap` 输入、完整 DTO | **PASS** | A:217/227（digest）、A:243（rejections）、A:125（Plan-only）、A:188-262（DTO） |
| 3 | R3 A0 第 0 步、gate/sandbox/inference 鉴权/scope、demo 授权替换/测试、workspace 门禁 | **PASS** | A:391（第 0 步）、A:40-46（鉴权与 demo）、A:392/397（members/lock/matrix/cargo-deny/ADR） |
| 4 | R4 DIGEST canonical（无 `Asset.description`）、`-0.0`/浮点、固定向量、`poie-digest` 进 workspace | **PASS** | DIGEST:58、71-72、96-102；A:392 |
| 5 | A API 可实现：ActionType 字段、只读 preview、错误包络、对象 ID/EvidenceRef、SourceModel、Catalog 图、Playwright 策略 | **PASS** | A:202-262、267-303、317-319；PLATFORM:46-90、121；`model.rs:270-272` |
| 6 | 无剩余 BLOCKER 到 A0/A1 | **PASS** | §3；O1–O8 均为 OPTIONAL/A2 |

## 0.2 关键判断

- **R2 的“完整 DTO”已达标。** `EvidenceRef` 六字段、`ObjectPage/ObjectDetail/ActionPreview` 的 model/catalog digest、`ObjectInstance.version`、`LinkType.cardinality`、`ActionType.duration_s` 都已给出；`ObjectDetail.evidence` 的填充规则是行为细化（O2），不是字段缺口。
- **R3 的 A0 已可执行。** bootstrap 例外、窗口标签、`sandbox_root`、no-follow/原子写、旧测试断言更新都已写入；`sandbox_list` 已删除。唯一未写进测试清单的是 no-follow 负例（O3），属测试计划补充。
- **A2 的预览/UI 仍有 OPTIONAL。** `ActionPreview` 的 host/`poie-objects` 代码块归属（O1）、七项状态表（O4）、`planId` 映射（O5）、capability 列（O7）都可在 A2 前收尾，不影响 A0/A1 开工。

## 1. 五个最强点

1. **A0 不再伪造授权。** A:43-46 明确不生成任何 `Authorization`，execution 只显示 proposed/validated 与 synthetic-preview，测试断言 `authorized`/`dispatched=0`、`dispatch_allowed=false`；`execution.rs`/`federation_view.rs` 的硬编码授权纳入清理并写明更新旧断言（[execution.rs](../../apps/desktop/src-tauri/src/execution.rs):96-101、181-197；[federation_view.rs](../../apps/desktop/src-tauri/src/federation_view.rs):85-90）。
2. **A0 命令矩阵可执行。** `auth_status/register/sign_in/sign_out` 明确为 bootstrap 例外（A:42、PLATFORM:74）；`execution_gate_set` 窗口为 `main/poie-view-execution`、`whatif_*` 为 `main/poie-view-plan`（PLATFORM:78、83）；`sandbox_root` 由 `host_info` 只读返回、无 `sandbox_list`（PLATFORM:79）；A0 第 2 条补 no-follow + 临时文件/原子 rename（A:41）。
3. **属性证据语义已修正。** A:76 现在写 `status_evidence=Some → source=evidence`、`None → source=declared`（模型作者的合成观测），与 [model.rs](../../crates/poie-ontology/src/model.rs):270-272 和 PO §3.1 一致；`asOf` 未知为 null。
4. **Catalog/DTO/digest 完整。** `catalog_digest`、`model_digest` 传播到 `ObjectPage`/`ObjectDetail`/`ActionPreview`（A:217、227、198、256）；`EvidenceRef` 补齐 `asOf/source`（A:188）并与 PLATFORM:121 对齐；`ObjectInstance.version=model_digest`、`LinkType.cardinality` 固定值、`ActionType.duration_s` 直接来自模型（A:194、186、207）；`ActionType.label/description` 已删除，不再伪造模型字段（A:203）。
5. **预览与 SDK 路径单一。** `plan.preview_sap` 只接受 host 持有的 Plan（A:125）；`ontology_action_preview` 改为 Plan/actionId + `PlannedActionView`（A:249-262、293），删除 `params?`；`SapPreview/CheckPreview/AssumedContext/PreviewError` 权威定义归 `poie-plan::sap`（A:267）；`PreviewError` 有错误码映射（A:245-247）；`poie-sdkgen --check` 零 diff + 黄金文件（A:297-303）。

## 2. R1–R4 闭合表

| 项 | 判定 | 证据 | 说明 |
| --- | --- | --- | --- |
| R1 文档已提交/冻结 | **closed** | 设计文档在 `0527c7f` 入库；README 以“分支 HEAD”为快照（README:103） | 任务基线 `55bd48d` 已过期；本报告以 `0527c7f` 为唯一快照，后续若再提交需重跑 |
| R2 catalog_digest / rejections / preview 输入 / DTO | **closed** | `catalog_digest` 统一（A:217、227）；`rejections` 统一（A:243）；`planRef`/`params?` 删除（A:125、293）；`ActionPreview/ObjectDetail/ObjectPage` 带 model/catalog digest；`EvidenceRef` 六字段；`ActionType` 仅模型字段（A:188-262） | `ObjectDetail.evidence` 的具体填充规则见 O2，属 A1 实现细化，不改变 DTO 契约 |
| R3 A0 第 0 步 / gate / sandbox / demo 授权 / workspace 门禁 | **closed** | A0 是第 0 步（A:391）；`execution_gate_set` 先鉴权（A:40）；`auth_*` bootstrap 例外（A:42）；demo 授权删除并写测试（A:43-46）；no-follow/原子写（A:41）；根/桌面 `Cargo.lock`、测试矩阵、cargo-deny、ADR、schema checker、limit 上限均在 A:392/397 | 无 |
| R4 DIGEST canonical / 浮点 / 固定向量 / poie-digest | **closed** | 删除 `Asset.description`（DIGEST:71-72）；`-0.0`/有限值/IEEE-754（DIGEST:58）；固定向量（DIGEST:96-102）；`poie-digest` 进 members/测试矩阵（DIGEST:102、A:392） | 实际根 `Cargo.toml` 尚无该 crate（A1 才创建），首个 PR 必须一并加 `members`/lock |

> R5–R18 属 B/C/D 门禁；不阻断 A0/A1，本报告不据此判定。

## 3. 剩余阻塞

**无 BLOCKER。** A0/A1 的契约、错误、scope、digest、DTO 和测试路径均已闭合；未发现需要推翻架构或会直接产生错误代码的缺口。

## 4. OPTIONAL（不阻断 A0/A1；进入 A2 或首个 PR 时收尾）

- **O1 预览 DTO 的代码块归属可更清楚。** A:160-262 的 §7.1 标题是 `poie-objects`，但 `PlannedActionView`/`ActionPreview` 使用 `plan::SapPreview`，且 A:259 引用 `poie-plan`。A:267 已写明 `SapPreview` 等归 `poie-plan`、`poie-objects` 不依赖 plan；建议再加一句“`ActionPreview/PlannedActionView` 是 host `objects.rs` DTO，不属于 `poie-objects`”，避免实现者误加 `poie-objects -> poie-plan`。这是 A2 代码归属注释，不是 A1 数据契约。
- **O2 `ObjectDetail.evidence` 的填充规则。** A:76 已把 `status_evidence` 标为 `source=evidence`，但 `ObjectDetail.evidence` 是引用列表；MVP 可返回空列表并把 `statusEvidence` 放在属性值里，或定义 `{kind:"report", id:status_evidence, fingerprint:status_evidence, source:"model", asOf:null}`。两种都实现 A1；建议在首个 PR 的二选一中加一条测试。
- **O3 A0 测试清单可补 no-follow 用例。** A:41 已要求 no-follow/原子 rename，A:46 的断言列表只写 gate/sandbox 未登录与 scope；建议 A0 PR 加“根内符号链接指向根外”和“写入中途失败不留下半文件”两个负例。
- **O4 七项检查状态表。** A:282 只把 ownership/reservations/authorization 默认 `assumed`；`AssumedContext`（A:232-235）没有 predicates/state_version/executor_dedup_window，preconditions 与 dedup 只能 `unknown`。A2 实现前给七项逐项状态表（observed/assumed/unknown）并测“不显示 7/7 通过”即可。
- **O5 `planId` 的 host 映射。** A:293 接受“plan object（host 持有）或 planId”；host 只持有当前 Plan。建议明确 `planId == held.plan_id`，不匹配返回 `unknown_object`（与 `plan.preview_sap` 的 Plan-only 规则一致）。
- **O6 A1 呈现层细化。** `ObjectType.label`/`PropertyDef.label` 的人类化、`FunctionType.input_schema/output_schema` 的四个固定 schema、`PropertyValue` 的 quantity/unit 编码可在 `poie-objects` 内用确定性规则定义；不影响 Catalog 的字段/ digest。
- **O7 A2 UI。** 图节点含 `capability`，布局只有 service/process/asset/resource 三列（A:317-318）；补 capability 列或从节点集删除，并定义“当前服务可达”的函数。Playwright 的 mock Catalog fixture（A:319）建议复用 `catalog.golden.json`，避免手写漂移。
- **O8 文档/注释清理。** `check-schema-registry.py` 在 SCHEMAS:4 写“A0”、在 A:392 写“A1 第 1 步”，建议统一为 A1 首 PR；P:181 的 “model store”、`sap.rs:87` 的 “six dispatch checks”、`poie-state/src/lib.rs:31` 的 M12 旧注释可随 A 清理 PR 修。

## 5. Phase A 就绪清单

- [x] **A0 授权语义**：不生成 `Authorization`；execution/federation 的 demo 授权移除；`dispatch_allowed=false`、`authorized/dispatched=0` 写入验收（A:43-46、391）。
- [x] **A0 命令 scope**：`auth_*` bootstrap 例外；`poie-view-execution`/`poie-view-plan` 标签；`sandbox_root` 由 `host_info` 返回、无 `sandbox_list`；`sandbox_*` no-follow + 原子写（A:40-42；PLATFORM:74-90）。
- [x] **单一模型**：`SourceModel = Base-01 + pump-p1 degraded`，host 持 `Arc<Model>`（PLATFORM:95-104；A:38）。
- [x] **Catalog/DTO**：`catalog_digest`、`model_digest`、`EvidenceRef`、`ObjectInstance.version`、`LinkType.cardinality`、`ActionType.duration_s`、`ObjectPage/ObjectDetail/ActionPreview` digest 已定（A:188-262）。
- [x] **预览只读**：`plan.preview_sap` 只收 host Plan；`ontology_action_preview` 基于 `PlannedActionView`；`SapPreview` 归 `poie-plan`；无 `params?`（A:125、249-293）。
- [x] **Digest**：`Asset.description` 删除、`-0.0`/浮点/固定向量、`poie-digest` 进 members/测试矩阵（DIGEST:58、71-72、96-102；A:392）。
- [x] **过程门禁**：ADR（sha2/serde/生成器）、`check-schema-registry.py`、limit 上限（500/page、1 MiB）、根与桌面 `Cargo.lock`、cargo-deny/check-test-matrix/check-docs（A:392、397）。
- [x] **UI/A2 方向**：Catalog 驱动图、删除 `GRAPH_NODES`、Playwright mock/host 分工（A:317-319）；capability 列/fixture 见 O7，属 A2 细化。
- [x] **快照**：以 `0527c7f` 为唯一基线；本报告提交后不再改设计文档，若改则重跑。

## 6. 校验方法、限制与证据

- 方法：只读对照 PO/SAP/VEC、`crates/poie-*`、`apps/desktop/src-tauri`、`apps/desktop/src`、根 workspace、`scripts/`；`python3 scripts/check-docs.py` exit 0；未运行完整 `cargo test`，因此本报告是设计门禁结论，不是实现验收。
- 代码对照：`execution.rs:96-101`（硬编码授权）、`execution.rs:181-197`（旧断言）、`federation_view.rs:85-90`（第二处授权）、`sandbox.rs:38-47`（`std::fs::write`）、`lib.rs:224`（`sandbox_root`）、`auth.rs:81-110`（register/sign_in 引导）、`model.rs:270-272`（`status_evidence` 语义）、`model.rs:304-315`（`Process` 无 description）、`plan.rs:31-43`（`PlannedAction`）。
- 设计证据：[A](phase-a-ontology-product.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)、[README](README.zh-CN.md)；代码链接见 §1/§2。
- 快照限制：校验期间分支从 `55bd48d` 前进到 `0527c7f`；`VALIDATION.zh-CN.md` 仍绑定 `c4bc294`，其结论已部分过期。若 `0527c7f` 之后再提交，必须以新 HEAD 重跑本报告。

**结论：A0/A1 可以开始实现。** 先按 A:391 的 step 0 做 A0（gate/sandbox 鉴权、synthetic preview、测试），再做 `poie-digest`/`poie-objects`/`poie-sdkgen`；§4 的 OPTIONAL 项在对应 PR 收尾即可。

VERDICT: PASS
