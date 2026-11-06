# M9 Modbus TCP 虚拟从站预演报告

- 预登记：[PREREG.md](PREREG.md)（先于运行提交）
- 日期：2026-11-06
- 性质：本机 TCP 虚拟 PLC；不是真实 PLC/硬件。

## 结果

```text
id	result	detail
M1	PASS	outcome Dispatched=true; flow_x10=80
M2	PASS	duplicate=Err(Duplicate("a1")); exec 1->1; stale_seq_write=Ok(()); exec_after_stale=1
M3	PASS	busy_reason="slave device busy (0x06)"; failure_reason="slave device failure (0x04)"
M4	PASS	fallback=Ok(false)
M5	PASS	fault=1; valve=0; flow=0; cleared=true; fault_after=0
failures=0
```

5/5 PASS。

## 已验证

- FC16 原子写 `[epoch, seq, target, ttl]`；FC03 读回 `flow_x10=80`
- 适配器重复 idempotency key → `Duplicate`；PLC 物理执行 1→1
- 直接写相同 `cmd_seq` → PLC 忽略，执行数仍 1
- `0x06 busy`、`0x04 failure` 映射为 `Rejected`，reason 分别包含 busy/failure
- TCP 不可达 → `Disconnected` → gate `Fallback`
- 心跳停止 2 s → PLC 看门狗 fault=1、valve=0、flow=0；clear 后 fault=0

## 局限

- 虚拟从站、内存寄存器；不是真实 PLC
- 未做真实 TCP 半开/延迟分布/重传
- 未接桌面端；未接真实传感器
- 不替代 M9.6 物理实验台

## 复现

```bash
cargo run -p poie-m9-modbus
```

期望输出：[`expected.txt`](expected.txt)。
