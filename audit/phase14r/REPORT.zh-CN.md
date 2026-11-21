# S5-V N14-R — Canonical Multi-Task Group & v7

基线：`161273d4599ff07fdd90f7a4be9e3bd6e221a94f`。

```text
N14R-0 REPRESENTABILITY DECISION
  VERIFIED

N14R-A CANONICAL GROUP EXECUTION
  ESTABLISHED (SCOPED)

N14R-B ORIGINAL N14-B RECOVERY
  ESTABLISHED (SCOPED: 10 CASES)

N14R-C GENERATED CROSS-MODEL EXTENSION
  VERIFIED (12 CASES)

N14R-D CERTIFICATE GOVERNANCE
  ESTABLISHED (41 v7 SUCCESSORS)

N14R-E REPRODUCIBILITY
  ESTABLISHED (SCOPED)
```

## Canonical 实现

新接口复用原有 event loop、scheduler、closure、inventory ledger 和 atomic commit：

```rust
run_engine_with_repair_group(...)
run_engine_with_repair_group_event_observed(...)
```

任务被转换为每个任务一个 `Plan`，由原 scheduler 选择资源并计算 start/end；全部任务完成后沿原 `apply_plans` 路径原子恢复目标。旧入口保持不变。

## 跨模型结果

10 个固定案例全部 ACCEPTED：

```text
R00 并行双机器人 10800
R01 同机器人串行 18000
R02 两个无关故障
R03 同刻双故障
R04 选中机器人故障 -> 中断
R05 必需工具故障 -> 中断
R06 活跃上游 bus-1 -> 中断
R07 4 故障
R08 7201 组提交前故障 -> 中断
R09 10800 完成与故障同刻
```

canonical 与 N14 FMU 的 task start/end、selected resources、makespan、failure target/time、interruption/completion 逐项一致。

## v7 证书

- v6 证书字节冻结，按 baseline `161273d` 的 engine 哈希和文件哈希验证。
- 41 个 v7 successor，分类 `EXECUTION_SEMANTIC_EXTENSION_SUCCESSOR`，predecessor 指向 v6。
- v7 verifier 检查 current engine/runner/expected/model/source env/native N14R manifest/replay；tamper controls 通过。
- 旧 254 个 legacy replay artifacts 在修改后仍通过，证明 legacy 非干扰。
