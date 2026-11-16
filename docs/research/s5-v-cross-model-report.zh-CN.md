# S5-V Independent Evidence Challenge — 报告

日期：2026-11-16
起点：`acdef3f`
Phase 1 合并：`7e14220`（PR #563）

## 状态

```text
Phase 1 Gate 1: PASS_FOR_EXTERNAL_MODEL_PHASE
Phase 2 Gate 2: PASS (energy subsystem, scoped)
S5 CROSS-MODEL VALIDITY — ESTABLISHED (SCOPED: PV/battery/load energy boundary)
Full S2 / model-wide cross-model validity — NOT ESTABLISHED
```

## Phase 1 — Independent Certificate Audit

- 12 个风险分层 claim 独立重跑 runner：9 `VERIFIED`，3 `VERIFIED_WITH_LIMITATIONS`
- 36 个证书哈希独立重算：0 failures
- 4 个负例全部被拒绝
- 无 `INCONSISTENT` / `UNSUPPORTED` / `UNDETERMINED`
- 限制：C66/C67/C73 只导出聚合随机统计；其余 24 个证书本轮仅哈希检查
- 独立性是**实现级**，不是第三方人工评审

详见 [audit/phase1/REPORT.zh-CN.md](../../audit/phase1/REPORT.zh-CN.md)。

## Phase 2 — Executable External Model Challenge

实现了一个独立 C99 FMI 3.0 Co-Simulation FMU：

- PV 2500 W
- 站点负载 3000 W
- 电池 200 kWh / 10 kW 充电限制
- 初始能量 40 kWh
- 初始维修消耗 0.5 kWh / 3 h
- 实际执行：解包 FMU、`dlopen`、实例化、初始化、变量交换、`fmi3DoStep`、终止

比较结果：

```text
depletion time       284400 s (79 h)
coarse max abs err   0 J
fine max abs err     0 J
tampered max err     35550000 J, detected=true
conservation         pass
capacity             pass
step sensitivity     pass
```

故意负例：PV 参数 +100 W，比较器检出不一致。

详见 [audit/phase2/REPORT.zh-CN.md](../../audit/phase2/REPORT.zh-CN.md)。

## 明确范围

已建立的是：

- 一个外部、可执行、独立实现的 FMI 3.0 CS FMU
- 一个能量子系统场景的逐时刻数值一致
- 可复现的配置、容差、日志、结果和哈希

未建立的是：

- 完整 S2 结论的跨模型验证
- 物理世界正确性
- 故障/维修/资源竞争子系统的独立模型验证
- 真实设备或安全认证

## 复现

```text
external/fmu-pv-battery/build.sh
external/fmu-pv-battery/build-tampered.sh
cargo run -q -p poie-viability --bin poie-s5v-energy-reference > audit/phase2/canonical-reference.txt
python3 audit/phase2/compare.py
```

在 macOS arm64 上已验证；源码和构建脚本支持 Linux，但当前提交的 FMU 二进制为 macOS arm64。Linux 复现需要重新执行 build 脚本，结果中的 FMU 哈希会不同。
