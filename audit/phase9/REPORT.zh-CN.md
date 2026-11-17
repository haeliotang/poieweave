# S5-V N9 — Independent Resource-Causal Trajectory Challenge

基线：`60e2150a5a6e5e5f7cb9fa06e68bffb5e71b1c71`。

## 状态

```text
N9 COMPARATOR INTEGRITY
  VERIFIED (SCOPED: N8-H comparator)

N9 RESOURCE-CAUSAL OBSERVABILITY
  IN PROGRESS (witness export implemented; N9 trajectory pending)

N9 INDEPENDENT RESOURCE CONTRACT VALIDATION
  NOT ESTABLISHED

N9 RESOURCE-CAUSAL CROSS-MODEL TRAJECTORY
  NOT ESTABLISHED

N9 ADVERSARIAL VALIDITY
  PARTIAL (N8-H comparator controls)

N9 EVIDENCE AND CERTIFICATE INTEGRITY
  IN PROGRESS
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
