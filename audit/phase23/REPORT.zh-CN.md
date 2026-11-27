# S5-V N23 — End-to-End Recovery Plan Binding & Temporal Causal Conformance

基线：`2dcbd4a9`。

```text
N23-0 HISTORY/FALSE-PASS      IN PROGRESS
N23-A NATIVE CHECKPOINT       VERIFIED — REAL PROJECTION + IDENTITY + SYNTHETIC REJECT
N23-B CANDIDATE/TIME          NOT ESTABLISHED
N23-C TRUSTED CHOICE-SET      PARTIAL — RESEARCH VALIDATOR, NOT CANONICAL EXECUTOR
N23-D NATIVE SCHEDULE         PARTIAL — external FMU exact schedule; canonical pending
N23-E CROSS-MODEL             PARTIAL — N23 FMI C/fmpy exact temporal fields
N23-F SECOND FAILURE          VERIFIED — external FMI + canonical consumed authorization
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

## Continuation second failure (external FMI)

独立 N23 FMU 现在支持 `continuationFailureTime` / `continuationFailureTarget`：

```text
relevant@15000 + robot-m3  -> interrupted, no restoration, executed robot remains 3
irrelevant@12000            -> observed, completion 20800 continues
before start @9500          -> no interruption
at completion @20800        -> no interruption
```

4 个 case C/fmpy 字段全部匹配；未发生自动候选切换（executedRobot 始终为 3）。
证据：`continuation-faults/results.json`。

canonical 侧消费授权持久性与 no-auto-fallback 仍待验证。

## Trusted certificate enforcement in canonical

`resume_engine_with_authorization` 现在要求：

- proposal 携带 choice-set certificate payload；
- certificate hash = SHA256(domain || payload) 与 proposal 一致；
- certificate hash 在 research trusted ledger 中；
- certificate checkpoint/execution identity 匹配；
- selected candidate 属于证书 feasible set。

self-attested fabricated certificate 现在返回 BLOCKED，而非 CompletedHorizon。

因 canonical 语义变化，v14 41 证书冻结；v15 41 个
`AUTHORIZATION_SEMANTIC_CORRECTION` successors 签发并通过 verifier/tamper。

## Canonical second-failure persistence

在 continuation 期间 robot-m2 于 15000 再次故障：

```text
first resume  -> BLOCKED continuation interrupted
second resume -> BLOCKED authorization already consumed
automatic second replan / candidate switch -> none
```

证据：`continuation-faults/canonical-persistence.json`。

## Full candidate payload binding

Certificate payload now includes selected candidate `resources` + `planned_start_s`;
canonical resume rejects `authorized candidate payload mismatch`. v15 41
certificates frozen; v16 41 `AUTHORIZATION_SEMANTIC_CORRECTION` successors issued
with verifier/tamper.

## Executable mutants M1–M4

```text
M1 synthetic checkpoint   -> SYNTHETIC_CHECKPOINT
M2 schedule substitution  -> ROBOT_M3_COMPLETES_AT_19800 rejected
M3 candidate substitution -> AUTHORIZATION_FOR_DIFFERENT_CANDIDATE
M4 hidden second replan   -> no automatic switch, executed robot remains 3
```

证据：`executable-mutants/results.json`。reducer 仍为 NOT_ESTABLISHED。

## Real reducer M2

从 5 个 resource options 的复杂 contract 自动删除无关 tool/horizon/failed 选项，
每个候选都重新运行 generator + reference oracle + trust verifier：

```text
original complexity = 5
minimized complexity = 2
signature AUTHORIZED_CANDIDATE_PAYLOAD_MISMATCH preserved
```

证据：`minimized-counterexamples/m2-reducer.json`。

## Resource-derived candidates

`resource_discovery.py` 从可信资产模型（availability / requires / ready_at / failure）
派生 robot options，不再依赖预填 `robot_options`：

```text
derived robots = [robot-m2, robot-m3]
soundness = true
completeness = true
```

证据：`candidate-generation/resource-discovery.json`。
