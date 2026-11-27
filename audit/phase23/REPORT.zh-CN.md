# S5-V N23 — End-to-End Recovery Plan Binding & Temporal Causal Conformance

基线：`2dcbd4a9`。

```text
N23-0 HISTORY/FALSE-PASS      IN PROGRESS
N23-A NATIVE CHECKPOINT       NOT ESTABLISHED
N23-B CANDIDATE/TIME          NOT ESTABLISHED
N23-C TRUSTED CHOICE-SET      NOT ESTABLISHED
N23-D NATIVE SCHEDULE         NOT ESTABLISHED
N23-E CROSS-MODEL             NOT ESTABLISHED
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
