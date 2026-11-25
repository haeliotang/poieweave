# S5-V N20 — Independent Causal Reference Oracle

基线：`8e582125f3a2418e3d0d726f93e7ea88df044680`。

```text
N20-0 HISTORY/CLAIM INTEGRITY       VERIFIED
N20-A 14 HELD-OUT ADJUDICATIONS     14/14 ORACLE_FALSE_POSITIVE
N20-B REFERENCE ORACLE              ESTABLISHED (BOUNDED DOMAIN)
N20-C REFERENCE SOUNDNESS           VERIFIED (TESTED DOMAIN)
N20-D CURRENT-RUN MUTANTS           VERIFIED
N20-E HELD-OUT GENERALIZATION       VERIFIED (TESTED DOMAIN)
N20-F EVIDENCE/REPRODUCIBILITY      IN PROGRESS
```

## 14 个历史 held-out 失败

全部从 seed 1901/1919 重建输入并四路重放。独立参考 oracle 在执行前生成
admissible schedules，实际 canonical/external 调度均匹配参考集合；参考方向
为 `HIT -> NO_HIT`，N19 生成器只覆盖 `NO_HIT -> HIT`，因此 14/14 裁决为
`ORACLE_FALSE_POSITIVE`，次级原因 `EXPECTED_EFFECT_DIRECTION_ERROR`。
原始 N19 ledger 保持 14 unexpected。

## 独立参考 oracle

`audit/phase20/reference_oracle/reference.py` 仅依赖标准库，自行枚举：
机器人分配、并行/串行顺序、起止时间、依赖闭包、故障应用、中断/完成。
不调用 canonical engine、scheduler、FMU、`model_outcome()` 或 `R.classify()`。

参考预测在四路执行前写入并哈希：
- 14 对 × 2 = 28 份 reference predictions；
- 新 held-out 64 对 × 2 = 128 份 reference predictions。

## 新 held-out

4 个新预登记种子（2001/2017/2033/2049）：
64 attempted、64 valid、64 executed、64 accepted、0 rejected、0 unresolved。
接受条件是参考集合匹配 + 每侧执行契约有效 + 关系一致。

## 当前执行 mutants

M1 spurious interruption、M2 OR path misbinding、M3 premature shared release
均在 N20 当次执行，正式 oracle 拒绝；未复用历史签名。

## 限制

- 参考 oracle 只覆盖 1–2 任务、2 机器人、固定命名上游。
- 接受是相对参考规范，不是现实物理真值。
- 64 个新 held-out 均 accepted，未构造独立第三方工业验证。
- v10 未变，无 v11。
