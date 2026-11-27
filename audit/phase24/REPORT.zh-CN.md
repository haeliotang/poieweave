# S5-V N24 — Compositional Multi-Task Recovery & Causal Trace Continuity

基线：`3048e3af`。

```text
N24-0 HISTORY/BOUNDARY        IN PROGRESS — pre-start false pass corrected in N24 FMU
N24-A CAUSAL TRAJECTORY       PARTIAL — single-task initial+resume event chain
N24-B MULTI-TASK              PARTIAL — real canonical checkpoint with B,C unfinished
N24-C SCHEDULING              PARTIAL — shared-tool + independent parallel oracle
N24-D AUTHORIZED SCHEDULE     NOT ESTABLISHED
N24-E NATIVE EXECUTION        PARTIAL — canonical B/C continuation CompletedHorizon
N24-F CROSS-MODEL             NOT ESTABLISHED
N24-G ADVERSARIAL             NOT ESTABLISHED
```

## Phase 0 已复现并纠正 pre-start failure false pass

N23 FMU 对 robot-m3 @9500（计划开始 10000）的故障忽略并完成 @20800。
N24 FMU 正确应用该故障：

```text
completed=0 restored=0 preStartInvalidated=1
executedRobot=3 (no automatic switch)
termination at 9500
```

证据：`failing-witness/PRESTART_FAILURE_FALSE_PASS/`。
N23 expected legacy 单任务回归仍需后续确认：N24 扩展必须保持 v17 默认路径不变。

## Canonical two-unfinished-task continuation

真实 checkpoint：

```text
A = COMPLETED
B = INTERRUPTED
C = NOT_STARTED
robot-m1 = FAILED
```

外部授权 B→robot-m3, C→robot-m2；canonical resume 原生完成：

```text
termination = CompletedHorizon
survived_s  = 19800
```

测试：`n24_canonical_two_unfinished_tasks_can_resume`。
证据：`canonical/two-task.json`。共享工具冲突、串行/并行独立 oracle、FMI 与 v18 尚未完成。

## 独立双任务调度 oracle（initial）

共享 tool-t1 的有限域：

```text
generated = 4
reference = 4
soundness = true
completeness = true
serial = 4
parallel = 0 (shared tool correctly forbids overlap)
optimal makespan = 23400
```

证据：`candidate-generation/two-task-shared-tool.json`。
独立资源并行 control、完整固定矩阵、shared resource ownership 尚未完成。

## Causal trajectory and independent-resource scheduling

- `causal-trajectories/n23-style.json`：真实 canonical initial 6 events + resume 6 events，
  checkpoint bridge 前后连续；validation 无问题（单任务 scoped）。
- `candidate-generation/two-task-shared-tool.json` 与 `two-task-independent.json`：
  shared tool 时 4 个串行候选、0 并行；独立资源时允许并行，soundness/completeness true。
