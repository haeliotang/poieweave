# S5-V Independent Evidence Challenge — 报告

日期：2026-11-16
起点：`acdef3f`
Phase 1 合并：`7e14220`（PR #563）

## 状态

```text
Phase 1 Gate 1: PASS_FOR_EXTERNAL_MODEL_PHASE
Phase 2 Gate 2: PASS_SCOPED_EXECUTION_AND_ENDPOINTS

S5-V EXTERNAL EXECUTION — ESTABLISHED (SCOPED)
S5-V ENDPOINT AGREEMENT — ESTABLISHED (SINGLE ENERGY CASE)
Native Observatory — ESTABLISHED (SCOPED, energy case)
Observation Noninterference — ESTABLISHED (five required cases)
Certificate v2 — ESTABLISHED (41 claim certificates; v1 frozen and historically verified)
S5-V TRAJECTORY-LEVEL CROSS-MODEL VALIDITY — NOT ESTABLISHED
S5 CROSS-MODEL VALIDITY — NOT ESTABLISHED (full S2)
```

此前把 `max_abs_error = 0 J` 解释为 canonical 引擎逐时刻误差是不正确的。
该 0 J 是 FMU 与独立解析参考的误差；canonical 真正作为运行时记录导出的只有维修完成记录和
耗尽时刻；t=0 是配置初值，终点电量零是约束，二者不冒充 canonical 运行时导出。

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
depletion time              284400 s (79 h)
analytic-reference max err  0 J
canonical-boundary max err  0 J (t=0, repair completion, depletion)
C harness vs fmpy max err   0 J
tampered max err            28440000 J, detected=true
conservation                pass
capacity                    pass
step sensitivity            pass
```

故意负例：PV 参数 +100 W，比较器检出不一致。

详见 [audit/phase2/REPORT.zh-CN.md](../../audit/phase2/REPORT.zh-CN.md)。

## Native Observatory and Observation Noninterference

Native observer 已从事件内核直接产生只读快照，不再通过零数量补货插入采样边界。

实现：

- `run_engine_observed` 在 `engine.rs` 内部按请求时刻切分 ledger，不插入 resupply 事件
- 保持调用者原始 horizon
- 支持 `Before` / `After` 事件前后观测
- 原生导出 `RunningRepair`（label、end、plans、resources）
- 终止、存活时间、最终库存与无观测运行一致
- 不可观测请求显式标记，不回填最终状态

证据：

- [audit/phase3/native-trajectory.csv](../../audit/phase3/native-trajectory.csv)
- [audit/phase3/README.zh-CN.md](../../audit/phase3/README.zh-CN.md)

0/1/2/3/79 h 能量：

```text
t=0      40.000 kWh
t=1 h    39.500 kWh
t=2 h    39.000 kWh
t=3 h    38.000 kWh
t=79 h    0.000 kWh
```

3 h 维修完成事件 Before/After：

```text
Before  38.500 kWh  repair active
After   38.000 kWh  repair committed 0.5 kWh
```

终止保持：

```text
blocked:energy_depleted, survived_s=284400
```

N2 非干扰测试覆盖提前终止、能源恰好耗尽于 horizon、同刻多事件、维修中断、
维修完成恰好等于 horizon，以及事件前/后观测；比较 termination、survived_s、
final_inventory、完整 trace 与累计能量。

旧的 neutral-probe `poie-s5v-canonical-observatory` 保留为受限交叉检查，
不再作为 N1 证据。

## Trajectory discrepancy

与 FMU 的差异：

```text
t=1 h   -0.1667 kWh
t=2 h   -0.3333 kWh
t=3 h    0
t=79 h   0
```

这些差异精确等于预登记的维修结算假设：

```text
FMU - canonical = -repair_energy * t / repair_duration   (0 < t < repair_duration)
```

最大假设残差为 0 J。另增加：

- 1000 s 非整除步长：最大解析参考误差 5.96e-7 J
- 故意篡改维修持续时间 +1 h：事件边界差异 450 kJ，成功检出
- 分类：`EVENT_BOUNDARY_MISMATCH`

该结果支持“差异可分解为模型假设差异”，但不支持
trajectory-level cross-model validity 已建立。

## Certificate v2

- 36 个既有 claim + C6/C69/C70/C71/C72 共 41 个 v2 证书
- v2 绑定完整运行时环境：engine、survival、clock、closure、scheduler、
  Cargo.toml、Cargo.lock、runner、models、expected、REPORT、PREREG
- v2 记录 v1 predecessor 与 SHA
- v1 87 个证书文件保持冻结：`audit/certificates/v1-frozen.json`
- v1 在 baseline `9f343b55` 上独立重验
- v2 tamper 测试覆盖 expected/engine/environment/claim/predecessor 五类变异

## 明确范围

已建立的是：

- 一个外部、可执行、独立实现的 FMI 3.0 CS FMU
- 一个能量子系统场景的 external execution 和 endpoint agreement
- Native Observatory 与五类非干扰反例
- v2 证书完整环境绑定，v1 冻结并在历史 baseline 独立验证
- FMI 3.0 标准平台目录与 modelIdentifier 命名
- 独立 fmpy 0.3.32 导入器实际加载并执行
- 可复现的配置、容差、日志、结果和哈希

未建立的是：

- 完整 S2 结论的跨模型验证
- trajectory-level cross-model validity
- 物理世界正确性
- 故障/维修/资源竞争子系统的独立模型验证
- 真实设备或安全认证

维修期存在已登记的模型假设差异：FMU 将 0.5 kWh 维修耗能均匀分摊到
前 3 小时；canonical 在维修完成时一次提交。canonical 不导出维修期内部
状态，因此不比较该区间轨迹，也不通过提高容差消除差异。

## 复现

```text
external/fmu-pv-battery/build.sh
external/fmu-pv-battery/build-tampered.sh
cargo run -q -p poie-viability --bin poie-s5v-energy-reference > audit/phase2/canonical-reference.txt
python3 audit/phase2/compare.py
```

在 macOS arm64 上已实际执行并通过 fmpy 0.3.32 独立导入器。CI 的 `S5-V external FMU Linux conformance` job 在 Ubuntu 上重新构建 FMU、
运行 fmpy importer，并把当次 fmpy CSV/log 显式传给比较器；比较器缺失该输入
会直接失败。该 job 将 Linux 当次 FMU、harness CSV、fmpy CSV/log、canonical
参考和 result 保存为 `s5v-linux-evidence` artifact，并用 SHA256SUMS 绑定。
当前提交的 FMU 二进制是 macOS arm64，Linux 上重新构建后哈希会不同。
