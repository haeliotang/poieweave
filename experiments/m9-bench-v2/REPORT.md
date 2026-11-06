# M9 台架 v2 报告

- 预登记：[PREREG.md](PREREG.md)（先于运行提交）
- 日期：2026-11-06
- 性质：**模拟台架**；不是 PLC/真实传感器。冻结连续数值观测模型。

## 结果

```text
id	result	detail
B6	PASS	dispatched=true; verify1=false; verify5=false; outcome_unknown=true; flow_locked=true
B7	PASS	early=false; settled=true; state=Some(Acknowledged)
B5R	PASS	tripped=true; valve_closed=true; blocked_before_clear=true; cleared=true
failures=0
```

3/3 PASS。

## 解释

- **B6**：适配器 ACK，但模拟卡阀使流量始终为 0；1 s/5 s 独立验证均为 false；gate 从 Acknowledged 合法转入 `OutcomeUnknown`，流路锁死。
- **B7**：3 s 建立窗口内（1 s）验证 false；4 s 验证 true；动作保持 Acknowledged，不误判 failed。
- **B5R**：宿主心跳停止 2 s 后，PLC 本地看门狗切阀、流量归零、Fault 置位；重连前禁止派发；`safe_clear` 只在阀关闭/流量 0 时清除。

## 局限

- 模拟 PLC/传感器，无真实时延、噪声、机械卡滞
- 看门狗在内存中实现，不是 PLC 梯形图/ST
- 未实现 Modbus TCP/寄存器契约
- 不替代 M9.6 物理实验台

## 复现

```bash
cargo run -p poie-m9-bench-v2
```

期望输出：[`expected.txt`](expected.txt)。
