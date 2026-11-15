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
| S2 claims | 15 |
| S2 experiment 目录 | 75 |
| 被 claim 引用的 experiment | 15 |
| 未被 claim 引用的 experiment | 60 |
| 已迁移到 canonical 的正式 S2 experiment | **0** |
| 缺少记录种子的随机 experiment | 2 |
| S2 证书 | 6 |
| 状态页 S2 数值结论行 | 13 |
| 其中未显式登记为 claim | 13 |

## 3. 关键发现

### F1：正式 claim 只覆盖 12/72 个 S2 实验

这不能自动解释为其余 60 个都是探索性实验。Phase B 必须先分类：

- 正式结论（必须迁移）
- 辅助/诊断实验（保留但不进入正式证书）
- 已被取代的旧实验

在分类完成前，不得把 60 个未映射实验视为“不需要迁移”。

### F2：正式 S2 迁移数为 0

Batch 1–3 是 kernel/migration probe，不是正式 S2 实验的 canonical 重算。当前只有：

- 稀疏 canonical/legacy 对照
- 竞争、密集、补货、负载、电池的迁移探针
- 3D 内核级账本单元测试

没有任何一个 `s2-*` 正式实验已经由 `engine.rs` 产生正式结果。因此 S5-T 不能声称收敛。

### F3：2 个随机实验缺少记录种子

- `s2-regen-threshold-battery`
- `s2-regen-threshold-initial`

它们被标记为 stochastic，但 REPR/报告和 runner 中未找到可审计种子。Phase B 前必须补种子或降级为确定性实验。

### F4：6 个 S2 证书中只有 time 证书被 claim 显式引用

`certificate-s2-joint/load/metrics/order/regeneration` 没有被 claims ledger 显式引用。Phase D 需要 Certificate Impact Matrix，不能假设它们自动有效。

### F5：状态页有 13 条 S2 数值结论未归一化为 claim

盘点把状态页 S2 小节的 13 条数值结论逐条纳入 `status_results`。它们目前没有
claim ID，也没有显式证书绑定。Phase A 的下一步必须决定每一条：

- 归并为已有 claim
- 新建 claim
- 降级为诊断性观察

不能把“已写入状态页”等同于“已被正式 claim 和证书覆盖”。

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
