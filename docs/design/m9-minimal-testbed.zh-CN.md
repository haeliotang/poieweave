# M9 最小物理实验台与 writable adapter 接口

状态：**规划稿**；硬件未采购、未接线、未运行。M9b/M9c 只完成了本机 BaSyx 容器上的只读影子/证据过期预演。

## 1. 目标

用最小、可控、可停机的物理回路验证：

1. 真实传感器观测 → 投影；
2. 非零 shadow plan；
3. 经 `poie-exec` 的 writable adapter 执行一个**有界、可幂等、可验证**的动作；
4. 独立传感器确认效果；
5. 断连、超时、重复命令和失控时 fail-closed。

## 2. 最小回路

```text
水箱 → 泵 → 电动阀 → 流量计 → 回水箱
              ↘ 压力传感器
电池 → 直流负载（可编程电子负载）
PLC/边缘控制器 ←→ 传感器/执行器
```

动作只允许：

- 泵开/关
- 阀开/关（默认关，fail-closed）
- 负载功率设定（限幅）

## 3. BOM（建议，非采购承诺）

| 类别 | 项 | 规格建议 | 数量 |
| --- | --- | --- | --- |
| 水回路 | 储水箱 | 20–40 L，带液位 | 1 |
| 水回路 | 小型离心泵 | 24 V DC，≤10 L/min | 1 |
| 水回路 | 常闭电动阀 | 24 V DC，断电关闭 | 1 |
| 传感 | 流量计 | 霍尔/涡轮，0–20 L/min，脉冲或 4–20 mA | 1 |
| 传感 | 压力传感器 | 0–1 MPa，4–20 mA/0–10 V | 1 |
| 传感 | 液位开关 | 高低位干接点 | 2 |
| 控制 | PLC / 边缘控制器 | 支持 Modbus TCP 或 OPC UA；24 V DI/DO + AI | 1 |
| 电源 | 24 V 电源 | ≥5 A，带保险/急停 | 1 |
| 安全 | 急停 | 切断泵/阀/负载 | 1 |
| 安全 | 漏电/过流保护 | 合适额定 | 1 |
| 电池 | 磷酸铁锂/铅酸 | 12–24 V，≤1 kWh | 1 |
| 负载 | 可编程电子负载 | 0–150 W，远程设定 | 1 |
| 管路 | 软管/接头/止回阀 | 与泵匹配 | 1 套 |
| 记录 | 独立数据记录 | 树莓派/工控机；只读采集 | 1 |

安全边界：

- 泵与阀默认断电关闭
- 急停硬切断，不依赖软件
- 负载限功率/限温
- 不允许无人值守长时间运行
- 水与电分离，接地/漏保

## 4. `poie-exec` 写适配器映射

实现 `poie_exec::WritableAdapter`：

```text
adapter_id = "plc-modbus-testbed"
supports_idempotency = true
dedup_window_ms >= command_validity + skew
supports_lease_epoch = true
verification_sources = ["flow-meter", "pressure-sensor", "plc-status"]
```

`dispatch(Command)`：

1. 校验 `valid_from_ms <= now <= valid_until_ms`；
2. 用 `idempotency_key` 查本地 dedup 记录；已存在 → `ExecError::Duplicate(original_action)`，不重复动作；
3. 写入 PLC（Modbus/OPC UA），带 lease epoch；低 epoch → 拒绝；
4. 返回 `Receipt { accepted, executed_at_ms, detail }`；
5. 不把“写入成功”当作“效果已达成”。

独立验证：

- 流量计 > 阈值 + 压力变化 → `VerificationSignal`
- 只有 PLC 回执、没有独立传感器 → 不满足 `verify_action`

## 5. 最小实验序列（M9.6）

| 步骤 | 动作 | 期望 |
| --- | --- | --- |
| T1 | 读传感器，建快照 | 投影状态与现场一致 |
| T2 | shadow plan | 动作非空，`dispatched=false` |
| T3 | 经 gate 派发“开阀 + 泵” | 回执 accepted；独立流量计确认 |
| T4 | 重复同一 idempotency key | `Duplicate`，现场不重复动作 |
| T5 | 过期 command | gate/适配器拒绝 |
| T6 | 拔网线/停 PLC | fail-closed；阀关闭；状态 Unknown |
| T7 | 急停 | 硬件切断；软件不得自动恢复 |

## 6. 与现有代码的关系

- 只读侧：`poie-runtime::ReadOnlyAdapter`（已有 BaSyx 实现）
- 写侧：`poie-exec::WritableAdapter` + `ExecutionGate`（已有契约，未接真实 PLC）
- 验证：`poie-exec::verify_action` 需要独立来源
- 适配器不得绕过 SAP/gate；shadow 路径没有 dispatch

## 7. 未完成

- 采购、接线、PLC 程序、Modbus/OPC UA 适配器实现
- 真实传感器校准与安全评审
- 现场/测试实例的 M9.2 兼容性 ADR
- 任何现场验收声明
