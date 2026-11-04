# Phase B 设计门禁复验（独立）

- 校验对象：[Phase B](phase-b-agent-governance.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[Phase A](phase-a-ontology-product.zh-CN.md)、[README](README.zh-CN.md)；代码 `crates/poie-objects`、`crates/poie-plan`、`apps/desktop/src/sdk/ontology.ts`、`apps/desktop/src/App.tsx`、`apps/desktop/src-tauri/src/{auth,db}.rs`、`scripts/`、CI。
- 快照：分支 `design/phase-b-closure`，HEAD **`07f4ee6`**；写本文件前工作树干净。本报告只对该快照负责。
- 方法：只读对照并运行：`cargo test --locked -p poie-objects -p poie-plan`（25 通过）、`poie-sdkgen --check`（exit 0）、`check-schema-registry.py`（24 schemas，exit 0）、`check-docs.py`（exit 0）、`check-test-matrix.py`（380/396，exit 0）、`cargo tree` 依赖边界。未跑全量 `cargo test`，未调用真实 provider/Tauri；未改源码/设计/git，只覆盖本文件。
- 结论：**B1/B3/B7 的核心矛盾已闭合；但 R1/R3/R4/R5/R7 及 B1/B3/B7 的字段/文档残留仍为 REQUIRED，不能开工。**

## 0. 判定矩阵

| 项 | 判定 | 一句理由 |
| --- | --- | --- |
| B1 六工具契约 | PASS* | 六个名称/SDK 已对齐，B 表已有版本/上限；A/B preview 参数仍不一致 |
| B2 只读 SAP preview | PASS* | 只读约束成立；`preview_plan_through_sap` 在 crates 中仍不存在 |
| B3 审批状态机/原子消费 | PASS* | `pending→approved→executing→consumed` 已一致；`started_at` 与崩溃/取消路径仍有缺口 |
| B4 registry/锁/取消 | PASS* | 核心已写；`status` 值域与等待审批时取消未冻结 |
| B5 先审批后执行 | PASS | `plan.propose` auto、`action.draft` confirm、subject 绑定明确 |
| B6 `answer.emit` | PASS* | `EvidenceRef`/digest/`citation_missing` 已写；`evidenceRefs` 仍悬空（R4） |
| B7 审计链/迁移/重建 | PASS* | 记录已含 schema/turnId/toolVersion/cancelled/interrupted；索引迁移仍不完整 |
| R1 错误码/包络 | **REQUIRED** | 缺 `policy_blocked`/`unsupported_schema`，`CommandError` 无 Rust 定义 |
| R2 profile 白名单 | PASS | analyst 无 `action.draft`，dispatcher 不摄入外部文档 |
| R3 bytes/截断 | **REQUIRED** | `max_prompt_bytes` 与 `promptCharsUsed` 冲突，截断标记未入模型 |
| R4 citation/evidenceRefs | **REQUIRED** | 示例已有 sha256；但工具输出/RunState 无 `evidenceRefs` |
| R5 eval 矩阵 | **REQUIRED** | 45 与 6×4×3=72 不符，且 `check-agent-evals.py` 不存在 |
| R6 UI 迁移 | OPTIONAL | 设计目标明确；`App.tsx:1021` 仍用 `inference_complete` |
| R7 schema checker | **REQUIRED** | 只扫 `docs/design`，未实现 SCHEMAS 的规则 3/4 |
| R8 checker 旧文案/preview 归属 | PARTIAL | 旧文案已修；A:275 与 B:81 仍重复声明 preview，A1 符号缺失 |
| R9 capability 生命周期 | PARTIAL | 寿命/会话绑定/撤销已写；无测试，随机来源未定义 |

## 1. 已闭合的核心

- **B1 名称与 SDK**：`crates/poie-objects/src/lib.rs:387-394` 的 `functions()` 已是 `catalog.describe`、`state.query`、`plan.propose`、`plan.preview_sap`、`action.draft`、`answer.emit` 六项，`evidence.get`/`sim.run` 已移除；`apps/desktop/src/sdk/ontology.ts:8` 的 `FunctionId` 与之一致；`poie-sdkgen -- --check` 通过，catalog digest 已更新。B:64-71 现在给出每个工具的 `1.0.0` 版本与每轮上限。A:119-124 也改成同一组六个名称。
- **B3 状态机**：B:106-107 固定 `agent_approve` 的 `pending→approved`（`WHERE state='pending' AND expires_at>?`）与 `agent_resume` 的 `approved→executing` 原子 claim（`WHERE state='approved' AND expires_at>? AND session_id=?`）；B:116 再执行一次并置 `consumed`，并发第二个 resume 得到 `approval_mismatch`（B:109）。PLATFORM:162/166 已改为同一顺序，不再写“`WHERE state='pending'` 原子消费”。B:121 已写 capability 在审批后签发、绑定 session/tool/version/argsSha256/tokenId、单次使用、登出/接管/降级/取消时撤销。
- **B7 审计记录**：B:169-174 已含 `schema`、`turnId`、`callId`、`tool`、`toolVersion`、`decision` 的 `cancelled|interrupted`、`sapRecordId`、`prevHash`/`hash`；B:181 要求从日志重建索引；B:220 已登记 `audit_integrity_failed`。原“日志无法派生 `turn_id`”的矛盾已消除。

## 2. 仍未闭合（REQUIRED，除非注明）

### B1 参数/输出仍未完全同名同参数
- A:122 写 `plan.preview_sap` 输入为 `planFingerprint, assumed context`，B:69 只接受 `{planFingerprint}`；A:126 却坚持“同名/同参数”。`assumed context` 由 host 解析可以成立，但文档没有这样写死，仍是两份冻结契约的差异。
- A:120 的 `state.query` 输出含“能力”，B:67 输出只有 `{objects,links,verdict,dependencies}`；A 的 `FunctionType` 的 `input_schema/output_schema` 仍是 `{"type":"object"}`（`lib.rs:400-401`），没有 B 的 args 结构。B:76 把工具契约放在 `poie-governance::tools`，该 crate 尚不存在；实施前需明确 A Catalog 只承载名称，B 才承载参数/版本。

### B3 共享类型与恢复路径
- B 的 token 表含 `started_at`（B:99），B:107 也写 `started_at=?`；PLATFORM:143-159 的共享 `ApprovalToken` 没有 `started_at_ms`，且用 `created_at_ms/decided_at_ms` 命名。B3 已对齐状态顺序，但共享原语仍不能表达“开始执行时间”。
- B:107 的 `executing` 在进程崩溃后没有恢复规则：B:144 只保留“已批准但未消费”的 token，`executing` 会永久卡住；`agent_cancel` 等待审批时的 token 取消路径也未写（state 枚举无 `cancelled`）。B:107 的“失败→approved”可重试也没有说明重复执行的幂等边界。

### B7 索引与重建等价
- B:180 只列 `turn_id/args_sha256/result_sha256/sap_record_id/prev_hash/hash`，没有覆盖日志记录的 `eventId/schema/sessionId/runId/callId/tool/toolVersion/decision`；现有 `audit` 表是 `at_ms/actor/event/detail`（`db.rs:71-79`），按此迁移无法逐字段重建等价索引。
- B:177 用 `poie.agent-audit/0.1` 作 hash domain；DIGEST §1/§1.2 与 SCHEMAS 的 digest domain 表均未登记该域。B §15 也只写“重建/迁移”，没有 B7 要求的逐字段、含 legacy 段的断言。

### R1 错误码与类型
- B:220 有 17 个码，但缺 PLATFORM:62 的 `policy_blocked`（B/C/D）与 PLATFORM:51 的 `unsupported_schema`（B 的 RunState schema 解析需要）；`preview_unavailable` 也未在 B 侧映射。
- B:60/220 只说 `Result<T, CommandError>`，没有 `CommandError` 的 Rust 类型/`details` 约束；PLATFORM:45 也只有 JSON 形状。R1 仍未闭合。

### R3 计量与截断
- B:129 用 `max_prompt_bytes=4096`，B:140 的 RunState 仍是 `promptCharsUsed`；同一字段在 byte/char 之间自相矛盾。
- B:129 要求截断标记 `truncated=true`，但 B:135-141 的 `toolCalls`/`RunState` 和 B:67-71 的工具输出都没有 `truncated` 字段；`max_context_bytes` 是否含 system prompt、多个 tool results 也未定义。截断算法仍不可实现/测试。

### R4 `evidenceRefs` 悬空
- B:150 的 `Citation` 复用 `EvidenceRef`、B:155 的示例含 `sha256`，digest 部分已修。
- 但 B:159 要求 citation 精确存在于“工具结果的 `citations/evidenceRefs`”；B:67/70/71 的输出没有 citations/evidenceRefs，RunState 只有 `citations[]`（B:137/141）。实现者无法按字面校验。

### R5 eval 覆盖
- B:214 说“至少 45 个”，同一句又要求“6 工具 × 正常/坏参数/超预算/未授权 × 引用/审批/注入”；按笛卡尔积至少 72 例，既无逐用例覆盖表，`scripts/check-agent-evals.py` 在 scripts 目录中不存在。B 的验收 #5（CI 确定性 eval）仍不可验证。

### R7 检查器
- `scripts/check-schema-registry.py:21` 只遍历 `docs/design/**/*.md`，且只做登记检查；SCHEMAS:55 却声称“扫描仓库中的 `schema = "poie.*"` 与 `schemaVersion`”，规则 3/4（多 major 兼容、SDK/Rust 常量一致）均未实现。B:231 还承诺 `check-agent-evals.py`，同样缺失。

### R8/R9 部分残留
- SCHEMAS:4/62 的“当前尚未存在/A0 加入”已修正，R8 的旧文案部分通过。
- 但 A:275 仍写 `Phase A 新增 preview_plan_through_sap`，B:81 又写 `poie-plan` 新增；两份文档重复归属。实际 `crates/poie-plan/src/sap.rs` 只有 `validate_plan_through_sap`（sap.rs:88），`SapPreview`/`AssumedContext` 也不存在，A1 缺口未闭合。B2/B5 的 preview 仍无执行入口。
- R9 的 B:121 已写寿命/绑定/撤销，但 B §15 没有 capability 生命周期测试；`128-bit` 随机来源只覆盖 Session（B:56），未覆盖 capability。

## 3. 已执行的检查

```text
cargo test --locked -p poie-objects -p poie-plan        25 passed
cargo run --locked -q -p poie-sdkgen -- --check         exit 0
python3 scripts/check-schema-registry.py                OK (24 schemas)
python3 scripts/check-docs.py                           OK
python3 scripts/check-test-matrix.py                    OK (380/396)
cargo tree -p poie-plan --edges normal                  no poie-exec
cargo tree -p poie-objects --edges normal               no poie-exec
```

## 4. 边界

本报告是设计门禁审查，不验证真实 provider、Tauri 运行时、SQLite 并发或桌面 Playwright。B0 的 `session_id`/`user_version`/删除启动 DROP 仍未实现（`auth.rs:22-26`、`db.rs:58-89`），按 B §5 属实施第一步，不计为设计阻断；但 `preview_plan_through_sap` 的 A1 缺口和上述 R1/R3/R4/R5/R7 必须在开工前闭合。未改源码/设计/git，只覆盖本文件。

VERDICT: CHANGES_REQUIRED
