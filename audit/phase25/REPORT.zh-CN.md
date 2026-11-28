# S5-V N25 — Runtime-Enforced Joint Recovery Contracts

基线 `0633a728`。

```text
Gate 0  HISTORY/FALSE-PASS       PARTIAL — premature restoration reproduced
Gate 1  STATE TRANSITIONS        PARTIAL — independent oracle passes after fix
Gate 2  JOINT SCHEDULE ENFORCEMENT VERIFIED — pre-execution rejection
Gate 3  NATIVE + FMI             NOT ESTABLISHED
Gate 4  ADVERSARIAL              NOT ESTABLISHED
Gate 5  CERTIFICATE/CI           NOT ESTABLISHED
```

## P0/P1 已复现并修复

N24 trajectory 中：

```text
TaskCompleted C @12600  a-lathe = available   <-- 错误
TaskCompleted B @19800  a-lathe = available
RestorationCommitted    @19800
```

原因：`TaskCompleted` 事件使用 group 完成后的最终资产快照。

修复后：

```text
TaskCompleted C @12600  a-lathe = failed
TaskCompleted B @19800  a-lathe = failed
RepairCompleted @19800  a-lathe = available
RestorationCommitted @19800 a-lathe = available
```

independent oracle `state_oracle.py` 现在返回 `valid: true`，且
initial 事件补齐 `CheckpointCreated` 后序列连续。

这是 N25 的起点，不是最终完成；Gate 2–5 仍未建立。

## P2 canonical 强制完整联合计划

`task_schedule` 写入 trusted choice-set certificate payload：

```text
authorized B 9000-19800, C 19800-23400 -> CompletedHorizon
unauthorized parallel B/C same robot -> Blocked full schedule mismatch
rejection clock = 9000, 未推进执行
```

证据：`canonical/full-schedule-enforced.json`。
v21 冻结；v22 41 successors 签发并通过 verifier/tamper。
