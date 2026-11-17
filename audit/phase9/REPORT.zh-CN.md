# S5-V N9 — Independent Resource-Causal Trajectory Challenge

基线：`60e2150a5a6e5e5f7cb9fa06e68bffb5e71b1c71`。

## 状态

```text
N9 COMPARATOR INTEGRITY
  VERIFIED (SCOPED: N8-H comparator)

N9 RESOURCE-CAUSAL OBSERVABILITY
  ESTABLISHED (SCOPED: canonical native witness + external native resource outputs)

N9 INDEPENDENT RESOURCE CONTRACT VALIDATION
  ESTABLISHED (SCOPED: eight enumerated cases)

N9 RESOURCE-CAUSAL CROSS-MODEL TRAJECTORY
  ESTABLISHED (SCOPED: N9-00..N9-06, N9-11)

N9 ADVERSARIAL VALIDITY
  ESTABLISHED (SCOPED: 11 resource-contract controls)

N9 EVIDENCE AND CERTIFICATE INTEGRITY
  ESTABLISHED (SCOPED)
```

## N9-0 完成内容

新增 N8-H 比较器与证据完整性负例：

```text
维修耗能符号反转          MODEL_ASSUMPTION_MISMATCH
Before/After 相位交换      EVENT_PHASE_MISMATCH
原始 CSV 篡改              EVIDENCE_INTEGRITY_FAILURE
manifest 内容/digest 篡改  EVIDENCE_INTEGRITY_FAILURE
证据文件缺失               EVIDENCE_INTEGRITY_FAILURE
错误执行批次               EVIDENCE_INTEGRITY_FAILURE
声明 MATCH 但 findings 非空 CLAIM_INCONSISTENCY
```

数值比较与证据完整性已分离：

```text
Raw File -> SHA-256 -> Frozen Manifest + manifest.sha256
        -> Independent verifier
        -> Strict comparator
        -> Claim consistency check
```

既有 N8-H 正例三个场景仍全部 `MATCH_WITH_EXPLAINED_ASSUMPTION`。

## 资源见证导出（只读）

`RepairObservation` 新增：

```text
execution_dependencies
restoration_targets
witness (capability/provider/dependencies)
```

通过 `run_engine_event_observed` 可读取当次 RunningRepair 的资源选择与依赖见证。
这是 observation-only 变更；已冻结 v4 并生成 41 个 v5 successors。

## 资源能力审计

见 [CAPABILITY_MATRIX.md](CAPABILITY_MATRIX.md)。当前关键缺口：

- canonical 无原生资源释放事件；终止不等于释放。
- canonical witness/execution_dependencies 未导出到 RepairObservation。
- N5 无工具/上游资源输出。
- N6 上游故障为 generic failureKind3，无命名上游资产。
- 无 step-level release。

## 七场景验证结果

```text
N9-00 single task, no failure                  ACCEPTED
N9-01 two tasks, one robot                    ACCEPTED
N9-02 two tasks, two robots                   ACCEPTED
N9-03 selected robot failure @7200             ACCEPTED
N9-04 unselected robot failure @7200           ACCEPTED
N9-05 required tool failure @9000              ACCEPTED
N9-06 upstream bus failure @9000               ACCEPTED
```

- canonical selected resources 来自原生 RunningRepair。
- execution dependencies / witness 来自 v5 只读导出。
- external FMU 自行计算 robot/tool/makespan。
- N9-06 的外侧模型只有 generic failureKind=3，无命名上游资产；
  upstream asset identity 标记为 NOT_AVAILABLE，只验证中断因果。
- energy trajectory 全部 NOT_COMPARABLE，没有填零或复制 N4 曲线。
- N9-11 双方顺序不同但都合法：canonical a→b，external b→a；分类
  `DIFFERENT_POLICY_VALID_DECISIONS`，未判为失败。
- N9-11 robot mask 允许同一机器人上的合法 OR/替代选择；实际选择相同。
- resource release 只有外部 tool/group release 字段；canonical native release
  NOT_OBSERVED，终止未被当作释放。

## 测试参数

```text
tasks: 1 or 2
robot pools: 1 or 2
tool: tool-t1 (N9-05)
upstream: bus-1 (N9-06)
failure times: 7200, 9000
group boundaries: 7200, 10800, 18000
step: 900 s
```

## 未建立

- 任何 N9 资源因果跨模型轨迹声明。
- N5/N6 资源生命周期跨模型轨迹。
- 事件引擎 priority、重规划、自主恢复、Level 3。
- 物理正确性。

## 下一步

1. 读取/扩展只读 RunningRepair 资源 witness（先做证书影响评估）。
2. 建立 N9 canonical resource event export。
3. 扩展 N6 FMU 只读上游/资源事件导出，保持调度规则不变。
4. 实现 N9 strict resource comparator 与六个核心场景。
5. 负例覆盖共同资源契约违反。
