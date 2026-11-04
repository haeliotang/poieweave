# Phase B 设计门禁复验（独立）

- 校验对象：[Phase B](phase-b-agent-governance.zh-CN.md)、[Phase A](phase-a-ontology-product.zh-CN.md)、`crates/poie-plan`、`evals/agent`、`scripts/`、CI。
- 快照：`design/phase-b-closure`，HEAD **`4adece7`**；写本文件前工作树干净。只读校验；未改源码/设计/git，只覆盖本文件。
- 方法：审阅 3301181..4adece7 diff；用 `tomllib` 审计 72 个 fixture；在 `/tmp` 复制 checker+fixtures 做变异检验；运行 `python3 scripts/check-agent-evals.py`、`cargo test --locked -q -p poie-plan`。
- 结论：**B2 已闭合；R5 仍未闭合**——fixture 仍是 `args={}` 占位，18 个 approval 中 15 个挂在 auto 工具上，且 checker 仍可通过上次的 approval 旁路。不能开工。

## 判定矩阵

| 项 | 判定 | 一句理由 |
| --- | --- | --- |
| B2 只读 SAP preview | PASS | §6.1/§15/A 已统一 throwaway 语义；测试断言 0/false 与三个 assumed |
| B3 审批状态机/恢复 | PASS | 4adece7 未触碰；沿用 3301181 已核实结论 |
| B7 审计链/迁移 | PASS | 同上，未触碰 |
| R5 eval 矩阵 | **REQUIRED** | 72 例仍为占位；approval 归属错误；checker 无 per-tool 审批覆盖 |
| R7 schema checker | PASS | 未触碰 |

## 1. 阻断项 R5：fixture 仍是占位，approval 覆盖错位

- “72 个 fixture 重新生成”不成立：`git diff 3301181..4adece7 -- evals/agent` 只改了 66/72 个文件；6 个 `--normal` 完全未动，48 个各只加一行 `error`，18 个只是把 `approval` 行从 `[script]` 后移到顶层。全部 72 个 `args` 仍为 `{}`。
- 18 个 approval fixture = 6 工具 × 3 动作，但 §6:66-71 只有 `action.draft` 是 `operator_confirm`，其余五工具为 auto；§11:190 也只允许 dispatcher 的 draft 走审批。15/18 个 approve/reject/expiry 挂在无审批的工具上，`reject/expiry` 期望的 `approval_rejected/expired` 与工具契约相反。
- 唯一合法的 `action-draft--approval` 仍用 `args={}` 且期望成功；§6:70 要求 `{planFingerprint,actionId}`，按严格参数校验应得 `invalid_tool_arguments`。`state.query/plan.propose/plan.preview_sap/answer.emit` 的 normal 同样缺必填参数却期望成功（5/6 个 normal 用例不可执行）。
- 错误码是跨工具复制、不编码场景：`catalog.describe` 的 citation/injection 期望 `citation_missing`，但 §6:66 该工具 citation 为“无”；五个 auto 工具的 `unauthorized` 期望 `approval_required`，与 §13:235 的 `unauthorized/scope_denied` 不符；`duplicate`（脚本只有一次调用）也期望 `budget_exceeded`。
- 因此 §12:213/224、§15:260 的“72 个确定性用例 + 覆盖矩阵”仍只是文件名计数；仓库无 runner 执行这些 TOML，CI 只跑结构 checker，“确定性 eval 绿”是名义上的。

## 2. checker 仍可通过上次的旁路

- 已修：目录缺失时 `return 1`（旧的硬编码绿已删除）；`scripts/verify-all.sh:57` 与 `.github/workflows/ci.yml:43` 均调用；计数、六工具覆盖、审批动作集合检查可执行。
- 未修：审批覆盖是**全局动作集合**，不是 per-tool。把 `action-draft--{approval,reject,expiry}.toml` 的 `approval` 行重新放回 `[script]`（即上次的畸形写法），checker 仍输出 `agent evals OK (72 fixtures)`、exit 0——另外五个工具仍提供 `approve/reject/expire`。唯一需要审批的工具三个 fixture 全部畸形也不会被发现。
- 另缺：不校验 `data["id"] == path.stem`、`profile` 合法值、`preregistered_at`、`script.responses` 形状；`approval` 多余键被接受（实测加 `tool="catalog.describe"` 仍绿）。

## 3. B2 已闭合（复核）

- B:83/86/255 与 A:285/375 已统一为“函数内 throwaway ledger；不写 live/external ledger、不创建 Authorization、不产生 Authorized transition；`authorized_actions=0`/`authorization_created=false`”；全库无“前后计数不变”残留。
- `crates/poie-plan/src/sap.rs:204-213` 只新建局部 `ActionLedger` 并 apply `Validated`；`Authorization {` 仅出现在 `validate_plan_through_sap` 路径（:155），preview 不调用。
- 测试 `preview_is_read_only_and_marks_authorization_assumed` 断言 7 个 check、command_ownership/reservations/authorization 三个 assumed、`authorized_actions==0`、`authorization_created==false`；`cargo test -p poie-plan` 4 个测试通过。
- 保留意见（非阻断）：两个返回值是常量，测试不能证明“没有外部写”；A:288 的“没有任何 authorization/Authorized 记录”仍无对应断言，只能靠结构审查。

## 4. 已执行检查

```text
git diff 3301181..4adece7 (71 files)              66 fixture touched; 6 normal unchanged
python3 scripts/check-agent-evals.py              OK (72 fixtures)
tomllib 审计 72 fixtures                           approve=6 reject=6 expire=6; 15 on auto tools
/tmp 变异：action-draft approval 嵌套回 [script]   OK (72 fixtures), exit 0  ← 旁路复现
python3 check-agent-evals.py（目录缺失/空）         exit 1, exit 1
cargo test --locked -q -p poie-plan               4 passed
```

边界：未跑全量 `cargo test`/Tauri/provider；`poie-governance` 尚未实现，其结构测试属实施期。

VERDICT: CHANGES_REQUIRED
