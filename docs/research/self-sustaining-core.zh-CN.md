# Self-Sustaining Core：S1–S4 研究综合（2026-11-14）

本页综合 2026-11 的 S1–S4 研究切片，回答一个问题：

> 给定能源、水、制造、维修与库存能力，外部补给中断后，究竟是什么决定了一个工业基地能生存多久，以及能否恢复生产能力？

**范围声明**：以下全部是合成模型上的模型级/软件级结果。它们不是物理可行性、设备寿命或工业安全性的证据；M9/M10 物理验收仍待外部条件。

## 1. 方法链

| 层 | 问题 | 确定性实现 | 主要证据 |
| --- | --- | --- | --- |
| S1 Capability Closure | 关键能力损坏后，本地能否在时限内恢复？最小阻塞是什么？ | `poie-viability::closure`，递归 AND/OR 搜索 + 资源/能力/占用/循环/时限 | [S1 REPORT](../../experiments/s1-capability-closure/REPORT.md) |
| S2 Viability Envelope | 哪些故障组合仍可恢复？能维持多久？再生物质是否改变边界？ | `survival`：故障包络、共享库存长周期、联合覆盖、再生、留出集 | [S2 envelope](../../experiments/s2-failure-envelope/REPORT.md)、[long horizon](../../experiments/s2-long-horizon/REPORT.md)、[horizon curve](../../experiments/s2-survival-horizon/REPORT.md) |
| S3 Recovery Loop | 系统能否在持续变化中提案、审批、执行、验证？ | 审批状态 + 租约 + 计划摘要 + 执行回执 | [S3 loop](../../experiments/s3-recovery-loop/REPORT.md)、[approval](../../experiments/s3-approval-lease/REPORT.md)、[digest](../../experiments/s3-approval-digest/REPORT.md)、[receipt](../../experiments/s3-receipt/REPORT.md) |
| S4 Independent Verification | 另一个实现能否复现/拒绝这些结论？ | Python 回执校验、计划回放、112 例差分语料、130 个变异的负向 fuzz | [S4 receipt](../../experiments/s4-independent-receipt/REPORT.md)、[differential corpus](../../experiments/s4-differential-corpus/REPORT.md)、[plan fuzz](../../experiments/s4-plan-fuzz/REPORT.md) |

## 2. 主要结果

### 2.1 生存边界由物质闭合决定

Base-05 在 10 天泵故障间隔下只能维持 **40 天**，失败原因是 `bronze-bar` 耗尽，而不是电力。电池即使被限制在 200 kWh / 10 kW，仍不是限制因素。生存曲线：

| 泵故障间隔 | 生存期 |
| --- | --- |
| 10 天 | 40 天 |
| 15 天 | 60 天 |
| 20 天 | 80 天 |
| 30 天 | ≥90 天 |

留出集 12 天 → 48 天，25 天 → 90 天；未参与调参，趋势一致。

### 2.2 独立覆盖 ≠ 联合生存

Base-06 的 56 个 0/1/2 故障组合中：

- 独立包络称 **25/56** 可恢复
- 共享库存联合执行只有 **24/56**
- 唯一差异是 `pump-p1,pump-p2`：第一台泵消耗唯一 sleeve 后，第二台无法再生产

因此 `Recovery Coverage` 必须被视为**乐观上界**，`Survival Horizon` 必须建立在联合状态上。

### 2.3 再生能改变边界，但不是无限的

- 本地 swarf→bar 回收把 10 天泵故障生存期从 **40 天提高到 60 天**
- 50% 回收收率时回落到 **50 天**
- 外部补给一件 sleeve 也能关闭 Base-06 的联合缺口

这说明“再生能力”是一个可度量的边界，而不是二元属性。

### 2.4 审批与执行证据

S3 把审批从布尔值推进到：

- `Required / Approved / Rejected` 三态
- 租约时长（计划超过租约不执行）
- 计划 SHA-256 摘要绑定（计划变化触发 mismatch）
- 每次成功恢复生成可自校验的执行回执
- 回执包含 plan digest、修复前后库存、验证结果
- 多服务共享库存恢复：8 h 租约可维持 30 天；1 h 租约在 3 h 泵维修前阻止
- 多服务多事件：day-10 泵 + day-20 车床，3 份回执
- 缺失 sleeve/bronze 时 day-10 阻塞并记录 `missing resource`
- 审批撤销：day-5 撤销后 day-10 修复被拒绝并记录 `approval_revoked`

### 2.5 独立验证发现了真实差异

S4 的 112 例差分语料一开始在 `bus-1` 上出现 3 个 Python/Rust 不一致。根因是 Python checker 没有传播 capability provider 的可用性（robot → charger → bus）。修正后 **112/112** verdict 一致；26 个可恢复计划全部独立回放通过；130 个结构化变异全部被拒绝。

这类“发现差异并修正独立实现”的结果，比单纯 PASS 更有研究价值。
后续又扩展到 504 个三 lot、648 个资产+lot、2592 个双资产+lot、900/4500 个
随机组合 fuzz 与 279 个能力子集案例，均一致；真正的第二次发现是 capability
verification 缺口（见 §2.5.1）。

### 2.5.1 S4 发现并修复 capability verification 缺口

后续的 capability-verification 变异发现了一个真实语义错误：Rust 的
`ensure_capability` 只检查 provider 资产是否可用，没有检查 provider 在该
capability 上的 verification 等级。Python 独立 checker 要求 provider 的
capability 证据达到 `modeled` 或更好。

初始结果：

```text
capability=sleeve-replacement target=pump-p1 rust=RESTORABLE python=BLOCKED
capability=lathe-repair       target=lathe-1  rust=RESTORABLE python=BLOCKED
45 cases: 43 matched, 2 mismatched
```

修复方式是新增 `BlockReason::UnverifiedCapability`，在求解 provider 前检查
`asset.capability_verification(capability) >= min_verification`，并增加回归测试。
修复后：

```text
single capability: 45/45
all 31 non-empty capability subsets: 279/279
5-seed combined fuzz: 4500/4500
```

这是 S4 的核心价值：独立实现不是重复跑一遍，而是能发现主实现没有执行的检查。

### 2.6 证据包与证书

S1–S4 的关键结论已经打包为机器可核查证书：

| 层 | 证书 | 核心内容 |
| --- | --- | --- |
| S1 | closure | Base-04 lathe verdict + plan |
| S2 | load | 0–2 kW 材料受限 / 3–4 kW 能量受限 |
| S2-joint | coverage | Base-06 独立 25 vs 联合 24 |
| S2-regen | regeneration | Base-05 40 天 → Base-08 60 天 |
| S2-metrics | metrics | 40/60 天、5/10 覆盖、+20 天再生 |
| S3 | recovery | 审批 + 事件 + 2 回执 + 库存链 |
| S3-multi | multi-recovery | 双服务/双事件/3 回执 |
| S4 | differential | 112/112 verdict、26/26 plan evidence |

九层证书通过 SHA-256 组成 bundle；独立验证器检查层集合、路径、schema 和哈希，
篡改负例返回 `tamper=detected`。一页式摘要见
[certificate bundle SUMMARY](../../experiments/certificate-bundle/SUMMARY.md)。

这些证书仍**未签名**，只证明内容完整性与模型身份，不证明作者、时效或物理真实性。


### 2.7 近期新增的可证伪结果

1. **电池容量语义修复**：初始电能超过声明容量时，长时仿真现在 fail-closed
   拒绝，而不是静默使用超额能量；等号边界（容量等于初始库存）仍被允许。
2. **负载切换点可解释**：Base-08 的材料/能量切换在 2.6 kW，平均 PV 充电为
   2.5 kW；把充电输出改为 10/20/40 kWh 后切换点移动到 1.3/2.6/5.1 kW，
   证明该边界由能量流入决定，而不是容量或充电功率上限。
3. **生产索引陈旧错误**：直接 `model.processes.push` 不会更新
   `producers`，导致新增修磨 process 对 S1 不可见。新增
   `Model::add_process` 后，刀片修磨把 2/3 片 insert 的生存期从
   40/50 天提升到 60 天；这是首个再生路径真正改变 S2 边界的结果。
4. **耗材前沿会移动**：开启修磨后，insert 数量不再主导；bronze 库存
   0/1.5/3/6 kg 对应 30/40/60/90+ 天。解决一个耗材瓶颈会暴露下一个。
5. **质量守恒残余物**：回收 1.2 kg swarf 到 bar+slag 时，0.9 kg 回收仍
   60 天，0.6 kg 降至 50 天并阻塞于缺少 bronze-bar；残余物只在低回收率
   时进入关键路径。

这些结果的共同限制仍是：单模型、单策略、模型级单位，不能外推为物理寿命。


### 2.8 冗余与执行权责的边界

1. **服务级 K+1 可以移除终点阻塞**：在 Base-08 中增加 `pump-p2` 并把
   `potable-water` 写成 `requires_any = [pump-p1, pump-p2]`，同一故障序列从
   60 天提升到完整 90 天。
2. **冗余必须覆盖整条服务链**：叠加 `filter-1` 故障后，两种模式都在 30 天
   停止（`no restorer for filter-1`）；共享组件需要自己的 restorer。
3. **维修层也是单点**：叠加 `robot-m1` 故障后，30/35 天都停止于
   `no restorer for robot-m1`。
4. **两层 restorer 缺一不可**：只有 filter+robot 两个 restorer 同时存在时
   才达到 90 天；任一单独存在仍停在 30 天。
5. **租约边界是闭区间**：3 h 计划（10800 s）在 lease=10800 s 时允许执行；
   10799 s 拒绝。
6. **撤销在执行中取消**：t=0 或计划内撤销（如 1 s/10799 s）取消且无
   receipt；撤销恰好等于计划终点（10800 s）仍执行并产生 receipt。

这些仍是模型级结论，不代表真实系统在共享电源、控制器、维修人员或物理
退化下的行为。


### 2.9 能量干预与 S1 选择语义

1. **能量制度由平均流入决定**：2.6 kW 负载下，容量 100–300 kWh 与充电
   功率 3–30 kW 的 9 种组合全部 16 天 `energy_depleted`；把 recharge 输出
   从 20 提升到 30/40 kWh 后，horizon 变为 16/60/60 天，阻塞回到材料/循环
   限制。
2. **冗余不能替代能量裕度**：冗余泵在 2.5 kW 下 60→90 天，在 2.6 kW 下仅
   16→17 天，3.0 kW 下无变化。
3. **S1 选择先优化时长，再优化相对库存消耗**：慢 restorer 排在前面时仍返回
   3h 计划；等时长的贵/便宜 restorer 会选相对消耗更低的方案（grease 剩余
   0.95 而不是 0.80）。
4. **撤销在执行中生效**：计划内撤销取消且无 receipt；恰好等于计划终点的
   撤销允许完成。

这些结果仍是模型级；尤其“容量/充电无效”只说明当前 recharge 过程结构，
不代表真实储能系统在更长时间尺度上无效。


### 2.10 并发、占用与截止时间

1. **共享机器人把并行退化为串行**：pump 修复 3 h、lathe 修复 2 h，两者都占用
   `robot-m1` 时区间重叠，makespan 为 5 h；改用 `robot-m2` 后无重叠，理想并行
   makespan 为 3 h。
2. **局部可行不等于全局截止时间可行**：两个修复分别 3 h/2 h，均满足 4 h
   deadline；串行 5 h 超时，共享机器人并行仍 5 h 超时，只有双机器人并行 3 h
   才满足。
3. **占用表是调度可行性的关键**：仅比较单计划时长不够，必须同时检查被占用
   资产是否重叠。
4. **修复成本影响生存**：在有限 grease 下，0.05 kg 修复器可支持 20–30 天，
   0.20 kg 修复器只能支持 0–10 天；S1 等时长成本选择因此具有 S2 后果。

这些仍是静态 makespan/区间观察，不是并发调度器，也没有抢占或 deadline
后悔机制。

## 3. 不应该从中得出的结论

- 不能推出真实设备的寿命、失效概率或安全性。
- 不能推出系统已具备工业自主能力。
- 不能把独立验证等同于形式化等价证明。
- 不能把内容寻址证书当作签名、第三方 attestation 或物理验收。
- 不能把合成库存/回收收率当作真实产率。
- 不能把 H2c 的小样本方向性结果当作普遍优势。

## 4. 仍然缺失的关键环节

1. 运行需求/水消耗与真实负载。
2. 随机故障、维修延迟、并发占用。
3. 质量守恒的回收收率、污染和残渣（已有无污染残余原型，仍缺污染/质量降级）。
4. 正式 Survival Horizon / Recovery Coverage / Regenerative Capacity 定义与留出集（已有原型，仍缺外部/独立数据）。
5. 独立过程模型、FMI co-simulation 或 SMT 验证。
6. 隔离 VM 的多进程/时钟/磁盘故障实验。
7. 证书签名/第三方 attestation 与 freshness/rollback 防护。
8. M9/M10 物理试验台与外部契约审查。

## 5. 当前可复现入口

```bash
cargo run --locked -q -p poie-viability --bin poie-closure
cargo run --locked -q -p poie-viability --bin poie-horizon
cargo run --locked -q -p poie-viability --bin poie-joint-coverage
cargo run --locked -q -p poie-viability --bin poie-recovery
cargo run --locked -q -p poie-viability --bin poie-closure-corpus-export
cargo run --locked -q -p poie-viability --bin poie-recharge-rule
cargo run --locked -q -p poie-viability --bin poie-insert-regeneration
cargo run --locked -q -p poie-viability --bin poie-joint-consumables
cargo run --locked -q -p poie-viability --bin poie-recycle-residue
python3 experiments/s4-differential-corpus/compare.py /tmp/corpus.json
```

所有实验的预登记、报告与冻结输出位于 `experiments/`；CI 重放检查由 `scripts/check-experiment-replays.py` 执行。
