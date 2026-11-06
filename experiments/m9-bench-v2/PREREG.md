# M9 台架 v2 预登记：物理时延、假动作、双端看门狗

- 实验 ID：`m9-bench-v2`
- 日期：2026-11-06
- 状态：**已运行；B6/B7/B5R 3/3 PASS**
- 性质：模拟台架；不是 PLC/真实传感器。

## 1. 问题

在冻结连续数值观测模型的前提下，验证：

- **B6**：执行器 ACK 但物理未生效（干转/卡阀）时，能否从 awaiting verification 转入 `outcome_unknown` 并锁死依赖流路；
- **B7**：水力建立窗口内（前 3 s 流量 0，第 4 s 越过阈值）不误判失败；
- **B5R**：断连关阀由 PLC 本地看门狗完成，重连必须先读脱扣/安全状态，不能自动恢复派发。

## 2. 判据

| 编号 | 场景 | 期望 |
| --- | --- | --- |
| B6 | ACK + flow 0（卡阀） | Dispatched；1 s/5 s 验证 false；gate 可 Acknowledged→OutcomeUnknown；流路锁死，后续动作不派发 |
| B7 | 正常动作，settling=3 s | +1 s 验证 false；+4 s 验证 true；gate Acknowledged，不误判 failed |
| B5R | 心跳停止 2 s | PLC 看门狗切阀、flow=0、fault 置位；宿主 Fallback；重连后未清除前禁止派发；清除后需重新授权 |

## 3. 非目标

- 连续数值滤波/卡尔曼/漂移拟合
- 真实 Modbus/OPC UA（下一阶段）
- 物理安全验收
