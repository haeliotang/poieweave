# M9 Modbus TCP 虚拟从站与帧适配器预登记（PREREG）

- 实验 ID：`m9-modbus`
- 日期：2026-11-06
- 状态：**已运行；M1–M5 5/5 PASS**
- 性质：本机 TCP 虚拟 PLC；不是真实 PLC/硬件。

## 1. 问题

在冻结连续数值观测模型的前提下，验证：

1. FC16 原子写 `[cmd_epoch, cmd_seq, target_state, ttl_ms]` 的寄存器契约；
2. `cmd_seq` 不递增时 PLC 侧忽略，不重复动作；
3. Modbus 异常码 `0x06 Slave Device Busy` / `0x04 Slave Device Failure` 正确映射到 `poie-exec` 故障分类；
4. TCP 不可达/超时映射为 `Disconnected` → gate Fallback；
5. PLC 本地心跳看门狗在宿主停止心跳后切阀，重连前必须读安全状态并显式清除。

## 2. 寄存器表（固定）

| 地址 | 名称 | R/W | 说明 |
| --- | --- | --- | --- |
| 0 | cmd_epoch | W | 租约纪元 |
| 1 | cmd_seq | W | 单调序号 |
| 2 | target_state | W | 1=pump+valve open, 0=safe close |
| 3 | ttl_ms | W | 命令有效期 |
| 4 | heartbeat | W | 心跳计数 |
| 5 | fault | R | 看门狗脱扣 |
| 6 | flow_x10 | R | 流量 ×10 L/min |
| 7 | valve_open | R | 阀位 |
| 8 | pump_on | R | 泵 |
| 9 | inject_exception | W | 测试用：6=busy, 4=failure |
| 10 | clear_fault | W | 1=清除，要求阀关闭/流量 0 |

传输：Modbus TCP MBAP + FC03/FC16；unit id 1。

## 3. 判据

| 编号 | 检查 | 期望 |
| --- | --- | --- |
| M1 | 正常 FC16 + FC03 | gate Dispatched；flow_x10=80；独立读回 |
| M2 | 重复 cmd_seq | PLC 物理执行次数不增加；适配器重复 key → Duplicate |
| M3 | exception 0x06/0x04 | 适配器映射 Rejected；gate 不派发成功 |
| M4 | TCP 不可达 | Disconnected → gate Fallback |
| M5 | 心跳停止 2 s | PLC watchdog 切阀、fault=1；清除前禁止恢复；clear 后 flow=0 |

## 4. 非目标

- 连续数值滤波/标定
- 真实 PLC 程序、真实寄存器表
- OPC UA
- 物理安全验收
