# S5-V N21-RH — Checkpoint Integrity, Authorization Durability & Cross-Model Conformance

基线：`3332755f431da1ac9e9d117ae5474abc5d27942c`。

```text
N21RH-0 HISTORY/FALSE-PASS      IN PROGRESS
N21RH-A CHECKPOINT INTEGRITY    PARTIAL — SHA-256 COMPLETE COMMITMENT + V12
N21RH-B AUTHORIZATION DURABILITY IN PROGRESS
N21RH-C FMI CONFORMANCE         IN PROGRESS
N21RH-D REDUCER                 IN PROGRESS
N21RH-E DOMAIN BOUNDARY         IN PROGRESS
N21RH-F CERTIFICATE             PARTIAL — V12 SUCCESSORS ISSUED
```

## 已完成

- 审计并复现旧 checkpoint 哈希字段遗漏：`selected_resources`、
  `applied_failures`、`pending_failures`、`trace_history`、
  `completed_plans`、`completed_planning_base`、`predecessor_hash`、
  `event_sequence` 等未完整覆盖；旧算法为 64-bit `DefaultHasher`。
- 新 `poie.repair-checkpoint/v12` 规范：确定性编码 + SHA-256 域分隔承诺，
  覆盖全部恢复相关字段；拒绝非有限浮点，规范化 -0.0。
- 新增 `verify_trusted_checkpoint`、`save_checkpoint`、`verify_checkpoint_file`：
  内容完整性之外要求 trusted provenance ledger 包含 commitment。
- 全字段 mutation 测试：16 个字段篡改均改变 commitment；重新计算哈希的
  伪 checkpoint 仍被 trusted ledger 拒绝；文件篡改被拒绝。
- v11 41 证书冻结为历史字节；v12 41 个 successor 证书已签发并通过
  v12 verifier/tamper。

## 未完成

- 跨进程持久授权消费 ledger、并发双消费、崩溃 fail-closed。
- 严格 C/fmpy 比较器与 false-pass 控制。
- 真实自动 reducer 与可执行缩减。
- N21-05/07/16 可表示性复核。
- N21RH external certificate、clean check-all。
