# S5-V N8-H — Temporal Alignment & Trajectory Evidence Hardening

基线：`c038ba82bc09dc7d15c1734b7a9bcbf9ed65d284`。

## 状态

```text
S5-V N8-H TEMPORAL ALIGNMENT
AND TRAJECTORY EVIDENCE HARDENING
  ESTABLISHED (SCOPED)

N8 REVALIDATED EVENT-RESOLVED
CROSS-MODEL TRAJECTORY VALIDITY
  ESTABLISHED (SCOPED: N8-01, N8-02, N8-04)

N8 DISCRETE EVENT AGREEMENT
  ESTABLISHED (SCOPED)
```

## False PASS 修复

旧 N8 比较器三处缺陷已复现并修复：

- P0-A：N8-02 最近邻把 14000 s 当作 14400 s，产生 1.2 MJ 差异却仍 `MATCH`。
- P0-B：N8-04 终止后 `UNOBSERVABLE_AFTER_TERMINATION` 的占位 0 J 进入统计，
  产生约 139.2 MJ 错误差值。
- P0-C：能量误差无条件归为 `MODEL_ASSUMPTION_DIFFERENCE`。

旧 N8 证据未改写；修复结果在 `audit/phase8h/`。

## 严格对齐

- exact sample only；无最近邻。
- N8-02 在 1000 s 名义步长下实际切分到 4500、7200、10800、14400。
- N8-04 终止后样本全部排除并保留原因。
- step size 从运行 metadata 读取。

## 能量与守恒

- 按登记维修耗能假设扣除解释项。
- canonical 与 external 分别验证能量守恒。
- 未解释残差 ≤ 1 J 才允许 MATCH。
- 第一处 raw difference / 未解释差异 / 无效对齐 / 离散事件分歧分别报告。

## Revalidation

```text
N8-01 MATCH_WITH_EXPLAINED_ASSUMPTION
N8-02 MATCH_WITH_EXPLAINED_ASSUMPTION
N8-04 MATCH_WITH_EXPLAINED_ASSUMPTION (validation interval 0..3600, terminal 7200 excluded)
```

负例覆盖时间错位、终止后伪值、错误 step 元数据、完成后额外误差、故障后发电、
同刻事件顺序反转；全部拒绝。

## 限制

- N8-04 连续能量只比较终止前 0..3600；7200 终止边界只做离散事件比较。
- N8-02 4500 事件侧能量使用 exact 4500 采样，不做插值。
- 不建立 N5/N6 资源轨迹、一般轨迹有效性或物理正确性。
