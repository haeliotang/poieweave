# S5-T Canonical Evidence Convergence — Migration Report

日期：2026-11-16
基线：`main` at `4c98b82`（PR #561）；起点 `a73c459`（PR #507）
状态：**S5-T SEMANTIC CONVERGENCE — ESTABLISHED (SCOPED)**
状态：**S5 CROSS-MODEL VALIDITY — NOT ESTABLISHED（完整 S2；能量子系统 scoped challenge 见 S5-V 报告）**

本报告记录当前可核查的迁移状态。`ESTABLISHED (SCOPED)` 仅覆盖第 8 节列出的
模型/仿真级范围；不是物理安全、真实设备或跨模型有效性声明。

## 1. 迁移的全部正式 S2 声明

S2 evidence inventory 共记录 53 条 S2 claims、112 个 S2 实验、12 条已归一化的状态页数值结论。40 个 claim-bearing canonical 实验完成迁移：22 个精确一致、18 个语义修正。

### 1.1 精确一致（9 个 canonical 实验）

| claim | 迁移实验 | 结果 |
| --- | --- | --- |
| C5 | [s2-canonical-c5](../../experiments/s2-canonical-c5/REPORT.md) | 40/60/80/90 天，能源与失败时刻一致 |
| C6 | [s2-canonical-c6](../../experiments/s2-canonical-c6/REPORT.md) | 独立 25/56、联合 24/56 |
| C7 | [s2-canonical-c7](../../experiments/s2-canonical-c7/REPORT.md) | 6/6 行 40/50→60 |
| C8 | [s2-canonical-c8](../../experiments/s2-canonical-c8/REPORT.md) | 11/11 负载行，2.6 kW 切换 |
| C9 | [s2-canonical-c9](../../experiments/s2-canonical-c9/REPORT.md) | 6/6 行，冗余 60→90 |
| C10 | [s2-canonical-c10](../../experiments/s2-canonical-c10/REPORT.md) | 4/4 补货间隔 |
| C69 | [s2-canonical-load-base05](../../experiments/s2-canonical-load-base05/REPORT.md) | Base-05 负载 5/5 |
| C70 | [s2-canonical-regen-horizon](../../experiments/s2-canonical-regen-horizon/REPORT.md) | Base-05/08 再生 8/8 |
| C72 | [s2-canonical-order](../../experiments/s2-canonical-order/REPORT.md) | 24/25、40/44，order_sensitive=0 |

### 1.2 语义修正（8 个 canonical 实验，9 条 claim）

| claim | 迁移实验 | 修正 |
| --- | --- | --- |
| C11/C62 | [s2-canonical-c11](../../experiments/s2-canonical-c11/REPORT.md) | t=0 事件延迟到初始维修后；最小集 filter-1；0→0.125 d |
| C12/C63 | [s2-canonical-c12](../../experiments/s2-canonical-c12/REPORT.md) | grease 边界被维修期故障语义取代 |
| C13/C64 | [s2-canonical-c13](../../experiments/s2-canonical-c13/REPORT.md) | 能量边界日级不变；精确耗尽最多移 1 h |
| C14/C65 | [s2-canonical-c14](../../experiments/s2-canonical-c14/REPORT.md) | 冗余 90 一致；baseline 31→2 天 |
| C18/C66 | [s2-canonical-c18](../../experiments/s2-canonical-c18/REPORT.md) | 均值 9.9→5.4，p90 15.7→6.0 |
| C19/C67 | [s2-canonical-c19](../../experiments/s2-canonical-c19/REPORT.md) | 均值 14.0→2.2，p90 15.6→2.0 |
| C68/C73 | [s2-canonical-c68-monte-carlo](../../experiments/s2-canonical-c68-monte-carlo/REPORT.md) | 中位 72→68，均值 64.8→64.0 |
| C71 | [s2-canonical-metrics-groups](../../experiments/s2-canonical-metrics-groups/REPORT.md) | coverage 5/10 资产可修复 → 9/10 服务影响 |
| C75 | [s2-canonical-energy-interventions](../../experiments/s2-canonical-energy-interventions/REPORT.md) | energy interventions 12 行日级一致；16→16.25 d |
| C76 | [s2-canonical-redundancy-restorers](../../experiments/s2-canonical-redundancy-restorers/REPORT.md) | filter/robot restorer 7/8 一致；共享机器人 baseline 30→20 |
| C74 | [s2-canonical-c12-components](../../experiments/s2-canonical-c12-components/REPORT.md) | C12 horizon/insert/repair-sweep 全部覆盖 |
| C75 | [s2-canonical-energy-interventions](../../experiments/s2-canonical-energy-interventions/REPORT.md) | energy interventions 12 行日级一致 |
| C76 | [s2-canonical-redundancy-restorers](../../experiments/s2-canonical-redundancy-restorers/REPORT.md) | filter/robot restorer 7/8 一致 |
| C77 | [s2-canonical-aux-batch1](../../experiments/s2-canonical-aux-batch1/REPORT.md) | 辅助抽样 20/20 一致 |
| C78 | [s2-canonical-aux-batch2](../../experiments/s2-canonical-aux-batch2/REPORT.md) | 辅助抽样 19/19 一致 |
| C79 | [s2-canonical-aux-batch3](../../experiments/s2-canonical-aux-batch3/REPORT.md) | 辅助抽样 14/14 一致 |
| C80 | [s2-canonical-aux-batch4](../../experiments/s2-canonical-aux-batch4/REPORT.md) | 辅助抽样 13/13 一致 |
| C81 | [s2-canonical-aux-batch5](../../experiments/s2-canonical-aux-batch5/REPORT.md) | coverage 定义修正 + threshold 一致 |
| C82 | [s2-canonical-aux-batch6](../../experiments/s2-canonical-aux-batch6/REPORT.md) | 辅助抽样 9/9 一致 |
| C83 | [s2-canonical-aux-batch7](../../experiments/s2-canonical-aux-batch7/REPORT.md) | 辅助抽样 17/17 一致 |
| C84 | [s2-canonical-aux-batch8](../../experiments/s2-canonical-aux-batch8/REPORT.md) | regen-long 语义修正 |
| C85 | [s2-canonical-aux-batch9](../../experiments/s2-canonical-aux-batch9/REPORT.md) | 辅助抽样 10/10 一致 |
| C86 | [s2-canonical-aux-batch10](../../experiments/s2-canonical-aux-batch10/REPORT.md) | 辅助抽样 18/18 一致 |
| C87 | [s2-canonical-aux-batch11](../../experiments/s2-canonical-aux-batch11/REPORT.md) | 辅助抽样 13/13 一致 |
| C88 | [s2-canonical-aux-batch12](../../experiments/s2-canonical-aux-batch12/REPORT.md) | 辅助抽样 8/8 一致 |
| C89 | [s2-canonical-aux-batch13](../../experiments/s2-canonical-aux-batch13/REPORT.md) | adversarial 日级一致 + executor 修正 |
| C90 | [s2-canonical-aux-batch14](../../experiments/s2-canonical-aux-batch14/REPORT.md) | adversarial-time + regen horizon/yield |
| C91 | [s2-canonical-aux-batch15](../../experiments/s2-canonical-aux-batch15/REPORT.md) | load + thresholds 11/11 |
| C92 | [s2-canonical-aux-batch16](../../experiments/s2-canonical-aux-batch16/REPORT.md) | grease + yield 10/10 |
| C93 | [s2-canonical-aux-batch17](../../experiments/s2-canonical-aux-batch17/REPORT.md) | two-restorers 4/4 |
| C94 | [s2-canonical-aux-batch18](../../experiments/s2-canonical-aux-batch18/REPORT.md) | adversarial-timing 30/32 + 2 executor |
| C95 | [s2-canonical-aux-batch19](../../experiments/s2-canonical-aux-batch19/REPORT.md) | regeneration/resupply Base-07 joint |
| C96 | [s2-canonical-aux-batch20](../../experiments/s2-canonical-aux-batch20/REPORT.md) | recycle + random distribution |

## 2. 保持一致的历史结果

以下历史数值在 canonical engine 下保持不变：

- C5 40/60/80/90 天
- C6 独立 25/56、联合 24/56
- C7 2/3 inserts 40/50→60，4 already 60
- C8 2.6 kW 制度切换，11/11 行
- C9 冗余泵 60→90，2.6 kW 17 天
- C10 10/20/30/60 → 60/50/40/30
- C69 Base-05 0–4 kW 五行
- C70 Base-05/08 再生八行
- C72 order 24/25、40/44，order_sensitive=0

这些结果可用于 canonical 证书；对应历史证书原样保留。

## 3. 被语义修正的历史结果

| 历史结果 | canonical 结果 | 原因 |
| --- | --- | --- |
| C11 day-10 最小 0 天 | 10 天 | legacy runner 忽略 `at_day` |
| C11 day-0 精确 0 天 | 0.125 天 | 初始维修 3 h 后才处理 t=0 事件 |
| C12 grease 10/16/21 天 | 2 天 | 5 天维修期间第 2 天故障按事件时刻中断 |
| C13 精确耗尽 3.292/1.097 | 3.333/1.111 | canonical 调度完整维修链 |
| C14 baseline 31 天 | 2 天 | 维修期故障按事件时刻生效 |
| C18 随机均值 9.9 | 5.4 | 同上 |
| C19 随机均值 14.0 | 2.2 | 同上 |
| C68 中位 72 / 均值 64.8 | 68 / 64.0 | 同上 |
| coverage 5/10 | 9/10 | 定义从资产可修复改为服务影响 |

所有变化均有 runner、expected 输出和报告；没有为保持旧数值而回退语义。

## 4. 被证据撤销或取代的声明

- C12 grease 边界的原数值被 C63 取代；horizon/insert/repair-sweep 由 C74 覆盖。
- `certificate-s2-time` 状态为 `SUPERSEDED`。
- `certificate-s2-metrics` 原聚合值被 scoped v2 取代；历史证书保留。
- C68 的 legacy-only 标记被 C73 取代。

## 5. 仍然无法解释的差异

在已迁移的 40 个 canonical 实验中：**无 `UNEXPLAINED`**。

已知但已解释的差异：

- C11 day-0 的 0.125 天是语义修正，不是误差。
- C13 的 1 h 位移是调度链差异。
- C71 coverage 差异是定义变化。
- legacy 的若干延迟故障行为已被明确取代。

59 个辅助实验不在此声明内：57 个已有 canonical batch 证据，2 个定义型/负结果记录排除，0 个未覆盖；全部排除在 S5-T scoped 收敛声明之外。

## 6. 未能迁移的执行入口及原因

| 范围 | 状态 | 原因 |
| --- | --- | --- |
| 59 个 `FORMAL_AUXILIARY` 实验 | 57 个已有 canonical 证据，2 个定义型/负结果排除，0 个未覆盖 | 已建立 batch1–20 |



| `certificate-s2-metrics` excluded group | 明确排除 | `redundancy_time` superseded baseline，不进入 v2 声明范围 |
| FMI 协同仿真 | 未执行 | 仅有 modelDescription/容器检查 |
| 真实设备/物理安全 | 未验证 | 超出本 GOAL 范围 |

## 7. 新旧证书对应关系

| 历史证书 | 新证书 | 状态 | 分类 |
| --- | --- | --- | --- |
| `certificate-s2-joint` | `certificate-s2-canonical-joint` | `REISSUED` | `EXPECTED_AGREEMENT` |
| `certificate-s2-load` | `certificate-s2-canonical-load` | `REISSUED` | `EXPECTED_AGREEMENT` |
| `certificate-s2-regeneration` | `certificate-s2-canonical-regeneration` | `REISSUED` | `EXPECTED_AGREEMENT` |
| `certificate-s2-order` | `certificate-s2-canonical-order` | `REISSUED` | `EXPECTED_AGREEMENT` |
| `certificate-s2-metrics` | `certificate-s2-canonical-metrics` | `REISSUED` | `MIXED_AGREEMENT_AND_DEFINITION_CHANGE` |
| `certificate-s2-time` | 无 | `SUPERSEDED` | `SEMANTIC_CORRECTION` |

完整矩阵：[certificate-impact-matrix.zh-CN.md](certificate-impact-matrix.zh-CN.md)

### 7.1 claim 级证书覆盖

53 个 claim 的机器生成覆盖见
[s2-claim-certificate-coverage.zh-CN.md](s2-claim-certificate-coverage.zh-CN.md)：

```text
REISSUED              5
CANONICAL_ISSUED     36
SUPERSEDED            4
SUPERSEDED_BY_CLAIM   8
NOT_CERTIFIED         0
```

36 个新证书位于 `experiments/certificate-canonical-c56..c67,c73,c74..c96`，
由 `scripts/build-canonical-claim-certificates.py` 生成，绑定 claim、实验输出、
runner、engine、模型、证据源文件与 claims ledger 哈希，并由
`scripts/verify-canonical-claim-certificates.py` 校验。

全部 53 个 claim 均已有证书或明确的 superseded 关系；`NOT_CERTIFIED=0`。

## 8. 当前模型能力与科学边界

已建立：

- 单一 canonical event engine 的时钟、资源、能量、执行依赖语义。
- 40 个 canonical S2 迁移实验与 expected 输出（22 精确 + 18 语义修正）。
- 53 个 claim 的证书覆盖：36 个 `CANONICAL_ISSUED`、5 个 `REISSUED`、
  4 个 `SUPERSEDED`、8 个 `SUPERSEDED_BY_CLAIM`、0 个 `NOT_CERTIFIED`。
- 57 个辅助实验有 canonical 证据；2 个定义型/负结果记录按设计排除；0 个未覆盖。
- 独立 verifier、tamper 失败测试、clean-room 现有流程。
- 可审计的 S2 inventory、claims、claim→certificate 覆盖和证书影响矩阵。

范围声明：

- `S5-T SEMANTIC CONVERGENCE — ESTABLISHED (SCOPED)`：仅覆盖上述模型/仿真级
  canonical 证据和证书；不覆盖外部模型、物理设备或安全认证。

未建立：

- `S5 CROSS-MODEL VALIDITY`：尚未执行外部 FMU/FMI 协同仿真或独立数值比较。
- 物理安全、真实工业设备自主运行：未验证。
- 持久信任锚、多写者/分区/时钟偏移：未建立。
- witness 仍是引擎派生的执行依赖，不是 S1 求解器的原生 witness。
- 维修期撤销、抢占、跨进程状态：显式未实现。

## 9. 复现

```text
cargo run --locked -q -p poie-viability --bin poie-migration-c5
cargo run --locked -q -p poie-viability --bin poie-migration-c6
cargo run --locked -q -p poie-viability --bin poie-migration-c7
cargo run --locked -q -p poie-viability --bin poie-migration-c8
cargo run --locked -q -p poie-viability --bin poie-migration-c9
cargo run --locked -q -p poie-viability --bin poie-migration-c10
./scripts/check-all.sh
```

干净 target 复现：`CARGO_TARGET_DIR=/tmp/poie-s5t-clean-target ./scripts/check-all.sh` 于 2026-11-16 通过，`ok all checks`。

机器可读证据：[s2-evidence-inventory.json](s2-evidence-inventory.json)、[certificate-impact-matrix.json](certificate-impact-matrix.json)、[certificate-s2-metrics-split.json](certificate-s2-metrics-split.json)。
