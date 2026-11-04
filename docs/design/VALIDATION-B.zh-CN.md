# Phase B 设计门禁复验（独立）

- 校验对象：[Phase B](phase-b-agent-governance.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[Phase A](phase-a-ontology-product.zh-CN.md)、代码 `crates/poie-plan`、`crates/poie-objects`、`apps/desktop/src/sdk`、`evals/agent`、`scripts/`、根 CI。
- 快照：`design/phase-b-closure`，HEAD **`3301181`**；写本文件前工作树干净。只读校验；未改源码/设计/git，只覆盖本文件。
- 方法：对照文档与代码并运行：`cargo test --locked -p poie-objects -p poie-plan`（26 通过）、`cargo fmt --all --check`、`cargo clippy -p poie-plan --all-targets -- -D warnings`、`poie-sdkgen --check`、四个 Python checker；未跑全量 `cargo test`，未调用真实 provider/Tauri。
- 结论：**B3/B7/R7 与 preview 代码已闭合；但 R5 的 72 个 fixture 仍是占位用例，且 18 个 approval fixture 的 `[approval]` 实际落在 `[script]` 下，checker 从未校验审批动作；B2 的 §15 仍保留与实现相反的 ledger/计数文字。不能开工。**

## 0. 判定矩阵

| 项 | 判定 | 一句理由 |
| --- | --- | --- |
| B2 只读 SAP preview | **REQUIRED** | §6.1/A:285 已改 throwaway；B:255 仍写“不推进 ledger”，B:86/A:375 的计数不变无测试 |
| B3 审批状态机/恢复 | PASS | 八态、租约恢复、幂等键、PLATFORM:155 与 §15 用例均在 |
| B7 审计链/迁移 | PASS | B:179 索引含 `schema/reason/legacy`；SCHEMAS 已登记 audit 域 |
| R5 eval 矩阵 | **REQUIRED** | 72 文件/CI 已加；approval fixture 放错表导致审批校验空转，且用例无场景断言 |
| R7 schema checker | PASS | 规则 3 多 major 已实现；规则 4 由 `poie-sdkgen --check` 承担且 CI 已接 |
| R6/R9/其余 | OPTIONAL | 旧 `inference_complete` 入口、`getrandom` 依赖清单等仍为实施项 |

## 1. 阻断项

### R5（REQUIRED）：72 个 fixture 存在，但审批校验未触发、场景断言为空

- 18 个 `--approval/--reject/--expiry` fixture 把 `approval = { action = "..." }` 写在 `[script]` 之后、`[expect]` 之前；TOML 将其解析为 `script.approval`，不是顶层 `approval`。逐文件用 `tomllib` 检查：72 个文件的顶层 `approval` 全为 `None`，`scripts/check-agent-evals.py:46` 的 `data.get("approval")` 分支一次也未执行。B:205-206 的 fixture schema 是顶层 `[approval]` 表，当前 fixtures 违反了该 schema，checker 也未拒绝。
- 全部 72 个文件的 `[script].responses` 均只有一条 `args = {}` 调用，`[expect]` 基本只有 `tools`；`bad-args/budget/citation/approval/injection` 等文件名没有对应的错误、预算、citation 或审批断言。12 个变体仅文件名和 prompt 不同，B:215-222 的 3×4×6 矩阵没有真正编码；“计数/工具覆盖”通过是名义上的。
- `scripts/check-agent-evals.py:20-22` 在 `evals/agent` 目录不存在时仍打印 “not present yet” 并 `return 0`；删掉整个目录后 CI 依旧绿。CI 步骤已加（`.github/workflows/ci.yml:43`），但该硬编码旁路仍在，`scripts/verify-all.sh` 也仍未调用本脚本。

### B2（REQUIRED）：§15 仍与 throwaway ledger 实现冲突

- B:83 与 A:285 已改为“函数内 throwaway ledger…不写 live/external ledger、不产生 Authorized transition”，代码 `crates/poie-plan/src/sap.rs:205-213` 确实只新建局部 `ActionLedger` 并 `add`/`apply(Validated)`，不创建 `Authorization`；这部分已对齐。
- 但 B:255 仍写“preview 不创建 Authorization/不推进 ledger”，未加 throwaway/live 限定；按字面，代码明确推进了局部 ledger 到 `Validated`。B:86 与 A:375 仍要求“preview 前后 ledger/authorization/transition 计数不变”，而测试 `sap.rs:354-372` 只断言 `checks.len()==7`、三个 assumed、`authorized_actions==0`、`authorization_created==false`，没有 ledger/authorization/transition 计数断言，也没有可数的外部对象。B2 未完全闭合。

## 2. 已核实闭合

- **R7**：`scripts/check-schema-registry.py:37-50` 已实现同 domain 多 major 失败（当前 24 个 schema 均为 0.1，规则可执行）；SCHEMAS:60 明确规则 4 交给 `cargo run -p poie-sdkgen -- --check`，`.github/workflows/ci.yml:39` 已有该步骤，`:41` 有 registry checker。规则 3 的报错文案仍提“migration note”而文档要求直接失败，属文案小瑕疵，不阻断。
- **preview 代码**：`sap.rs:234-263` 将 `CommandOwnership`/`Reservations`/`Authorization` 统一标为 assumed，`authorized_actions=0`、`authorization_created=false`；测试新增两条 assumed 断言，26 个测试通过。
- **72/CI**：`evals/agent` 实为 72 个 `.toml`，六工具均有覆盖；`check-agent-evals.py` 的 schema 字符串、计数、未知工具、工具覆盖检查可执行，CI 步骤存在。
- **A/B 参数**：A:120 的 `state.query` 输入已改为 `{subject,asOf?}`，与 B:67、代码一致；B:67 输出补了 `capabilities[]`，与 A:120 对齐。
- **未改源码/设计/git**：仅覆盖本文件；工作树检查在写入前干净。

## 3. 已执行检查

```text
cargo test --locked -p poie-objects -p poie-plan       26 passed (3+19+1+3)
cargo fmt --all --check                                exit 0
cargo clippy -p poie-plan --all-targets -- -D warnings exit 0
cargo run --locked -q -p poie-sdkgen -- --check        exit 0
python3 scripts/check-schema-registry.py               OK (24 schemas; rule 3 checked)
python3 scripts/check-agent-evals.py                   OK (72 fixtures)
python3 scripts/check-docs.py                          OK
python3 scripts/check-test-matrix.py                   OK (381/397)
tomllib 检查 72 个 fixture 的顶层 approval                 0 个存在
```

## 4. 边界

本报告是设计门禁审查，不验证真实 provider、Tauri 运行时、SQLite 并发或桌面 Playwright。B0 的 `session_id`/`user_version`/删除启动 DROP 仍未实现，按 B §5 属实施第一步，不计设计阻断。R5 的 approval fixture 表位置/场景断言与 B2 的 §15 残余文字须在开工前修正；R7 本次通过。

VERDICT: CHANGES_REQUIRED
