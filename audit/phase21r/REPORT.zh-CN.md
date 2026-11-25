# S5-V N21-R — Native Checkpoint Continuity & Authorized One-Shot Replanning

基线：`e052f010fe8d4023fc2004793dbab7941341b75c`。

```text
N21R-0 ADR / CERT IMPACT        VERIFIED
N21R-A NATIVE CHECKPOINT        VERIFIED (CANONICAL)
N21R-B NATIVE RESUME            VERIFIED (CANONICAL)
N21R-C ONE-SHOT REPLANNING      VERIFIED (CANONICAL, SCOPED)
N21R-D CROSS-MODEL              VERIFIED (FMI 3.0, SINGLE DECISIVE CASE)
N21R-E ADVERSARIAL              VERIFIED (SCOPED: 15 PASS, 3 NOT REPRESENTABLE)
N21R-F V11 MIGRATION            VERIFIED (41 SUCCESSORS, CLEAN CHECK-ALL)
```

## 实现

- opt-in `run_engine_with_repair_group_to_checkpoint` 在 `RepairInterrupted`
  处捕获原生 `RepairCheckpoint`。
- `resume_engine_with_authorization` 校验 checkpoint-bound one-shot 授权，
  消费 authorization_id，保留失败资产、已完成任务、库存和时钟，仅重排
  未完成任务。
- 默认 `run_engine` / `run_engine_with_repair_group` 未改变。
- 原生事件：`CheckpointCreated`、`ReplanAuthorized`、`RepairResumed`、
  `TaskRestarted`、`RestorationCommitted`。

## 决定性案例

```text
Task A robot-m2 completes @7200
Task B robot-m1 interrupted @9000
checkpoint: A completed, B unfinished, robot-m1 Failed
authorization + proposal B -> robot-m2
resume completes @19800
A not rerun; robot-m1 remains Failed; inventory conserved
```

canonical Rust 测试 9/9 通过。

## 独立 FMI 3.0 跨模型

`external/fmu-n21r-replanning/` 独立 C99 FMI 3.0 模型，自己创建
checkpoint、验证授权、执行 continuation，不读取 canonical checkpoint。

C harness 与 fmpy 分别执行同一 FMU，并与 canonical 对齐：

```text
checkpointAt_s = 9000
completedA = 1
taskBRobot = 2
failedRobotM1 = 1
restored = 1
resumed = 1
unauthorized resume rejected
survived_s = 19800
```

## 对抗

- 6 canonical checkpoint/resume 负例；
- 3 个 FMI executable mutants：
  M1 unauthorized resume、M2 failed asset forgotten、M3 completed task replay；
- 自动 reducer 保留 `COMPLETED_TASK_ERASURE` 签名；
- 18 项矩阵：15 PASS，3 个 bounded 域不可表示。

## 证书

- v10 41 证书字节冻结，历史/tamper verifier 通过。
- v11 41 个 `EXECUTION_SEMANTIC_EXTENSION_SUCCESSOR` 证书通过。
- `N21R_EXTERNAL_VALIDATION_CERTIFICATE.json` 绑定 FMI、mutants、reducer、
  矩阵与 v11 index；tamper control 通过。
- detached clean worktree `check-all` 通过。

## 科学边界

```text
GENERAL REPLANNING VALIDITY — NOT ESTABLISHED
MULTI-ATTEMPT REPLANNING — NOT ESTABLISHED
AUTONOMOUS REPLANNING AUTHORIZATION — NOT ESTABLISHED
LEVEL 3 PLANNING INDEPENDENCE — NOT ESTABLISHED
AUTONOMOUS RECOVERY — NOT ESTABLISHED
FULL S2 CROSS-MODEL VALIDITY — NOT ESTABLISHED
PHYSICAL-WORLD CORRECTNESS — NOT ESTABLISHED
INDUSTRIAL SAFETY CERTIFICATION — NOT ESTABLISHED
```

## 结论

```text
S5-V N21-R NATIVE CHECKPOINT CONTINUITY
AND AUTHORIZED ONE-SHOT REPLANNING
  ESTABLISHED (SCOPED)
```

范围仅限 single interruption / single authorization / one continuation，
以及测试过的 bounded 2-task、2-robot、FMI 3.0 域。
