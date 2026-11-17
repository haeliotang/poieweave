# S5-V N8 — Event-Resolved Observatory & Cross-Model Trajectory Validation

基线：`dc9ebb39e6cf404ffd5dab880f99ec632700a306`。

## 状态（第一切片）

```text
N8 EVENT-RESOLVED NATIVE OBSERVATORY
  ESTABLISHED (SCOPED: native event identity, sequence, before/after)

N8 EVENT-LEVEL NONINTERFERENCE
  ESTABLISHED (TESTED SCENARIOS: N8-00..13)

N8 INDEPENDENT EVENT MAPPING
  NOT ESTABLISHED

N8 EVENT-RESOLVED CROSS-MODEL TRAJECTORY VALIDITY
  NOT ESTABLISHED

N8 CERTIFICATE V4 MIGRATION
  PENDING / separate gate
```

## Event evidence

- [events.csv](event-observations/events.csv)
- [summary.csv](event-observations/summary.csv)
- same-timestamp N8-03: RepairCompleted sequence 2, AssetFailure sequence 3 at 10800.
- direct, time-observed, and event-observed outcomes are equal per scenario.

## 未完成

N4/N5/N6 外部 FMU 的事件级轨迹映射与比较尚未建立。本切片不得宣称跨模型轨迹有效性。
