# M9 模拟台架闭环报告

- 预登记：[PREREG.md](PREREG.md)（先于运行提交）
- 日期：2026-11-06
- 性质：**模拟台架**；没有 PLC、没有真实传感器、没有物理动作。**不计入 M9.6 现场验收。**

## 结果

```text
id	check	result	detail
B1	gate 派发	PASS	Dispatched
B2	独立验证	PASS	flow-meter confirmed
B3	重复 key	PASS	Duplicate，物理执行不增加
B4	过期命令	PASS	Rejected，物理执行不增加
B5	断连 fallback + fail-closed	PASS	Fallback + valve closed
physical_executions=1 (after first 1, after expired 1)
```

5/5 PASS。

## 证明的链路

```text
authorized Action
  → ExecutionGate.dispatch
  → BenchAdapter (writable)
  → 模拟泵/阀动作
  → 独立 FlowSensor 确认 flow > 5 L/min
```

额外验证：

- 同一 idempotency key 第二次 → `ExecError::Duplicate`，物理执行次数仍为 1
- 过期 validity → gate `Rejected`，物理执行次数不变
- 执行器不可达 → gate `Fallback`；台架关闭阀门/流量归零（fail-closed）

## 局限

- 台架状态在内存中，不是 PLC/Modbus/OPC UA
- 流量值是模拟值，不是真实传感器
- 没有安全评审、急停、漏保、真实电池/负载
- 没有接入 BaSyx AAS 写路径
- 不能替代 M9.6 物理实验台

## 复现

```bash
cargo run -p poie-m9-bench-loop
```

期望输出：[`expected.txt`](expected.txt)。
