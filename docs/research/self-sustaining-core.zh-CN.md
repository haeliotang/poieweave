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

### 2.5 独立验证发现了真实差异

S4 的 112 例差分语料一开始在 `bus-1` 上出现 3 个 Python/Rust 不一致。根因是 Python checker 没有传播 capability provider 的可用性（robot → charger → bus）。修正后 **112/112** verdict 一致；26 个可恢复计划全部独立回放通过；130 个结构化变异全部被拒绝。

这类“发现差异并修正独立实现”的结果，比单纯 PASS 更有研究价值。

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
| S3 | recovery | 审批 + 事件 + 2 回执 + 库存链 |
| S4 | differential | 112/112 verdict、26/26 plan evidence |

证书通过 SHA-256 组成 bundle；独立验证器检查层集合、路径、schema 和哈希，
篡改负例返回 `tamper=detected`。一页式摘要见
[certificate bundle SUMMARY](../../experiments/certificate-bundle/SUMMARY.md)。

这些证书仍**未签名**，只证明内容完整性与模型身份，不证明作者、时效或物理真实性。

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
3. 质量守恒的回收收率、污染和残渣。
4. 正式 Survival Horizon / Recovery Coverage / Regenerative Capacity 定义与留出集。
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
python3 experiments/s4-differential-corpus/compare.py /tmp/corpus.json
```

所有实验的预登记、报告与冻结输出位于 `experiments/`；CI 重放检查由 `scripts/check-experiment-replays.py` 执行。
