# S5-V N21-R — Native Checkpoint Continuity & Authorized One-Shot Replanning

基线：`e052f010fe8d4023fc2004793dbab7941341b75c`。

```text
N21R-0 ADR / CERT IMPACT        VERIFIED
N21R-A NATIVE CHECKPOINT        VERIFIED (CANONICAL)
N21R-B NATIVE RESUME            VERIFIED (CANONICAL)
N21R-C ONE-SHOT REPLANNING      VERIFIED (CANONICAL, SCOPED)
N21R-D CROSS-MODEL              VERIFIED (SINGLE DECISIVE CASE, FMI)
N21R-E ADVERSARIAL              NOT ESTABLISHED
N21R-F V11 MIGRATION            VERIFIED (41 SUCCESSORS)
```

## 实现

- 新增 opt-in `run_engine_with_repair_group_to_checkpoint`：
  在 `RepairInterrupted` 处捕获原生 `RepairCheckpoint`，包含资产状态、
  库存、完成任务、未完成任务、选中资源、活跃依赖、已应用/剩余故障、
  组状态、恢复目标、事件序列、剩余 horizon 和 trace 历史。
- 新增 `resume_engine_with_authorization`：
  校验 checkpoint-bound one-shot authorization，保留失败资产状态、
  已完成任务、库存和时钟；仅对未完成任务重新调度并执行一次 continuation。
- `run_engine` 与 `run_engine_with_repair_group` 默认路径未改变。
- 原生事件新增 `CheckpointCreated`、`ReplanAuthorized`、`ReplanResumed`
  相关事件与 `RestorationCommitted`。

## 决定性测试

Rust 测试 `n21r_checkpoint`：

```text
Task A robot-m2 completes @7200
Task B robot-m1 running, robot-m1 fails @9000
checkpoint at 9000: completed=[A], unfinished=[B], robot-m1 Failed
authorization + proposal B->robot-m2
resume completes @19800
Task A not rerun
robot-m1 remains Failed
final inventory conserved
```

## 证书

- v10 41 证书字节冻结并由 historical verifier 校验。
- v11 41 个 `EXECUTION_SEMANTIC_EXTENSION_SUCCESSOR` 证书已签发，
  predecessor 指向 v10，v11 verifier 与 tamper controls 通过。
- v10 source environment 通过冻结基线校验。

## 独立 FMI 3.0 模型

新增 `external/fmu-n21r-replanning/poie-n21r-replanning.fmu`，由独立 C99
modelDescription/model.c 实现，输入 checkpoint authorization/proposal，
自己保存中断状态并执行一次 continuation。

C harness 与 fmpy 分别执行同一 FMU，结果一致：

```text
checkpointAt_s = 9000
completedA = 1
unfinishedB = 1
taskBRobot = 2
failedRobotM1 = 1
restored = 1
resumed = 1
unauthorized resume rejected
survived_s = 19800
```

证据：`audit/phase21r/comparisons/fmi-c-fmpy-canonical.json`。

## 独立 C99 外部状态机（非 FMI）

`external/fmu-n21r-replanning/external_replan` 是独立 C99 checkpoint/resume
状态机，不读取 canonical checkpoint。决定性案例：

```text
external: A completed, B robot-m1 interrupted @9000, checkpoint, valid auth,
B -> robot-m2, completion @19800, robot-m1 Failed, A not rerun,
inventory conserved
```

`audit/phase21r/comparisons/canonical-vs-external.json` 显示
`CROSS_MODEL_MATCH`。但该程序不是 FMI 3.0 模型，N21R-D 仍不成立。

## FMI executable mutants

三个测试专用 FMI mutant 均产生真实错误状态并被检测：

```text
M1 UNAUTHORIZED_RESUME      UNAUTHORIZED_CONTINUATION
M2 FAILED_ASSET_FORGOTTEN   FAILED_ASSET_REUSED
M3 COMPLETED_TASK_REPLAY    COMPLETED_TASK_ERASURE
```

## Reducer

`audit/phase21r/reducer.py` 对 M3 完成签名运行候选削减；当前决定性场景
`MINIMAL_BY_CONSTRUCTION`，保留 `COMPLETED_TASK_ERASURE` 签名并记录重放命令。

## 部分 adversarial 控制

Rust 测试已覆盖：

- forged authorization rejected
- checkpoint state tamper invalidates authorization
- failed robot alternative rejected
- horizon insufficient blocks continuation
- failure during continuation blocks second replan
- completed task replay rejected by scope check

## 未完成

- 独立 C99 FMI checkpoint/resume 外部模型与跨模型比较。
- 完整 17 项 adversarial 状态变异、三个 executable mutants、reducer。
- N21R external validation certificate / check-all clean run。
