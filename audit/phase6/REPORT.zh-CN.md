# S5-V N6 — Contract-Driven Coordination & Causal Boundary Challenge

基线：`c38ea8500426a5bc9ea9e4b521a9c4fcc0bf9423`（PR #576）。
本轮仅完成 N6-A 强制资源生命周期审计；N6-B..E 保持未完成。

## 状态

```text
N6-A RESOURCE LIFETIME SEMANTICS
  ESTABLISHED (SCOPED: group-level hold until group commit)

N6-A STEP-LEVEL RESOURCE RELEASE
  NOT REPRESENTABLE / NOT ESTABLISHED

N6-B CONTRACT-DRIVEN SCHEDULING — NOT ESTABLISHED
N6-C CAUSAL DEPENDENCY AND EVENT BOUNDARIES — NOT ESTABLISHED
N6-D REPAIR-DRIVEN RESTORATION STATE VALIDITY — NOT ESTABLISHED
N6-E EVIDENCE INTEGRITY — PARTIAL (N6-A only)

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

## 证据

- [PREREG.md](PREREG.md)
- [SEMANTICS.md](SEMANTICS.md)
- [canonical](canonical)
- [external](external)
- [differences/resource-lifetime.json](differences/resource-lifetime.json)
- [negative-controls/lifetime-results.json](negative-controls/lifetime-results.json)
- [manifest.json](manifest.json)

## 验证

```text
python3 audit/phase6/check.py
```

## 未覆盖

- 参数化外部调度器（N6-B）
- 工具与上游依赖失效（N6-C）
- 同刻事件与 horizon 边界（N6-D）
- 完整恢复前后资产状态（N6-D）
- 完整 Linux CI 证据包（N6-E）尚未为本阶段单独建立；N6-A 使用本地验证
