# S5-V N14-RH — Post-Migration Evidence Hardening

基线：`9a084980ae08c64f61b0a378d98ffba4c4ac57a2`。

```text
N14RH-0 RELEASE / HISTORY INTEGRITY        VERIFIED
N14RH-A V6 HISTORICAL EXECUTION REPLAY     VERIFIED (41/41)
N14RH-B V7 CLAIM-SPECIFIC EVIDENCE         VERIFIED (41/41)
N14RH-C INDEPENDENT FMI IMPORTERS          VERIFIED (10 CASES)
N14RH-D AUTOMATIC COUNTEREXAMPLE REDUCTION VERIFIED (SCOPED)
N14RH-E TWO-TASK ROBOT OR + TOOL AND       NOT ESTABLISHED
N14RH-F REPRODUCIBILITY                    IN PROGRESS
```

## 已完成

- `9a08498` direct-main 更新审计，确认修改范围；main 当前无 branch protection。
- v7 41 证书/index/engine/N14R manifest 冻结。
- 在 detached `161273d` 历史 worktree 中从历史源码编译全部 bins，逐 claim 执行 41 个 runner，41/41 stdout 与 frozen expected 一致。
- 当前 v7 逐 claim replay 41/41，生成 41 个 `SUPPLEMENTARY_EXECUTION_ATTESTATION`，绑定 v6 replay、v7 cert、predecessor、runner/expected/actual。
- 同一 N14 FMU + 同输入分别由 C harness 与 fmpy 真实执行：N14R-00..09 全字段及完整 journal 完全一致。
- 自动 reducer 将 2 tasks / 4 failures 缩减为 1 task / 0 failures，保留 `RESOURCE_ELIGIBILITY_VIOLATION` 签名，并保存可重放证据。
- N14RH-E 未实现：当前 `ExplicitRepairTask` 无逐任务 robot OR 组；需要新的 execution-semantic extension 与 v8 迁移，本轮未做。

## 限制

```text
N14RH-E task-specific OR + tool AND = NOT ESTABLISHED
N14RH-F clean CI pending
```
