# S5-V N13 — Multi-Failure Causal Execution

基线：`6ee164b7b68ccfb80bfea797a626be49e85bea0c`。

## 状态

```text
N13-0 VALIDATOR GENERALIZATION
  VERIFIED

N13-A NATIVE MULTI-FAILURE EXECUTION
  ESTABLISHED (SCOPED: 10 CASES, UP TO 2 FAILURES)

N13-B MULTI-FAILURE CAUSAL STATE-TRANSITION
  ESTABLISHED (SCOPED: 10 CASES)

N13-C METAMORPHIC AND COUNTEREXAMPLE VALIDATION
  VERIFIED (SCOPED: 12 generated cases, seed 20261121)

N13-D ADVERSARIAL INTEGRITY
  ESTABLISHED (SCOPED)

N13-E EVIDENCE AND CERTIFICATE INTEGRITY
  ESTABLISHED (SCOPED)
```

## 本轮完成

- N12 OR2 特判移除；策略分类由实际选择资源和故障目标决定。
- 资源验证循环全部任务，不再只看 Task 1。
- stress 多实例在 CI 使用 `--require-multi-instance`，缺失 fmpy 时 FAIL。
- N12 报告过期“未建立”段落修正；原始证据保留。
- 新 N13 FMU 真正维护最多 2 个故障的队列与持久 `failed_mask`。
- 同刻故障成批应用，逐事件记录 target/failure_id/failed_before/after。
- 10 个多故障案例全部 ACCEPTED：
  - 无故障 / 单故障回归
  - 两个无关故障 A→B、B→A
  - 两个无关故障同刻
  - 无关→选中故障、选中→无关
  - 活跃上游→无关、未选路径→活跃
  - 选中+未选机器人同刻
- 终止后第二故障未被应用（N13-06/N13-07 canonical/executed 故障数一致）。

## 未建立

```text
metamorphic generated cases
real overflow for N13 multi-failure journal
1/900/1000 multi-failure step invariance
native both-models-invalid execution
N13 execution manifest/clean check-all/final certificate
repair-state complete projection
```
