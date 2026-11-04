# Phase B 设计门禁复验（独立）

- 校验对象：[Phase B](phase-b-agent-governance.zh-CN.md) §6/§7/§11/§12/§13/§15、`evals/agent`、`scripts/check-agent-evals.py`、CI、`scripts/verify-all.sh`。
- 快照：`design/phase-b-closure`，HEAD **`5ed076e`**；写本文件前工作树干净。只读校验；未改源码/设计/git，只覆盖本文件。
- 方法：审阅 4adece7..5ed076e；用 `tomllib` 审计 72 个 fixture；在 `/tmp` 复制 checker+fixtures 做变异检验；运行 checker；核对 CI/verify-all 调用。
- 结论：**结构修复属实（approval 归属、per-tool args、旧 checker 旁路）；R5 仍是阻断项**——多数 fixture 仍无场景机制、期望错误码与工具/画像契约相反，§12 覆盖矩阵新表述也与 fixture 不符。不能开工。

## 判定矩阵

| 项 | 判定 | 一句理由 |
| --- | --- | --- |
| B2 只读 SAP preview | PASS | 5ed076e 未触碰 preview 文档/代码；沿用 4adece7 复核 |
| B3 审批状态机/恢复、B7 审计链/迁移 | PASS | 未触碰 |
| R5 eval 矩阵 | **REQUIRED** | 72 个标签齐全但语义空心；checker 不校验场景 |
| R7 schema checker | PASS | 未触碰 |

## 1. 已核实修复（不阻断）

- 72 = 六工具 × 12；approval 仅 3 个 `action.draft` fixture，均为顶层 `action = approve|reject|expire`，其余 69 个无 approval。旧“挪回 `[script]`”变异现 exit 1。
- 必填 args 已按工具落地（`state.query`/`plan.propose`/`plan.preview_sap`/`action.draft`/`answer.emit`；`catalog.describe` 无参数）；`action.draft--approval` 原有 `args={}` 已修复。
- checker 现拒绝 auto 工具 approval、缺失必填 args、目录缺失；`.github/workflows/ci.yml:43` 与 `scripts/verify-all.sh:57` 均调用。
- 5ed076e 只改 `evals/agent`、§12、checker、本文件；无其他 B 面回归。
## 2. 阻断：fixture 仍不可执行，且与契约相反

- 72 个 fixture 全部只有 1 个 tool response，prompt 全为 "Deterministic X fixture (case)" 模板；无注入文本、无第二次调用、无大响应、无坏 schema、无 provider 失败字段——除文件名外没有任何场景机制。
- 与 §6 表直接矛盾：
  - `catalog.describe` citation=无（:66），但其 citation/no-citation/injection 均期望 `citation_missing`；17 个 `citation_missing` 中只有 `answer.emit` 的 3 个有 §15 依据。
  - 6 个 budget 各仅 1 次调用，未超每轮上限 1/4/2/2/2/1（:66-71），不可能 `budget_exceeded`。
  - 6 个 unauthorized 全用 `dispatcher`（:186-190），而 dispatcher 拥有全部六工具，不可能 `unauthorized`。
  - 6 个 duplicate 仅 1 次调用且期望成功，未覆盖 §7:4 的幂等键；6 个 truncated 期望 `budget_exceeded`，而非 `provider_response_too_large`（:226）。
  - 5 个 unsupported 期望 `not_implemented`，但六工具均为 B1 已定义实现；5 个 bad-schema 参数/fixture schema 均合法，无 `unsupported_schema` 触发源；6 个 tool-error 用合法参数却期望 `invalid_tool_arguments`，等同 bad-args。
  - `catalog-describe--bad-args` 的 `args={}` 正是其合法调用（无参数），仍期望 `invalid_tool_arguments`。
- 因而“72 个确定性用例”仍是标签计数；仓库无 runner（`poie-governance` 未建，代码无 `poie.agent-eval` 引用），CI 只跑结构 checker，§15:251/259 的“确定性 eval 绿”仍名义化。
## 3. 阻断：checker 仍不校验场景语义

- 实测把 `state-query--budget` 改为 `not_implemented`、`catalog-describe--bad-args` 改为 `citation_missing`，仍输出 `agent evals OK (72 fixtures)`、exit 0。
- 实测 `id != stem`、`profile="root"`、`preregistered_at="not-a-date"` 仍 exit 0；`len(files) < 72` 即通过，不是“恰好 72”。
- `ERROR_CODES` 仅 8 个，缺 `provider_response_too_large`/`provider_timeout`/`approval_required`/`scope_denied` 等（§13:226），使 truncated/unauthorized 无法写对。
- bad-args/bad-schema/tool-error 三个 stem 豁免必填 args 校验；把 approval 塞进这类 action.draft 用例仍可计数。
## 4. §12 覆盖矩阵的新矛盾

- :213 “恰好 72，六工具各 12，仅 action.draft 含 approval”成立。
- :214 称“每个工具覆盖 [12 类]；action.draft 额外有 approve/reject/expire”，但 action.draft 实缺 no-citation/bad-schema/unsupported，只有 9 类 + 3 审批 = 12；按字面应为 15，与 72 自相矛盾——这是 5ed076e 新引入的表述矛盾。
- 0 个 fixture 使用 `expect.claims`（schema :209），`answer.emit` 无正例引用断言。
## 5. 已执行检查

```text
git diff 4adece7..5ed076e                     仅 evals/agent + §12 + checker + 本文件
tomllib 审计 72 fixtures                     6×12；approval 3 个且仅 action.draft
python3 scripts/check-agent-evals.py         OK (72 fixtures), exit 0
/tmp 变异：approval 移入 [script]            exit 1（旧旁路已闭）
/tmp 变异：删 state.query 必填 args          exit 1
/tmp 变异：错误码/id/profile 改写             exit 0  ← 语义仍可绕过
删除 evals/agent                             exit 1
```

边界：未跑全量 `cargo test`/Tauri/provider；`poie-governance` 尚未实现，其 runner 属实施期。

VERDICT: CHANGES_REQUIRED
