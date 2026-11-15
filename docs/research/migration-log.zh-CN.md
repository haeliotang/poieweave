# Canonical Migration Log

日期：2026-11-15
状态：批次 2 部分解释、密集修复阻塞

## 批次表

| 批次 | 场景 | 状态 | 结论 |
| --- | --- | --- | --- |
| 1 | 稀疏直接修复（Base-08） | 接受 | 30 天一致 |
| 2a | 单机器人竞争 | 接受 | canonical 5 h vs legacy 3 h，资源占用语义修复 |
| 2b | 双机器人竞争 | 接受 | 3 h 一致 |
| 2c | 密集长修复 | **阻塞** | canonical 出现未解释 `Cycle { turn-sleeve-p1 }` |
| 3 | resupply/负载/电池 | 未开始 | — |
| 4 | S2 指标/证书 | 未开始 | — |

## 差异记录规则

每条差异必须记录：

- 旧/新入口与版本
- 第一条差异事件
- 事件前后库存、时间、资产状态
- 分类：语义修复 / 预期模型差异 / 未解释异常
- 受影响声明与证书

禁止直接修改 expected 输出让 CI 变绿。

## 批次 2 记录

详见 [s5-migration-batch2](../../experiments/s5-migration-batch2/REPORT.md)。

- 竞争场景的差异来自共享机器人序列化，属于语义修复。
- 密集长修复的 canonical 路径在 20.9 天因 turn-sleeve 循环终止，legacy 为
  15.6 天，差异未解释，迁移阻塞。
