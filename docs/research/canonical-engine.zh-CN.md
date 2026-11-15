# Canonical Event Engine（S5-T 目标实现）

日期：2026-11-15
状态：首版内核；仅一个 S2 入口迁移完成，其余仍为 legacy

## 1. 目标

让所有正式 S2 结论通过同一套时间、库存、资源与状态提交规则产生。legacy
`survival.rs` 入口保留用于历史对照，但不再新增语义补丁。

## 2. 状态模型

`SimulationState` 由以下内容组成：

- 事件时钟：单调、夹紧 horizon
- 资产状态：Failed / Available
- 库存账本：按规划快照结算，校验后再提交
- 资源占用：required（AND）与 alternative groups（选一）
- 待处理事件：initial repair / failure / resupply / horizon

## 3. 执行器

`run_engine()` 的单一循环：

1. 读取下一事件并推进时钟
2. 结算区间能源与站点负载
3. 生成计划并用 scheduler 预留资源
4. 检查 horizon 与库存
5. 原子提交库存与资产状态
6. 记录 before/after 库存、计划 process、选中资源

终止状态是 `EngineTermination`：

- `CompletedHorizon`
- `Blocked(reason)`
- `IncompleteAtHorizon(reason)`

不再从日志字符串推导。

## 4. 迁移策略

按实验依赖分批迁移，不新增研究场景：

| 批次 | 入口 | 状态 |
| --- | --- | --- |
| 1 | 稀疏直接修复（Base-08） | 已完成，30 天一致 |
| 2 | 密集长修复 / 资源竞争 | 待迁移 |
| 3 | resupply / 负载 / 电池 | 待迁移 |
| 4 | S2 指标/证书 | 待迁移 |

每个迁移差异必须记录：

- 旧/新版本
- 第一条差异事件
- 事件前后库存、时间、资产状态
- 分类：语义修复 / 预期模型差异 / 未解释异常
- 受影响声明与证书

不得通过直接修改 expected 输出让 CI 变绿。

## 5. Dependency witness（首版）

每次运行中的维修记录 `WitnessEntry`：

- 选择了哪个能力提供者
- 该提供者的递归 `requires`/`requires_any` 依赖
- 依赖写入 `execution_dependencies`，`repair_completed` trace 保留 witness

当前 witness 由引擎从计划与调度结果派生，不是 S1 求解器直接输出；这是下一阶段的过渡实现。

## 6. 未完成

- 初始维修仍是独立实现，未进入事件枚举
- 事件期间新故障/取消未进入运行中 schedule
- 只有失败与初始维修路径被测试
- 正式 S2 入口尚未全部迁移
