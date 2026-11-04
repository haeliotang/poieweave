# Phase B 设计门禁复验（独立）

- 校验对象：[Phase B](phase-b-agent-governance.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[Phase A](phase-a-ontology-product.zh-CN.md)、代码 `crates/poie-plan`、`crates/poie-objects`、`apps/desktop/src/sdk`、`scripts/`、根 CI。
- 快照：`design/phase-b-closure`，HEAD **`b3aad29`**；写本文件前工作树干净。只读校验；未改源码/设计/git，只覆盖本文件。
- 方法：对照文档与代码并运行：`cargo test --locked -p poie-objects -p poie-plan`（26 通过）、`cargo fmt --all --check`、`poie-sdkgen --check`、`check-schema-registry.py`、`check-docs.py`、`check-test-matrix.py`、`check-agent-evals.py`。未跑全量 `cargo test`，未调用真实 provider/Tauri。
- 结论：**B3 八态审批/恢复、B7 索引字段、A preview DTO、六工具函数 schema、72 矩阵已闭合；但 R5 checker 仍是空转且未接 CI、R7 四条规则仍未实现、B2 仍保留与代码相反的只读/assumed 表述。不能开工。**

## 0. 判定矩阵

| 项 | 判定 | 一句理由 |
| --- | --- | --- |
| B1 六工具/参数 schema | PASS* | 六工具 schema 已填充且与 B 表一致；A:120 的 `subject/type` 仍与代码 `subject` 不一致（次要） |
| B2 只读 SAP preview | **REQUIRED** | B:83/255“不写 ledger/不推进 ledger”与 `sap.rs:205-214` 的局部 ledger + `Validated` 相反；B:84/A:287 的 assumed 语义也与代码相反 |
| B3 审批状态机/恢复 | PASS | 八态、租约恢复、幂等键、PLATFORM:155 与 §15 测试均已补；仅余租约窗口边界（OPTIONAL） |
| B5 先审批后执行 | PASS | `plan.propose` auto、`action.draft` confirm、subject 绑定明确 |
| B6 `answer.emit` | PASS* | 协议在；citation 的校验目标仍未在工具输出表落字段（R4，OPTIONAL） |
| B7 审计链/迁移 | PASS | B:179 已含 `schema/reason/legacy`；SCHEMAS:39 已登记 audit 域；wire+digest 双表登记为可接受的双用途 |
| R1 错误码/包络 | PASS | `approval_rejected` 已入 PLATFORM:57，B:235 同步 |
| R5 eval 矩阵 | **REQUIRED** | 72 矩阵/§15 已改对；但 checker 在 fixture 缺失时直接 exit 0，未接 CI/verify-all，且不校验 `[approval]` |
| R7 schema checker | **REQUIRED** | SCHEMAS:4/56-62 声称四条规则与 `schemaVersion` 扫描；脚本只做登记，规则 3/4 与 `schemaVersion` 均未实现 |
| R9 capability | OPTIONAL | §15 已列 capability 用例；`getrandom` 仍未入 B:244 依赖清单 |
| R6 UI 迁移 | OPTIONAL | `App.tsx:1021` 仍用 `inference_complete`，属 B0 实施项 |

## 1. 阻断项

### R5（REQUIRED）：checker 存在但不构成门禁
- `scripts/check-agent-evals.py:20-22` 在 `evals/agent` 不存在时打印“not present yet”并 `return 0`；仓库当前无 `evals/agent`，实跑 exit 0。
- `.github/workflows/ci.yml` 与 `scripts/verify-all.sh` 均无该脚本调用；B:260/268 的“CI 确定性 eval 72 例/CI eval 绿”没有对应步骤。
- B:224 声称 checker 校验“审批脚本”，但脚本只读 `[script].responses`，完全忽略 `[approval]`；`seen` 用 `path.stem`，重复 id 检查实际不可触发。
- 因此 72 例覆盖（B:213/260）仍不可执行验证；这是新增的“checker 已校验”错误陈述，不是闭合。

### R7（REQUIRED）：SCHEMAS 的四条规则未实现
- `scripts/check-schema-registry.py:14` 只匹配字面量 `poie.*/x.y` 并做“是否登记”；无 `schemaVersion` 扫描、无“同 domain 多 major 兼容/迁移说明”、无生成物/SDK 与 Rust 常量一致性比较（规则 3/4）。
- `git diff 1cead55..b3aad29 -- scripts/check-schema-registry.py` 为空；上轮“规则 3/4 未实现”的 finding 未修。SCHEMAS:4/56-62 仍强于实际。
- 扫描范围也不含 `apps/desktop/src-tauri`（`objects.rs:59` 的 action-preview 字面量靠人工登记）。

### B2（REQUIRED）：B/A 仍保留与代码相反的 preview 语义
- B:83“不写 `ActionLedger`，不产生 transition”与 B:255“preview 不推进 ledger”：`sap.rs:205-214` 明确 `ActionLedger::new()`、`add`、`apply(Validated)`；A:285 已改为“throwaway ledger…不产生 Authorized transition”，B 未同步。
- B:84 与 A:287“ownership/reservations/authorization 默认 `assumed`”：`sap.rs:236-242` 只把 `Authorization` 标 `assumed`；ownership/reservations 按输入成为 `observed` 或 `failed`。
- 外部副作用确为零（不建 `Authorization`、不写持久 ledger），但 B2 的冻结文字与实现不符，属于上轮已点名的残留。

## 2. 次要/OPTIONAL

- A:120 `state.query` 输入写 `subject/type`，代码 `poie-objects/src/lib.rs:411-413` 与 B:67 均为 `{subject,asOf?}`；A:126 又声称“同参数”。建议统一为 `subject`。
- B:69 的 `SapPreviewDto` 未在 A/代码中定义，且只列 `checks/assumed/rejections`，丢掉 `SapPreview` 新增的 `authorized_actions/authorization_created`；B:86 仍要求“计数不变”测试，A:288 已改为字段断言。建议定义 DTO 或直接引用 `SapPreview`。
- B3 租约边界：启动恢复只在启动时检查 `started_at`；120s 内重启会留下 `executing` token，之后无定时/`agent_status` 触发的转移，`agent_resume` 也不接受 `executing`。属恢复时序边缘，不是永久卡死。
- B §10 审计：`interrupted` 由启动恢复写入，但 `actor` 枚举仍只有 `operator|agent`；`legacy` 列无取值/比对规则。
- R4/R6/R9：citation/evidenceRefs 未进 B:64-71 工具输出表；`App.tsx:1021` 仍是旧入口；`getrandom` 未入 B:244 依赖清单。

## 3. 已核实闭合

- **B3**：`state ∈ pending|approved|rejected|expired|executing|interrupted|consumed|cancelled`（B:98、PLATFORM:155）；`agent_cancel`、`interrupted→executing/cancelled`、`(runId,callId)` 幂等键、§15 的崩溃重试用例均在。
- **B7**：B:179 索引字段含 `schema/reason/legacy`；SCHEMAS:16 与 :39 分别登记 wire schema 与 digest 域；DIGEST:49 有名。
- **A preview**：`CheckPreview.check`、`AssumedContext{now_ms,holds_command_ownership,reservations_held,predicates_holding,state_version}`、`PreviewError{EmptyStateVersion,InvalidPlan}` 与 `sap.rs:27-55` 一致；`lib.rs:17-20` 已 re-export；测试断言 `authorized_actions==0`、`authorization_created==false`。
- **六工具**：`poie-objects/src/lib.rs:387-441` 六个 schema 已填充，input 均 `additionalProperties:false`；`poie-sdkgen --check`、`cargo fmt --check` 通过。
- **72**：B:213 矩阵 6×4×3=72，B:260/§15 与之一致；“72/45 并存”已消除。

## 4. 已执行检查

```text
cargo test --locked -p poie-objects -p poie-plan   26 passed (3+19+1+3)
cargo fmt --all --check                            exit 0
cargo run --locked -q -p poie-sdkgen -- --check    exit 0
python3 scripts/check-schema-registry.py           OK (24 schemas)
python3 scripts/check-docs.py                      OK
python3 scripts/check-test-matrix.py               OK (381/397)
python3 scripts/check-agent-evals.py               "not present yet", exit 0
```

## 5. 边界

本报告是设计门禁审查；不验证真实 provider、Tauri 运行时、SQLite 并发或桌面 Playwright。B0 的 `session_id`/`user_version`/删除启动 DROP 仍未实现（`auth.rs`、`db.rs`），按 B §5 属实施第一步，不计设计阻断。R5/R7/B2 三项须在开工前修正；其余为 OPTIONAL。

VERDICT: CHANGES_REQUIRED
