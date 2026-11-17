# S5-V N8 — Event-Resolved Observatory & Cross-Model Trajectory Validation

基线：`dc9ebb39e6cf404ffd5dab880f99ec632700a306`。

## 状态（第一切片）

```text
N8 EVENT-RESOLVED NATIVE OBSERVATORY
  ESTABLISHED (SCOPED: 15 scenarios, native event identity/sequence/before-after)

N8 EVENT-LEVEL NONINTERFERENCE
  ESTABLISHED (TESTED SCENARIOS: N8-00..14)

N8 INDEPENDENT EVENT MAPPING
  ESTABLISHED (SCOPED: PV failure, repair completion, repair interruption)

N8 EVENT-RESOLVED CROSS-MODEL TRAJECTORY VALIDITY
  ESTABLISHED (SCOPED: N8-01, N8-02, N8-04; N4 FMU; 900/1000 s steps)

N8 CERTIFICATE V4 MIGRATION
  ESTABLISHED (41 claims)
```

## Event evidence

- [events.csv](event-observations/events.csv)
- [summary.csv](event-observations/summary.csv)
- same-timestamp N8-03: RepairCompleted sequence 2, AssetFailure sequence 3 at 10800.
- direct, time-observed, and event-observed outcomes are equal per scenario.
- N8-06 required tool failure: tool-t1 Failed, a-lathe Failed, repair interrupted.
- N8-14 deterministic repeat: identical event ids/order across repeated event runs.
- 8 event mutation controls reject swapped sequence, duplicate id, skipped event,
  uncommitted event, wrong before/after state, batch-after-as-event, time shift.

## 跨模型轨迹

- N8-01 / N8-02 / N8-04 与 N4 PV/repair FMU 事件轨迹比较均为 `MATCH`。
- N8-02 使用 1000 s 非通信网格步长，failure 4500 不在网格上。
- N8-04 同刻 `AssetFailure` + `RepairInterrupted`。
- 已知维修耗能差异登记为 `MODEL_ASSUMPTION_DIFFERENCE`，不当作数值错误。
- 6 个轨迹 mutation controls 全部检出（延迟、缺失、错误状态、维修完成延迟、终止延迟、事件顺序交换）。

## 限制

仅枚举场景；不建立一般轨迹级跨模型有效性。N5/N6 资源生命周期轨迹未纳入本轮比较。
