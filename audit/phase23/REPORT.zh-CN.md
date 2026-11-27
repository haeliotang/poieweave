# S5-V N23 — End-to-End Recovery Plan Binding & Temporal Causal Conformance

基线：`2dcbd4a9`。

```text
N23-0 HISTORY/FALSE-PASS      IN PROGRESS
N23-A NATIVE CHECKPOINT       VERIFIED — REAL PROJECTION + IDENTITY + SYNTHETIC REJECT
N23-B CANDIDATE/TIME          NOT ESTABLISHED
N23-C TRUSTED CHOICE-SET      NOT ESTABLISHED
N23-D NATIVE SCHEDULE         PARTIAL — external FMU exact schedule; canonical pending
N23-E CROSS-MODEL             PARTIAL — N23 FMI C/fmpy exact temporal fields
N23-F SECOND FAILURE          NOT ESTABLISHED
N23-G MUTANTS/CERTIFICATE     NOT ESTABLISHED
```

## Phase 0 已复现的缺口

### Temporal contract mismatch

```text
candidate robot-m3 completion = 20800
N22 FMI survived_s            = 19800
first divergence              = 1000s
```

证据：`failing-witness/N22_TEMPORAL_CONTRACT_MISMATCH/`（含原始 model.c、C 输出、
候选证书、replay.sh）。

### Synthetic checkpoint accepted

`"checkpoint_hash": "a"*64` 仍生成 2 个候选并被 reference 接受：

```text
SYNTHETIC_CHECKPOINT_ACCEPTED_BY_CHOICE_SET_PIPELINE
```

边界：只证明 bounded algorithmic choice-set validity，不证明 trusted
native-checkpoint-bound validity。

### Self-attested choice-set false pass

执行器只检查 proposal 自带的 candidate 列表/resources；伪造
`choice_set_certificate_hash` 与 `fabricated-candidate` 仍得到 `CompletedHorizon`：

```text
SELF_ATTESTED_CHOICE_SET_FALSE_PASS
```

### Comparator coverage

```text
observed_but_not_compared = survived_s
not_observed = task start/duration/completion, candidate payload, authorization id, choice-set hash, checkpoint identity
```

## Phase A — Native checkpoint binding

Native projection `native-checkpoints/native.json`, commitment
`94df8c4a...a8b7`, trusted provenance verified.

End-to-end identity:

```text
native checkpoint hash
  = candidate input checkpoint hash
  = choice-set certificate checkpoint hash
```

2 feasible candidates generated from real checkpoint; synthetic/untrusted/wrong
execution/missing asset/omitted completed/horizon mutations rejected.

## N23 independent FMU temporal execution

`external/fmu-n23-recovery-execution/` 独立支持 `taskStartConstraint` /
`taskDuration`，实际执行：

```text
robot-m2 start 9000  -> completion 19800  (legacy zero-wait preserved)
robot-m3 start 10000 -> completion 20800  (committed schedule)
```

事件时间区分：

```text
TaskRestarted @10000
TaskCompleted @20800
```

C/fmpy 两个 case 全部字段匹配；11 个 temporal false-pass 负例全部拒绝，
包括 `ROBOT_M3_COMPLETES_AT_19800`、wrong start、wrong duration、missing wait、
wrong survived time、event order、duplicate 和 post-termination event。

证据：`temporal-comparisons/strict.json`、`external/fmpy-n23.json`。

尚未完成 canonical 侧 committed schedule 执行，因此 N23-D/E 仍为 PARTIAL。
