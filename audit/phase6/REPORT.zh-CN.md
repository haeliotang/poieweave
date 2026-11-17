# S5-V N6 — Contract-Driven Coordination & Causal Boundary Challenge

基线：`c38ea8500426a5bc9ea9e4b521a9c4fcc0bf9423`（PR #576）。
范围：N6-A group-level resource lifetime、N6-B parameterized scheduling、N6-C dependency/event boundaries、N6-D derived restoration state、N6-E evidence integrity，以及 N6 evidence hardening。所有声明均为 SCOPED。

## 状态

```text
N6-A RESOURCE LIFETIME SEMANTICS
  ESTABLISHED (SCOPED: group-level hold until group commit)

N6-A STEP-LEVEL RESOURCE RELEASE
  NOT REPRESENTABLE / NOT ESTABLISHED

N6-B CONTRACT-DRIVEN SCHEDULING — ESTABLISHED (SCOPED: durations, task count 1–3, robot pool, scheduler-level priority contract)
N6-C CAUSAL DEPENDENCY AND EVENT BOUNDARIES — ESTABLISHED (SCOPED: tool-t1, bus-1, same-time, horizon)
N6-D REPAIR-DRIVEN RESTORATION STATE VALIDITY — ESTABLISHED (SCOPED: derived asset state from completion trace; native asset-status export unavailable)
N6-E EVIDENCE INTEGRITY — ESTABLISHED (SCOPED: A/B/C + derived D)

S5-V N6 CONTRACT-DRIVEN COORDINATION
AND CAUSAL BOUNDARY CHALLENGE
  ESTABLISHED (SCOPED)

S5-V N6 EVIDENCE HARDENING — ESTABLISHED (SCOPED)
N6 FROZEN EVIDENCE REPRODUCIBILITY — VERIFIED
N6 INDEPENDENT SCHEDULING CONTRACT VALIDATION — ESTABLISHED (SCOPED)
N6 REPORT / EVIDENCE CONSISTENCY — VERIFIED

N5-03 GROUP-LEVEL INTERRUPTION CONSISTENCY
  ESTABLISHED
N5-03 STEP-LEVEL LIFETIME CORRECTNESS
  NOT ESTABLISHED
```

## N6-A result

Canonical 将选中资源视为组级占用，直到整个维修组 makespan 提交。

| failure | applied | group commit | interrupted | restored |
| ---: | ---: | ---: | --- | --- |
| 7199 | 7199 | none | yes | no |
| 7200 | 7200 | none | yes | no |
| 7201 | 7201 | none | yes | no |
| 9000 | 9000 | none | yes | no |
| 10800 | 10800 | 10800 | no | yes |

- 7200 s 时 lathe 步骤已到名义结束，但 robot-m1 仍被组级持有；
- 7200/7201 s 失效会中断整组；
- 10800 s 时组完成先于外生故障，恢复提交后故障被记录；
- 外部 N5 scheduler FMU 采用相同组级语义，5 个边界全部 EXPLAINED；
- 负例覆盖错误步骤级释放、提交后中断、移动步骤、延迟失效、缺失恢复。

详细见 [RESOURCE_LIFETIME.md](RESOURCE_LIFETIME.md)。

## N6-B 参数化调度

新增独立参数化 C99 FMI 3.0 scheduler，从 contract 文件读取：

```text
taskCount, task durations, task priorities, robot masks, tool requirements,
robotCount, failure time/kind/index, horizon
```

不再按固定实验编译最终安排。N6-B 案例：

| case | 参数变化 | canonical | external | decision/status |
| --- | --- | ---: | ---: | --- |
| B00 | 默认两任务 | 10800 | 10800 | SAME |
| B01 | lathe 5000 s | 10800 | 10800 | SAME |
| B02 | pump 9000 s | 9000 | 9000 | SAME |
| B03 | 5000/9000 | 9000 | 9000 | SAME |
| B04 | 单任务 | 7200 | 7200 | SAME |
| B05 | 三任务 | 13200 | 13200 | SAME |
| B06 | 反转优先级、单机器人 | 18000（pump→lathe） | 18000（pump→lathe） | SAME |
| B07 | 单机器人串行对照 | 18000 | 18000 | SAME |
| B08 | 外部 priority 与 canonical 计划顺序不同 | 18000（a→b） | 18000（b→a） | DIFFERENT_POLICY_VALID_DECISIONS |

B06 现在由双方从同一 priority 契约独立计算，决策一致。
B08 故意使用不同 priority 输入，两种顺序都合法且 makespan 相同，比较器分类为
`DIFFERENT_POLICY_VALID_DECISIONS`，而不是失败。

边界：canonical scheduler 通过 `plans` 顺序接收 priority；canonical **event engine**
的 `start_repair` 仍按固定初始资产顺序生成计划，因此“事件执行阶段的 priority
输入”仍为 NOT REPRESENTABLE。N6-B 的 priority 声明只覆盖 scheduler-level contract。

负例覆盖资源重叠、任务时长篡改、越池资源、缺失恢复、makespan 移动。

## N6-C/D 依赖与事件边界

工具失效（tool-t1）：

| case | failure | canonical | external |
| --- | ---: | --- | --- |
| C-tool-1 | 1 s | interrupted at 1 s | interrupted at 1 s |
| C-tool-9000 | 9000 s | interrupted at 9000 s | interrupted at 9000 s |
| C-tool-10800 | 10800 s | completed, failure after commit | completed, failure after commit |
| C-tool-12000 | 12000 s | completed, failure after release | completed, failure after release |

上游依赖（bus-1）：9000 s 失效中断，10800 s 在提交后失效。

event boundaries：

```text
N6-D-same-10800: completion before same-time failure
N6-D-horizon-completion: completion at horizon, failure at horizon NOT processed
N6-D-horizon-failure: repair completes earlier; horizon failure NOT processed
```

外部 FMU 由 contract 驱动，并通过独立事件顺序处理；9 个 C/D 场景全部 EXPLAINED。
负例覆盖工具失效被忽略、bus 失效被忽略、完成时错误中断、horizon 接纳故障、
同刻顺序反转、恢复缺失。

## 恢复状态（N6-D）

`restoration/restoration-state.json` 记录目标资产 before/after、完成/提交/释放时间。
当前限制：

```text
NATIVE ASSET STATUS EXPORT — NOT AVAILABLE
status_source = DERIVED_FROM_TRACE_COMPLETION
```

因此 N6-D 标记为 **ESTABLISHED (SCOPED: derived)**：恢复目标、before/after、
提交与释放时间可审计，但资产状态来源是 canonical completion trace 的确定性派生，
不是原生资产状态字段。该限制在 EVIDENCE.md 中保留。

## Evidence hardening

- 冻结 manifest 仅由 `git ls-files audit/phase6` 的 allowlist 文件生成，排除
  `__pycache__`、`.pyc`、运行时与执行 manifest。
- 冻结校验先于任何重算执行；当次运行使用独立 `execution-manifest.json`。
- 调度契约逐任务校验 robot mask、必需工具、工具占用/释放与 priority。
- 新增 mutation controls：mask 禁止机器人、无合法机器人、缺失工具、提前释放工具、
  同 priority 顺序违规、契约输入变更、冻结证据篡改。
- B06/B08 分类不再依赖 case ID，而由实际 priority 输入和资源竞争关系决定。
- `selected_resources` 缺失时记录为 `null` + `selected_resources_observed=false`，
  不把空数组解释为“无需资源”。
- 冻结证据仅来自 Git-tracked allowlist；当次运行使用独立 execution manifest。
- 干净 checkout 必须先通过冻结校验，再允许任何重算。


- [PREREG.md](PREREG.md)
- [SEMANTICS.md](SEMANTICS.md)
- [canonical](canonical)
- [external](external)
- [differences/resource-lifetime.json](differences/resource-lifetime.json)
- [negative-controls/lifetime-results.json](negative-controls/lifetime-results.json)
- [manifest.json](manifest.json)
- [EVIDENCE.md](EVIDENCE.md)

## 验证

```text
python3 audit/phase6/check.py
```

## 未覆盖 / 范围限制

- step-level resource release（canonical 仅组级）
- event-engine priority input（仅 scheduler-level）
- 原生 asset-status export（N6-D 为 completion trace 派生）
- post-interruption replanning
- autonomous recovery
- Level 3 independent planning
- trajectory-level / full S2 / physical-world correctness
- 本阶段未修改 canonical 运行时，也未重签 S2 证书
