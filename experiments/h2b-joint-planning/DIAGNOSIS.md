# H2b 开发诊断：场景不可行，留出集保持封存

日期：2026-11-06
状态：**开发诊断失败；未运行留出集；H2b 主假设未得到检验。**

## 1. 第一轮：Base-01 母线故障不可恢复

在 Base-01 上首次运行开发集（16 场景）：

- 所有双目标任务失败（A 0/16，基线 0/16）
- 未达成原因：`restore:bus-1:blocked: cycle bus-1` 与 `potable-water:blocked: cycle bus-1`
- 根因：`robot-m1 → charger-1 → bus-1`；母线失效后机器人无电，无法修复母线自身。

原始输出：[`raw/dev-base01-invalid.txt`](raw/dev-base01-invalid.txt)。

这说明 Base-01 的 `bus-1` 失效在 4 h 内是**模型不可恢复环**；H2 原实验把母线失效放在 12–20 h（截止之后），所以未暴露。

## 2. 第二轮：Base-03 独立发电机

为避免该环，新增：

- [`scenarios/base-03/model.toml`](../../scenarios/base-03/model.toml)
- 仅一处差异：`charger-1.requires = ["generator-1"]`，发电机可用且不依赖母线

开发集重跑（16 场景）：

| 方法 | 成功 | 动作数 | 典型未达成 |
| --- | --- | --- | --- |
| A_joint | 0/16 | 1.6 | deadline missed / missing electric-energy |
| B1/B2/B3 | 0/16 | 2.2 | 同上 |
| abl_no_replanning | 0/16 | 1.0 | — |

原始输出与计划签名：

- [`raw/dev-raw.txt`](raw/dev-raw.txt)
- [`raw/dev-diagnose.txt`](raw/dev-diagnose.txt)
- [`dev-report.md`](dev-report.md)

## 3. 为什么仍然 0/16

母线恢复 1 h + 泵恢复 3 h = **4 h 串行**，且泵在母线恢复前不可用。场景网格中 `bus_failure_h ≥ 0.5 h`，任何故障都使完成时间 ≥ 4.5 h，超过 4 h 服务截止。

计划签名直接显示：

```text
A_joint: bus@2.0-3.0, pump@3.0-6.0
B1:      bus@2.0-3.0, pump@3.0-6.0, bus-again@6.0-7.0
```

A 已经做出不同于基线的两动作顺序，但**场景本身不可行**，因此不能比较成功率。

## 4. 结论

- H2b 当前登记版本**不成立**，不能运行留出集，也不能据此判断规划器优劣。
- 可复用的诊断结论：
  1. Base-01 的 bus 失效是修复环，不应在截止内直接使用；
  2. bus + pump 串行 4 h 在 4 h 截止下没有 feasible 场景；
  3. 需要新预登记：更长截止、独立泵电源，或不同资源竞争对（如泵 + 车床，并补 spare lathe-nut）。
- 留出集未运行；`PREREG.md` 的判定规则未被使用。

## 5. 下一步（需新实验 ID）

建议 H2c：

- 场景：泵 + 车床同时失效，车床修复需要 spare `lathe-nut`；
- 期限：服务 4 h、车床 8 h；
- 成功：两者都在各自期限前恢复；
- 预登记先提交，再跑 dev；
- 新 holdout 与新 seed，不复用 H2b/H2 holdout。
