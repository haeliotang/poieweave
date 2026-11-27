# S5-V N23 — End-to-End Recovery Plan Binding & Temporal Causal Conformance

基线：`2dcbd4a9`。

```text
N23-0 HISTORY/FALSE-PASS      IN PROGRESS
N23-A NATIVE CHECKPOINT       VERIFIED — REAL PROJECTION + IDENTITY + SYNTHETIC REJECT
N23-B CANDIDATE/TIME          NOT ESTABLISHED
N23-C TRUSTED CHOICE-SET      PARTIAL — RESEARCH VALIDATOR, NOT CANONICAL EXECUTOR
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

## Trusted choice-set certificate verifier

`choice_set_trust.py` 从 e2e certificate 取 feasible set 与完整 payload，验证：

- schema、certificate commitment；
- checkpoint identity、execution identity；
- selected candidate ∈ feasible；
- candidate payload hash 重新计算等于 selected hash；
- authorization 的 certificate/checkpoint/candidate binding。

负例：

```text
SELF_ATTESTED_FABRICATED_CANDIDATE -> CANDIDATE_NOT_IN_SET
STALE_CHOICE_SET                  -> STALE_CHOICE_SET
CHECKPOINT_SWAP                   -> CHECKPOINT_CHANGED_AFTER_SELECTION
AUTHORIZATION_FOR_DIFFERENT       -> AUTHORIZATION_FOR_DIFFERENT_CANDIDATE
AUTHORIZED_SCHEDULE_MISMATCH      -> AUTHORIZED_CANDIDATE_PAYLOAD_MISMATCH
```

该验证器尚未接入 canonical `resume_engine_with_authorization`；Gate N23-C 仍未成立。

## Canonical committed schedule + v14

`ReplanProposal.planned_start_s` 纳入 v13 proposal commitment；resume 使用
`max(checkpoint.at_s, planned_start_s)` 调度：

```text
robot-m2 planned_start 0     -> survived 19800 (legacy preserved)
robot-m3 planned_start 10000 -> survived 20800 (committed)
```

测试：`canonical_committed_planned_start_is_enforced`。
v13 41 证书字节冻结；v14 41 个 `EXECUTION_SEMANTIC_EXTENSION` successors
签发并通过 verifier/tamper。
