# S2 Evidence Audit（Phase A）

日期：2026-11-16
基线：`main` at `a73c459`（PR #507）
状态：初始盘点完成；声明归一化与迁移未完成

## 1. 方法

由 [`scripts/build-s2-inventory.py`](../../scripts/build-s2-inventory.py) 扫描：

- `experiments/s2-*` 实验目录
- `experiments/certificate-s2-*` 证书
- claims ledger
- REPORT/PREREG/expected 与 runner 源码中的模型路径、随机种子

输出：

- [s2-evidence-inventory.json](s2-evidence-inventory.json)
- [s2-evidence-inventory.md](s2-evidence-inventory.md)

`--check` 模式在 CI 中防止清单与仓库漂移。

## 2. 盘点结果

| 指标 | 数量 |
| --- | --- |
| S2 claims | 30 |
| S2 experiment 目录 | 89 |
| 被 claim 引用的 experiment | 30 |
| 未被 claim 引用的 experiment | 59 |
| 未分类 experiment | **0** |
| FORMAL_CLAIMED / FORMAL_AUXILIARY | 30 / 48 |
| SUPERSEDED_BY_CANONICAL / EXPLORATORY_NEGATIVE | 6 / 2 |
| DEFINITIONAL / DETERMINISTIC_SWEEP | 1 / 2 |
| 已迁移到 canonical 的正式 S2 experiment | **9** |
| canonical 语义修正 | **8**（C11–C14、C18、C19、C71、C73） |
| 缺少记录种子的随机 experiment | **0** |
| S2 证书 | 11（6 历史 + 5 canonical 重签） |
| 状态页 S2 数值结论行 | 12 |
| 其中未显式登记为 claim | **0** |

## 3. 关键发现

### F1：25/84 为 claim 主证据，59 个未映射实验已全部分类

分类结果（审计标签，不是证书）：

- `FORMAL_CLAIMED` 25
- `FORMAL_AUXILIARY` 48：支持已有 claim 的敏感性/对照
- `FORMAL_AUXILIARY_MISSING_SEED` 2
- `SUPERSEDED_BY_CANONICAL` 6：旧时间语义值被 canonical 取代
- `EXPLORATORY_NEGATIVE` 2
- `DEFINITIONAL` 1
- `UNCLASSIFIED` **0**

分类不等于迁移；48 个辅助实验仍可能包含需要单独 canonical 重算的正式数值。

### F2：正式 S2 迁移数为 0

Batch 1–3 是 kernel/migration probe，不是正式 S2 实验的 canonical 重算。当前只有：

- 稀疏 canonical/legacy 对照
- 竞争、密集、补货、负载、电池的迁移探针
- 3D 内核级账本单元测试

没有任何一个 `s2-*` 正式实验已经由 `engine.rs` 产生正式结果。因此 S5-T 不能声称收敛。

### F3：2 个再生阈值实验是确定性扫描，不是随机实验

- `s2-regen-threshold-battery`
- `s2-regen-threshold-initial`

它们的 runner 对 multiplier/capacity 做确定性枚举，没有 RNG。此前因报告正文出现
“seed”一词被误判为 stochastic；已修正扫描规则并分类为 `DETERMINISTIC_SWEEP`。
`stochastic_experiments_missing_seed` 从 2 降为 **0**。这不是补造种子，而是纠正
错误分类。

### F4：证书状态已按影响矩阵和 canonical 重签区分

- `certificate-s2-canonical-joint/load/regeneration/order/metrics-v2`：REISSUED，绑定 engine/model/runner/output SHA
- `certificate-s2-time`：SUPERSEDED
- `certificate-s2-metrics`：REISSUED 为 scoped v2；coverage 定义 `service-impact-v1`，10 组纳入，2 partial + 1 superseded 明确排除
- `certificate-s2-order`：REISSUED（canonical order 适配器）

历史证书原样保留；重签是新目录，不覆盖旧文件。

### F5：状态页 12 条 S2 数值结论已全部归一化

状态页 S2 小节原有 13 条 bullet，其中 1 条是 S1 工具循环，已排除。其余 12 条
全部映射到 claim：

- 11 条归并到 C5–C19 / C63
- 1 条 Monte Carlo 新建为 C68，但仍是 legacy LCG 结果，尚未 canonical 重算
- `unmapped_status_results` 从 13 降为 0

这不等于 Monte Carlo 已获 canonical 证书；C68 明确标注 legacy-only。

### F8：C7 的 claim 证据映射错误已修正

原 claims ledger 将 C7 指向 `s2-regeneration`（Base-06/07 覆盖），但 40→60
数值来自 `s2-insert-regeneration`。已修正并新增 `s2-canonical-c7`。

### F7：C5 的 claim 证据映射错误已修正

原 claims ledger 将 C5 指向 `s2-failure-envelope`，但 40/60/80/90 数值来自
`s2-survival-horizon`。已修正为后者，并新增 `s2-canonical-c5` 作为 canonical
迁移证据。`s2-failure-envelope` 重新成为未映射实验。

### F6：Batch 3D 状态需要区分内核测试与迁移对照

3D 的 9.5 kWh 守恒是 **engine 单元测试**，不是独立的正式 S2 迁移 artifact。迁移日志已修正为：

```text
3d | 联合能源账本 | 内核账本测试通过 | 独立迁移对照未单列
```

## 4. Phase A 状态

- 机器可读清单：完成 v0.2（含 expected 输出、状态页数值、证书记录）
- 全部 claim 与 experiment 的正式/探索分类：未完成
- 60 个未映射实验的处置：未完成
- C5 细节态对比：summary/energy/failure-time 一致，trace 事件数为记录粒度差异
- 13 条状态页数值结论的 claim 归一化：未完成
- 证书影响矩阵：未开始

因此 Phase A 只能标记为 **INITIAL INVENTORY COMPLETE**，不能标记为完成。

## 5. 下一步

Phase B 的第一项应是 C5（Base-05 故障包络），因为它有明确 claim、模型和证书依赖，且能暴露 canonical coverage 枚举与 legacy 的语义差异。

如果 C5 在现有 engine 上无法表达 `NOT_PORTABLE_WITH_CURRENT_ENGINE`，必须记录原因，不能伪装成通过。
