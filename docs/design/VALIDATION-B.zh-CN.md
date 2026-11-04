# Phase B 设计门禁复验（独立）

- 校验对象：[Phase B](phase-b-agent-governance.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[Phase A](phase-a-ontology-product.zh-CN.md)、[README](README.zh-CN.md)；代码 `crates/poie-objects`、`crates/poie-plan`、`apps/desktop/src/sdk/ontology.ts`、`apps/desktop/src/App.tsx`、`apps/desktop/src-tauri/src/{auth,db}.rs`、`scripts/`、CI。
- 快照：分支 `design/phase-b-closure`，HEAD **`1cead55`**；写本文件前工作树干净。本报告只对该快照负责。
- 方法：只读对照并运行：`cargo test --locked -p poie-objects -p poie-plan`（26 通过）、`poie-sdkgen --check`（exit 0）、`check-schema-registry.py`（24 schemas，exit 0）、`check-docs.py`（exit 0）、`check-test-matrix.py`（381/397，exit 0）。未跑全量 `cargo test`，未调用真实 provider/Tauri；未改源码/设计/git，只覆盖本文件。
- 结论：**预览入口、六工具名称、审批 started_at、审计域、错误码已实质补齐；但 R5 的 72/45 内部矛盾与缺失 checker、B3 的 executing 崩溃恢复、A/B 的 DTO 与输出契约、B7 索引字段仍为阻断项。不能开工。**

## 0. 判定矩阵

| 项 | 判定 | 一句理由 |
| --- | --- | --- |
| B1 六工具契约 | PARTIAL | 六名称/SDK 已对齐；A 的 `FunctionType` 无参数 schema，A:120 的“能力”输出与 B:67 不一致 |
| B2 只读 SAP preview | PARTIAL | `preview_plan_through_sap` 已实现且不建 `Authorization`；但未在 crate 根导出，局部 ledger 有 Validated 转移，测试未断计数 |
| B3 审批状态机/原子消费 | **REQUIRED** | `started_at`/`executing` 已加；崩溃后 `executing` 恢复、`cancelled` 状态、失败重试幂等仍缺 |
| B4 registry/锁/取消 | PASS* | 核心已写；本次未复跑运行时 |
| B5 先审批后执行 | PASS | `plan.propose` auto、`action.draft` confirm、subject 绑定明确 |
| B6 `answer.emit` | PARTIAL | 协议在；citation 的校验目标仍无工具输出字段承载（R4） |
| B7 审计链/迁移/重建 | **PARTIAL** | DIGEST 域已登记；索引仍缺 `schema`/`reason`/legacy 列，SCHEMAS 域表未列 |
| R1 错误码/包络 | PASS* | `CommandError` 与 `policy_blocked`/`unsupported_schema` 已补；`approval_rejected` 未入 PLATFORM |
| R2 profile 白名单 | PASS | analyst 无 `action.draft`，dispatcher 不摄入外部文档 |
| R3 bytes/截断 | PASS* | `promptBytesUsed`/`truncated` 已对齐；截断算法与 `max_context_bytes` 范围仍留白 |
| R4 citation/evidenceRefs | PARTIAL | RunState `toolCalls` 已有 `evidenceRefs`；工具输出表仍无 citations/evidenceRefs |
| R5 eval 矩阵 | **REQUIRED** | B:211 写 72，B:246 仍写 45；`check-agent-evals.py` 不存在，无 fixture/CI 步骤 |
| R6 UI 迁移 | OPTIONAL | 设计目标明确；`App.tsx:1021` 仍用 `inference_complete`（B0 实施项） |
| R7 schema checker | PARTIAL | 已扫 product code；SCHEMAS:55-60 的规则 3/4 仍未实现 |
| R8 checker 文案/preview 归属 | PARTIAL | preview 归属已单一；A:246-248 的 DTO 形状与代码不符 |
| R9 capability 生命周期 | PASS* | `getrandom` 与测试句已写；§15 仍未列 capability 用例 |

## 1. 已核实闭合

- **六工具/SDK**：`crates/poie-objects/src/lib.rs:387-402` 与 `apps/desktop/src/sdk/ontology.ts:8` 均为 `catalog.describe`、`state.query`、`plan.propose`、`plan.preview_sap`、`action.draft`、`answer.emit`；`poie-sdkgen --check` 通过。
- **A/B 参数**：A:122 的 `plan.preview_sap` 输入已改为 `planFingerprint（host 解析）`，B:69 为 `{planFingerprint}`；B:81 已改为“复用 Phase A 定义”，重复归属消除。
- **审批原语**：PLATFORM:143-160 已有 `state=...|executing|consumed` 与 `started_at_ms`；B:95/103 有 `started_at` 与 `approved→executing` 原子条件更新。
- **审计域**：DIGEST:49 已登记 `poie.agent-audit/0.1`，SCHEMAS:16 已登记同名 wire schema；B:174 的 hash domain 有出处。
- **错误码/类型**：B:220 给出 `CommandError` Rust 定义，B:222 已含 `policy_blocked`/`unsupported_schema`/`citation_missing`。
- **计量/引用字段**：B:125 按字节，B:133 的 `toolCalls` 已含 `evidenceRefs[]`/`truncated`，B:137 已改 `promptBytesUsed`。
- **preview 实现**：`crates/poie-plan/src/sap.rs:190` 已实现；`to_action(..., None)` 不创建 `Authorization`；`preview_is_read_only_and_marks_authorization_assumed` 通过，且测试总数由 25 增至 26。

## 2. 仍未闭合（阻断项在前）

### R5 eval（REQUIRED，最明确）
- B:211 写“至少 **72** 个”，B:246 的 §15 仍写“确定性 eval 45 例”；同一冻结文档内两个数字并存，覆盖矩阵也不存在。
- `scripts/check-agent-evals.py` 缺失（`ls` 报 No such file）；仓库无 `poie.agent-eval/0.1` fixture，CI 无 eval 步骤。B:211/233 的承诺与验收 #5 均不可验证。

### B3 审批恢复（REQUIRED）
- B:103 置 `executing` 后，若进程在执行中崩溃，B:141 只保留“已批准但未消费”的 token 并标 run `interrupted`；`agent_resume` 仅接受 `approved`，`executing` 无恢复/回退规则，会永久卡住。
- B:98 的 state 枚举无 `cancelled`，但 B:141 写 token“可查询/取消”；等待审批期间 `agent_cancel` 的 token 路径未定义（B:143 只覆盖 provider 调用期间）。
- B:103 的“失败→approved 可重试”没有幂等边界；若失败发生在派发后，重复 resume 可能重复执行。started_at 的加入没有解决这三项。

### B1/A 契约（PARTIAL，仍阻断“唯一投影”声明）
- A:120 的 `state.query` 输出含“能力”，B:67 只有 `{objects[],links[],verdict?,dependencies[]}`；若能力即 `objects[]` 中的 capability 对象，文档应写明，否则是两份输出契约。
- A:242-248 的“字段形状”与代码不符：`CheckPreview` 代码字段是 `check`（sap.rs:36-38），A 写 `id`；`AssumedContext` 代码是 `now_ms/holds_command_ownership/reservations_held/predicates_holding/state_version`（sap.rs:27-33），A 写 `ownership/reservations/authorization/now_s`；`PreviewError` 代码是 `EmptyStateVersion/InvalidPlan(String)`（sap.rs:50-53），A 写 `InvalidPlan/UnsupportedContext/NoPreviewAvailable`。
- `poie-objects` 的 `FunctionType.input_schema/output_schema` 仍是 `{"type":"object"}`（lib.rs:400-401）；B:73 的“同参数/唯一投影”没有在 A 侧承载。B:76 的 `poie-governance::tools` 仍是未来 crate。

### B2 preview 细节（PARTIAL）
- A:284/B:82 写“不写 `ActionLedger`、不产生 transition”；实现新建局部 ledger 并执行 `add` + `Validated` 转移（sap.rs:205-212）。无外部持久化副作用，但与字面契约不符。
- A:287/B:86 要求“调用前后 ledger/authorization 计数不变”，测试只断言 `checks.len()==7`、authorization=assumed、无 authorization rejection；没有计数断言。
- A:286 写 ownership/reservations/authorization 默认 `assumed`，代码只把 authorization 标为 assumed，ownership/reservations 按输入判 observed/failed（sap.rs:224-232）。
- `poie-plan/src/lib.rs:17` 只导出 `validate_plan_through_sap/PlanValidationError/SapValidation`；B:69/A:278 的 `poie-plan::preview_plan_through_sap` 实际路径是 `poie_plan::sap::...`。另：按 check 去重会保留首个 action 的状态，后续 action 失败只进 rejections，状态列可能仍是 observed。

### B7 审计（PARTIAL）
- B:166-171 的记录含 `schema/atMs/actor/reason`；B:177 的索引新增字段列表不含 `schema`/`reason`，且“历史行标记 legacy”没有对应列/规则，无法按字面“逐字段比对”。
- SCHEMAS:30-38 的 Digest domains 表仍未列 `poie.agent-audit/0.1`（仅 DIGEST:49 有）；SCHEMAS 自称 DIGEST 域为附加表，但两处表已漂移。
- 当前 `audit` 表仍是 `at_ms/actor/event/detail`（db.rs:72-78），启动仍 `DROP TABLE`（db.rs:58-89）；按 B §5 属 B0 实施第一步，不计设计阻断，但与 B:177 的迁移承诺尚未落地。

### R7 checker（PARTIAL）
- 脚本已加 `poie-objects/src`、`poie-sdkgen/src`、`apps/desktop/src/sdk` 三处（scripts/check-schema-registry.py:23-38），R7 的“只扫 docs/design”已修。
- 但脚本只做“是否登记”；SCHEMAS:55-60 的规则 3（多 major 兼容/迁移说明）与规则 4（SDK 与 Rust 常量一致）没有实现，`schemaVersion` 也未扫描；SCHEMAS:4/62 的表述仍强于实际。

### R1/R4/R6/R9 的余项（OPTIONAL，非阻断）
- R1：B:222 的 `approval_rejected` 未进 PLATFORM 表；B 未映射 PLATFORM:55-56 的 `preview_unavailable`/`invalid_parameter`。
- R4：B:156 要求 citation 存在于“工具结果的 citations/evidenceRefs”，但 B:64-71 的输出表仍无这两个字段；`toolCalls` 有字段，却没有“由工具输出生成 evidenceRefs”的规则。
- R6：`apps/desktop/src/App.tsx:1021` 仍调用 `inference_complete`；B:226/验收 #7 的迁移尚未开始。
- R9：B:117 已写 `getrandom` 与测试句，但 §15 的测试清单未列 capability 用例；`getrandom` 也只在桌面 `Cargo.toml:34`，不在 B:231 的依赖清单。

## 3. 已执行的检查

```text
cargo test --locked -p poie-objects -p poie-plan   26 passed (3 + 19 + 1 + 3)
cargo run --locked -q -p poie-sdkgen -- --check    exit 0
python3 scripts/check-schema-registry.py           OK (24 schemas)
python3 scripts/check-docs.py                      OK
python3 scripts/check-test-matrix.py               OK (381/397)
ls scripts/check-agent-evals.py                    not found
```

## 4. 边界

本报告是设计门禁审查，不验证真实 provider、Tauri 运行时、SQLite 并发或桌面 Playwright。B0 的 `session_id`/`user_version`/删除启动 DROP 仍未实现（auth.rs:22-26、db.rs:58-89），按 B §5 属实施第一步，不计设计阻断；但 R5 的 72/45、B3 的 executing 恢复、A/B 的 DTO/输出契约与 B7 的索引字段必须在开工前闭合。未改源码/设计/git，只覆盖本文件。

VERDICT: CHANGES_REQUIRED
