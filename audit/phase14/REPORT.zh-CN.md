# S5-V N14 — Compositional Multi-Failure Validation

基线：`5704c100fe4ebef36b0121da337125a60cae5cdb`。

```text
N14-0 VALIDATOR INTEGRITY
  VERIFIED

N14-A COMPOSITIONAL EXECUTION
  ESTABLISHED (SCOPED: EXTERNAL 2 TASKS / 4 FAILURES)

N14-B CROSS-MODEL COMPOSITIONAL CAUSAL VALIDITY
  ESTABLISHED (SCOPED: 24 GENERATED ONE-TASK CASES)

N14-C GENERATED CROSS-MODEL VALIDATION
  VERIFIED (SCOPED)

N14-D ADVERSARIAL INTEGRITY
  VERIFIED (SCOPED)

N14-E EVIDENCE AND CERTIFICATE
  ESTABLISHED (SCOPED)
```

## 本轮完成

- N13 严格复验发现真实缺陷：完成/恢复事件的 `failed_before` 使用旧 mask。
- N14 FMU 修复该历史连续性，支持 4 故障、2 任务、双机器人。
- 越界 failureCount、非法 kind/asset 返回 FMI 错误，不再静默截断。
- 2 任务外部案例：0/2/2 同刻/选中+工具/4 故障，全部通过。
- 同一 JSON 契约驱动 canonical adapter 与 N14 FMU，24/24 生成案例 ACCEPTED。
- 负例：共同错误时间、事件丢失、状态历史丢失、failure_id 交换、终止后事件、Task 2 非法资源、超容量输入全部拒绝。

## 限制

```text
two-task canonical execution = NOT_REPRESENTABLE
canonical repair-state before/after = NOT_OBSERVED
generated domain = 1 task / 1 robot / 0-4 failures
```
