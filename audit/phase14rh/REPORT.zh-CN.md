# S5-V N14-RH — Post-Migration Evidence Hardening

基线：`9a084980ae08c64f61b0a378d98ffba4c4ac57a2`。

```text
N14RH-0 RELEASE / HISTORY INTEGRITY        VERIFIED
N14RH-A V6 HISTORICAL EXECUTION REPLAY     VERIFIED (41/41)
N14RH-B V7 CLAIM-SPECIFIC EVIDENCE         VERIFIED (41/41)
N14RH-C INDEPENDENT FMI IMPORTERS          VERIFIED (10 CASES)
N14RH-D AUTOMATIC COUNTEREXAMPLE REDUCTION VERIFIED (SCOPED)
N14RH-E TWO-TASK ROBOT OR + TOOL AND       ESTABLISHED (v8 task OR)
N14RH-F REPRODUCIBILITY                    VERIFIED
```

## 已完成

- `9a08498` direct-main 更新审计，确认修改范围；main 当前无 branch protection。
- v7 41 证书/index/engine/N14R manifest 冻结。
- 在 detached `161273d` 历史 worktree 中从历史源码编译全部 bins，逐 claim 执行 41 个 runner，41/41 stdout 与 frozen expected 一致。
- 当前 v7 逐 claim replay 41/41，生成 41 个 `SUPPLEMENTARY_EXECUTION_ATTESTATION`，绑定 v6 replay、v7 cert、predecessor、runner/expected/actual。
- 同一 N14 FMU + 同输入分别由 C harness 与 fmpy 真实执行：N14R-00..09 全字段及完整 journal 完全一致。
- 自动 reducer 将 2 tasks / 4 failures 缩减为 1 task / 0 failures，保留 `RESOURCE_ELIGIBILITY_VIOLATION` 签名，并保存可重放证据。
- N14RH-E：新增逐任务 `robot_alternatives`，复用 scheduler per-task alternatives；OR-01/03/04/05 通过，OR-06 非法选择负例通过；需要 v8 证书迁移。

## 限制

```text
N14RH-E implemented by v8 task OR; N14RH-F CI and clean check-all passed.
See v8 certificate index and OR evidence for scope.
```
