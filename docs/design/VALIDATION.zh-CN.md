# Poieweave Phase A–D 设计校验（最终门禁）

- 校验对象：[README](README.zh-CN.md)、[Phase A](phase-a-ontology-product.zh-CN.md)、[Phase B](phase-b-agent-governance.zh-CN.md)、[Phase C](phase-c-what-if.zh-CN.md)、[Phase D](phase-d-site-delivery.zh-CN.md)、[DIGEST](DIGEST.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md) 与 `research/` 三份备忘。
- 代码基线：`design/phase-a-d`，HEAD **`c4bc294`**；工作树除本报告外干净。设计文档已入库。
- 文档快照：2026-11-04 23:07（本地）。校验期间文档经历 `4a90edc → 45a2a93 → f75c31d → 55bd48d → 91beea9 → c4bc294` 多轮修订；本报告只对 `c4bc294` 负责。
- 方法：只读对照 PO/SAP/VEC、`crates/poie-*`、`apps/desktop/src-tauri`、`apps/desktop/src`、根 workspace、CI 与 `scripts/`。`python3 scripts/check-docs.py` 与 `scripts/check-test-matrix.py` 均 exit 0；未跑完整 `cargo test`，未修改应用源码、设计文档或 git 状态（本文件除外）。
- 结论：方向正确；A0 的假授权、schema 单一来源、StateId、Predicate、PreviewError、dispatch_allowed、scope 主行已在 `91beea9`/`c4bc294` 关闭，R1–R5 大部分实质闭合。但 **Phase A 仍有 4 组可定点修复的契约阻塞**：ActionType/ActionPreview 的建模、SapPreview 的 crate 归属、A0 的 `auth_*`/窗口矩阵残余，以及 `status_evidence` 的 provenance 语义。当前不能开工。

## 0. 总判定

**CHANGES_REQUIRED。** 这不是对架构方向的否决。单一 SAP 执行点、只读 Catalog、`SourceModel` 单一来源、C 的 “只创建 proposed”、D 的 “元数据 bundle 不替换 app/模型” 都成立；剩余问题是 A0/A2 会直接变成错误代码或不可测验收的接口细节，且都是局部的。

关键判断：

1. **R1 仍未完全闭合。** 删掉 `risk_class/approval_class/budget_template/executor_kind/allowed_in_modes` 与 `lot.location/evidence` 是对的；但 `ActionType.label/description` 不在已验证 `Process` 中，`ActionPreview` 仍是从 `PlannedAction` 借字段（`plan_ref`/`preconditions`），而命令只接收 `actionTypeId`。这仍是“Catalog 字段没有模型来源”。
2. **R3 在声明层闭合、在 API 层未闭合。** 不再调用 `validate_plan_through_sap`、不再显示“7/7 通过”，这两点已做到；但 `ontology_action_preview(actionTypeId)` 无法调用 `preview_plan_through_sap(&Plan)`，且 `SapPreview` 的 crate 归属与依赖方向未定。
3. **R4 的 A0 授权语义已正确，但命令矩阵仍有残余。** A:43–46 已改为“不生成任何 Authorization”且 `dispatch_allowed=false`，这是实质进步；但 A:42 要求未登录拒绝 `auth_*`，PLATFORM:74 又把注册/登录列为免登录；PLATFORM:83 的 `poie-view-planning` 与实际 `poie-view-plan` 不符。
4. **A0 的 `status_evidence` 极性写反。** A:76 说“有证据→declared、无证据→合成观测”，与 `model.rs:270–272` 和 PO §3.1 正好相反。它会把有证据的属性标成最弱等级，把无证据的属性标成观测。

## 1. 六个最强点

1. **A0 的授权语义已纠正**：`execution.rs`/`federation_view.rs` 的硬编码 `Authorization` 移除；execution 只显示 proposed/validated 与 synthetic-preview，`dispatch_allowed` 恒为 `false`，测试断言 `authorized`/`dispatched` 为 0（A:43–46）。这关闭了上一轮“预览即授权”的核心风险。
2. **单一执行点守得住**：A/B/C 只生成 proposed/草稿；B 的模型工具目录无 `dispatch.request`/`shell.*`/`secrets.*`（B:94–101）；`gate`/`adapter` 仍只在 host/SAP 侧；`preview_plan_through_sap` 只读（A:266–276）。
3. **Digest 规范可落地**：`poie-digest` + canonical JSON + domain 分离，明确 fingerprint 与 SHA-256 不可互冒充（DIGEST:29–40），覆盖 model/catalog/plan/simulation/variant/manifest/approval/evidence/contracts；`-0.0`、整数值浮点、键序不变性与固定 fixture 均进入验收（DIGEST:58、94–102）。
4. **跨阶段契约已集中**：`PLATFORM` 定义 DTO/错误包络/命令 scope/`SourceModel`/`ApprovalToken`；`SCHEMAS` 成为唯一注册表（PLATFORM:15 已删内联表）；`PreviewError` 已映射，`StateId` 已改为 DTO 字符串，`Predicate` 已删除（A:244–247、A:252）。
5. **C 的诚实性明显提高**：明确 MVP 只针对静态 Base-01 演示快照，删除“live 不变”的假阳性；`OutcomeKind.structural/timed` 分离并带 caveat，禁止把 `candidate` 显示成 `feasible`（C:66、100–118、154）。
6. **D 的边界克制且完整**：不做 updater/遥测/SBOM 强制门禁/模型导入；bundle 是 JSON 元数据，不替换二进制；单一版本源、`host_info.build`、`delivery_plan/recall/channel`、`write_atomic` no-follow、tag release 依赖根 verify 都已写入（D:20–40、82–88、162–205）。

## 2. REQUIRED（Phase A 开工前必须修复）

### RA1 ActionType/ActionPreview 仍混用两种模型，且有非模型字段

- A:106 的字段表写 `id, label, description, target_types`；A:202–206 的 `ActionType` 也有 `label/description`。
- 已验证的 `Process` 只有 `id/inputs/outputs/occupies/requires_capabilities/restores/duration_s/verification/asset_mass_change_kg/mass_tolerance/process_yield`（`crates/poie-ontology/src/model.rs:304–315`），**没有 `label/description`**；只有未验证的 `ProcessDoc.description`（`model.rs:197–200`）。读 `Model.doc` 会绕过 `Model::from_doc` 校验，正是 R1 禁止的第二套投影。
- A:248–253 的 `ActionPreview` 仍带 `plan_ref`、`preconditions: Vec<String>`；这两者属于计划实例 `PlannedAction`（`crates/poie-plan/src/plan.rs:31–43`），不属于 `ActionType`。`PlannedAction.preconditions` 是 `Vec<String>`（plan.rs:38），而 `Process` 没有前置条件；所谓“来自 plan/model”实际只能来自 plan。
- A:285 的 `ontology_action_preview` 只接收 `actionTypeId`，但 A:276 要求它调用 `preview_plan_through_sap(&Plan)`；A:125 又规定 `plan.preview_sap` 只接受 host 持有的 `Plan`（已删除 planRef）。actionTypeId 与 Plan 之间没有定义转换，`params?` 在删除通用参数后也没有定义。

**最小修复（二选一）**：
1. 保留 ActionType 预览：新增 `preview_action_through_sap(actionTypeId, assumptions)`，由 host 构造单个 `Action`（target/runs/validity 显式 nullable），只跑七项检查、不建 plan；或
2. 改成 Plan 预览：`ontology_plan_preview(planRef, assumedContext)`，plan 由 `plan.propose`/现有规划产生，`ActionPreview` 只作为 plan 内动作的行视图（`plan_ref` 有值，`preconditions` 来自 `PlannedAction`）。

同时：MVP 删除 `label`/`description`，或规定 `label=id` 的确定性人类化、`description=None`，明确不得读 `ProcessDoc`。为“每个 `ActionType` 字段逐项来自已验证 `Model`”加黄金 fixture。完成后 R1 才算闭合。

### RA2 `SapPreview` 的 crate 归属与依赖方向未定

- A:239–243 在 `poie-objects` 的 API 清单里定义 `SapPreview`；A:253 的 `ActionPreview` 直接包含它；A:266 的 `poie-plan::preview_plan_through_sap` 又返回它。
- PLATFORM:169–178 的依赖表是 `poie-objects -> ontology/digest`、`poie-governance -> objects/state/plan`，**没有 `poie-plan -> poie-objects`**；A:276 又要求 `poie-objects` 不依赖 `poie-plan`。两边都成立时，`SapPreview` 只能被复制成两份，违反 PLATFORM §2 的“单一公开 DTO”。
- 若让 `poie-plan` 依赖 `poie-objects`，是产品层反向依赖领域规划层，且未在依赖表/ADR 中声明。

**最小修复**：把 `SapPreview/CheckPreview/AssumedContext` 定义在 `poie-plan`（它拥有七项检查和 plan 输入）；host 组合 `ActionPreview` 时引用 `poie_plan::SapPreview`；`poie-objects` 不持有 plan 类型。或者显式增加 `poie-plan -> poie-objects` 依赖并解释分层，二选一。

### RA3 A0 的 `auth_*` 白名单、窗口矩阵仍有残余

- A:42 要求“未登录调用 `execution_gate_set`、`sandbox_write`、`inference_configure`、`auth_*` 必须拒绝并留痕”。PLATFORM:74 却把 `auth_status/register/sign_in/sign_out` 全部标为“登录：否”。`auth_register`/`auth_sign_in` 是引导入口（`apps/desktop/src-tauri/src/auth.rs:74–110`），按 A 实现会让新装机器永远无法注册/登录。
- PLATFORM:83 给 `whatif_*` 的允许窗口是 `main/poie-view-planning`，但 `open_workspace_window` 对 view `plan` 生成的标签是 `poie-view-plan`（`apps/desktop/src-tauri/src/lib.rs:139–143`）。仓库没有 `poie-view-planning`。
- `sandbox_root`（lib.rs:224）仍未列入矩阵；A0 的拒绝测试若按文字实现，会在错误窗口/未列命令上产生盲区。

**最小修复**：A:42 改为“除 `auth_status/register/sign_in` 三个引导命令外，未登录的 `auth_sign_out`/`audit_list` 等拒绝”；PLATFORM §4 使用 `poie-view-plan`（或让窗口用 `planning`）；补 `sandbox_root`。A0 的拒绝测试按命令逐项列出，而不是 `auth_*` 通配。

### RA4 `status_evidence` 的 source 极性写反

- A:76 写“`Asset.status` 有 `status_evidence` 时 `source=declared` 并带证据字符串；没有时为合成观测”。
- 代码与 PO §3.1 的语义相反：`status_evidence: None` 才表示 **declared / synthetic observation**（`crates/poie-ontology/src/model.rs:270–272`）；有证据引用时是证据支撑的状态，应标为 `observed`/`evidence`，并尽量带 `asOf`。
- 这会把“有证据”的属性标成最弱等级，把“无证据”的属性标成观测，直接违反 Phase A 的属性证据目标（A:166）与 UI 的“声明/合成”提示。

**最小修复**：改为“有 `status_evidence` → `source=observed`（或新增 `evidence`），`asOf` 取证据时间（未知则 null）；无 `status_evidence` → `source=declared`、`asOf=null`、UI 显示声明/合成”。为两种分支各加 fixture。

## 3. 后续阶段门禁（不阻塞 A，但不能带进 B/C/D）

- **C 的 `ProposedActions`/`plan_to_proposed` 签名未定。** C:194 只给名字与 `basis` 编码；`ProposedActions` 的字段、每个 action 的 `ActionState`、由谁写 ledger 未定。C:149 又说 C 不写 SAP 记录/ActionLedger；若只返回内存对象，后续“交给现有 SAP 验证命令”需要写入点。**C1 前必须定。**
- **C 的 promote 审批 subject 未定义。** `whatif_variant_promote(variantId, approvalTokenId)`（C:171）使用 PLATFORM 的 `ApprovalToken`，但没有像 B:173–178 那样列出 `snapshotSha256/variantResultSha256/planSha256/policyVersion/sessionId/expiresAt`。没有这些，审批无法绑定具体结果。
- **C 的 `delta` 无类型、`unknown_snapshot` 未登记。** C:177 的 `delta` 无结构，C:187 的 `unknown_snapshot` 不在 PLATFORM 错误码表（PLATFORM:48–62）。
- **B 的 `Session` 没有 id。** `auth::Session` 只有 `display_name`/`expires_at_ms`（auth.rs:23–26）；B 的 capability 需要 `sessionId`（B:161），PLATFORM 的 `ApprovalToken` 需要 `session_id`（PLATFORM:139–153）。必须先给 Session 加不可猜测 id 与 epoch，并定义登出/接管时 capability/token 的撤销。
- **B 的审批 token 没有表/迁移。** B:237 只定义 `agent_runs`；PLATFORM:155 说“所有 token 持久化（SQLite）”，但没有 `approvals` 表与 `user_version` 迁移。否则暂停/恢复只能靠内存。
- **B 的 `citation_missing`/`not_implemented` 未登记。** B:122 使用 `citation_missing`，B:99 使用 `not_implemented`，PLATFORM:48–62 没有；错误码表与实现会分叉。
- **B 的“既有 SAP 验证/授权命令”不存在。** B:317 说审批通过后由既有命令接管；桌面 host 没有 `sap_authorize`/`sap_validate` 命令，A 只做 preview，C 才加 `plan_to_proposed`。应明确由后续 SAP 命令/UI 接管，或补命令契约，不能写成“既有”。
- **B 的 HTTP 响应上限缺失。** 现有 HTTP 传输只有 30s 超时（http.rs:23–25）；B 的多步工具调用会接收 provider 响应，必须定义最大字节/超时/截断测试，否则“预算”只覆盖步数不覆盖内存。
- **D 的 `contracts.toml` 只有一句。** D:88/194 要求 id/version/revision/sha256，但没有文件 schema、生成方式、黄金样例与“契约正文变更时如何更新 revision”的规则；D:68–71 的 manifest 示例又手写三个条目。
- **D 的 `release.json`/`RELEASE.json` 大小写不一致。** D:87/152 用 `release.json`，D:199/275 用 `RELEASE.json`；构建期文件名必须唯一。
- **D 的召回不阻止继续使用。** D:28 目标写“召回阻止继续使用”，D:122 只写“已安装则显示召回+允许回滚”；没有启动/导入/派发任一执行点的拒绝规则。
- **D 的导入策略下限未定。** D:173 的 `delivery_policy_set` 与导入 bundle 的 `policy` 可覆盖 `agentScopes/remoteInference`；D:268 只说“MVP 允许，需审批/审计”。这是安全边界，必须定义不可放宽的下限（例如 `remoteInference` 只能从 true→false）与测试。
- **D 的 `HostPolicy.channel` 与 `DeliveryState.channel` 双源。** PLATFORM:91 把 `channel` 放进 HostPolicy，D:96–105 又把 channel 放在 DeliveryState 并声明“本地订阅优先”；导入 manifest 时会出现三方冲突。
- **D 的 `ModelStore` 术语残留。** D:35 仍写“构建期/本地 `ModelStore` 提供模型”，PLATFORM:104 明确“不存在 ModelStore 与 SourceModel 两个版本”，D:193 也改为 `SourceModel`。统一为 `SourceModel`。
- **DIGEST 的 catalog 排除项措辞。** DIGEST:21 写“Catalog（不含 digest 字段）”，A:215–217 明确只排除 `catalog_digest` 自身、保留 `model_digest`。建议同步为“不含 `catalog_digest` 字段；`model_digest` 纳入”，否则两种实现会得到不同 hash。
- **研究备忘仍是旧词表。** `research/agent-governance.zh-CN.md:98` 仍主张七个工具含 `dispatch.request`，`:125` 仍用 `poie-eval`；`repo-gap-audit.zh-CN.md:104/113/116` 仍用 `poie-agent`/`auto_read`。建议加一行“以 c4bc294 设计决定为准”，避免新读者把提案当契约。

## 4. R1–R5 闭合表

| 编号 | 判定 | 依据 |
| --- | --- | --- |
| R1（Catalog 无来源字段） | **部分** | `risk_class/approval_class/allowed_in_modes/executor_kind/budget_template` 已移入 B 并 fail-closed（A:112）；`lot.location/evidence` 已排除（A:68）。但 `ActionType.label/description` 不在已验证 `Process`，`ActionPreview.preconditions` 仍只能来自 `PlannedAction`；见 RA1。 |
| R2（digest 规范） | **闭合** | `DIGEST.md` 定义 domain、canonical JSON、fingerprint/SHA-256 分离、manifest 自哈希、固定 fixture 与键序测试；`poie-digest` 已列入 PLATFORM:169–178。唯一建议：DIGEST:21 的 catalog 排除项写清楚（见 §3）。 |
| R3（预览不授权） | **部分** | 已删除 `validate_plan_through_sap` 调用与“7/7 通过”，新增 `SapPreview`/`assumed`/`rejections`（A:235–276）；A0 不再生成 Authorization（A:43–46）。但 ActionType 级命令无法调用 Plan 级函数（RA1），`SapPreview` 归属未定（RA2）。 |
| R4（A0 宿主鉴权） | **部分** | A0 已成为实施第 0 步，覆盖 `execution_gate_set` 先鉴权、`sandbox_*`、demo 授权替换（A:36–46、383）；`execution_gate_set` 窗口与 `sandbox_list` 已修正。但 `auth_*` 白名单与 PLATFORM:74 冲突、`poie-view-planning` 标签不存在（RA3）。 |
| R5（README 门禁对齐） | **闭合** | README:30 同时引用 FAST 与最终报告，并明确“Lead 不要求固定结论”；README:41、99–120 记录 R1–R18；README:80 已改为桌面内消费者；P5 已改为“C 不写 live/ledger；promote 只经 `plan_to_proposed` 产生 proposed”。状态行 README:126–127 仍写“VALIDATION → CHANGES_REQUIRED / 待重跑”，本报告提交后应由 Lead 更新为最终结论。 |

## 5. Phase A 就绪清单

- [x] A0 授权语义：不生成 `Authorization`，execution/federation 的 demo 授权移除，`dispatch_allowed=false`，测试断言 authorized/dispatched 为 0（A:43–46）。
- [x] 单一模型：`SourceModel = Base-01 + pump-p1 degraded`，host 持有 `Arc<Model>`（PLATFORM:95–104；A:38）。
- [x] 单一执行路径：Catalog 只读、`poie-objects` 不依赖 plan、预览不 authorize（A:164–168、263–276）。
- [x] Digest：`model_digest`/`catalog_digest` 命名一致，canonical/domain/manifest 规则齐全（A:214–217；DIGEST:20–40、89）。
- [x] 对象 ID/EvidenceRef：`<type>:<id>` 与 `EvidenceRef` 统一（PLATFORM:106–122）。
- [x] SDK 唯一生成路径、零 diff 退出码、黄金文件（A:271–300）。
- [x] 六个旧视图、Playwright mock/host 策略、i18n、`check-docs`/`check-test-matrix` 已列入验收。
- [x] schema 单一来源（SCHEMAS）、`StateId`、`PreviewError`、`ObjectDetail`/`SapPreview` 登记已收尾。
- [ ] `ActionType` 的每个字段都能追溯到已验证 `Model`（`label/description` 待处理；RA1）。
- [ ] ActionType 预览与 Plan 预览的 API 二选一已冻结（RA1）。
- [ ] `SapPreview` 的 crate 归属/依赖方向已定（RA2）。
- [ ] A0 的 `auth_*` 例外、`poie-view-plan` 标签与 `sandbox_root` 已修正（RA3）。
- [ ] `status_evidence` 的 source/asOf 极性已修正（RA4）。
- [ ] 上述修改提交为新 commit，并以该 commit 重跑本报告（当前 HEAD `c4bc294` 不含这些修订）。

## 6. 结论

**当前不能开始 Phase A 编码。** 需要修的是一组局部契约，不是推翻设计：

1. 冻结“ActionType 预览”与“Plan 预览”二选一；删除 `label/description` 或给出确定性派生；把 `plan_ref`/`preconditions` 移到 plan 预览。
2. 把 `SapPreview/CheckPreview/AssumedContext` 放到 `poie-plan`，host 组合 `ActionPreview`，消除复制/反向依赖。
3. A0 修正 `auth_*` 例外、`poie-view-plan` 标签与 `sandbox_root`；按命令逐项列拒绝测试。
4. 修正 `status_evidence` 的 source/asOf 语义。

完成这 4 组后，A0/A1 可以开工；B/C/D 的 §3 门禁项必须在各自阶段开始前关闭。**不要**在未重跑本报告的情况下把本节改成 PASS。

VERDICT: CHANGES_REQUIRED
