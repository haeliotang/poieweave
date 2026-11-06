# M9 模拟台架闭环预登记（PREREG）

- 实验 ID：`m9-bench-loop`
- 日期：2026-11-06
- 状态：**已登记，尚未运行**
- 性质：**模拟台架（bench simulator）**，不是物理硬件；不替代 M9.6 现场闭环。

## 1. 问题

在没有 PLC/真实传感器时，能否先闭合“**非零影子 → 有界命令 → 独立传感器验证**”的软件链路，并证明：

1. 只有经 `ExecutionGate` 的 authorized action 才能到达 writable adapter；
2. 命令有有效期；过期拒绝；
3. 同一 idempotency key 不重复动作；
4. 独立流量传感器（不是执行器回执）确认效果；
5. 断连时 fallback 并关闭阀门（fail-closed），不自动重试。

## 2. 台架模型

- 模拟 PLC/执行器：`BenchAdapter` 执行 `pump_on` + `open_valve`
- 独立流量计：`FlowSensor` 读取共享台架状态（来源与执行器不同）
- 动作窗口 10 s；lease epoch 1；dedup key 固定
- 初始阀门关闭、泵关闭、流量 0
- 正常命令后流量 8.0 L/min；传感器阈值 >5 L/min

## 3. 判据

| 编号 | 检查 | 期望 |
| --- | --- | --- |
| B1 | gate 派发 | `Dispatched`，动作状态 acknowledged |
| B2 | 独立验证 | 来源 `flow-meter`，`confirmed=true` |
| B3 | 重复 key | adapter 返回 `Duplicate`，物理执行次数不增加 |
| B4 | 过期命令 | gate 返回 `Rejected`，物理执行次数不增加 |
| B5 | 断连 | gate 返回 `Fallback`；台架 `safe_close` 后阀门关闭、流量 0 |

## 4. 非目标

- 不是 BaSyx AAS 写路径
- 不是 PLC 程序
- 不是真实传感器校准
- 不是 M9.6 物理实验台验收
