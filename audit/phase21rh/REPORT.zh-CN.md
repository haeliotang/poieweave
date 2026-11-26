# S5-V N21-RH — Checkpoint Integrity, Authorization Durability & Cross-Model Conformance

基线：`3332755f431da1ac9e9d117ae5474abc5d27942c`。

```text
N21RH-0 HISTORY/FALSE-PASS      IN PROGRESS
N21RH-A CHECKPOINT INTEGRITY    PARTIAL — SHA-256 COMPLETE COMMITMENT + V12
N21RH-B AUTHORIZATION DURABILITY PARTIAL — FILE LEDGER, PROCESS AUTH, STRICT RESUME WRAPPER
N21RH-C FMI CONFORMANCE         PARTIAL — STRICT FIELD COMPARATOR, NO EVENT JOURNAL
N21RH-D REDUCER                 VERIFIED — 27000 -> 11400, SIGNATURE PRESERVED
N21RH-E DOMAIN BOUNDARY         VERIFIED — 3 STILL NOT REPRESENTABLE
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

### Phase B 授权持久性（partial）

- `ReplanAuthorization` 增加 issuer、issued_at、expiration、signature；
  research-only 固定测试密钥，不是生产凭据。
- `FileAuthorizationLedger` 使用 `create_new` 原子 claim：
  UNCONSUMED → CLAIMED → COMMITTED；未 commit 的重放 fail-closed。
- 8 进程并发 claim 精确 1 个成功。
- durable resume 首次完成并 commit，第二次被拒绝。
- forged issuer / invalid signature / expired / wrong proposal 均拒绝。
- 未证明分布式 exactly-once 或 resume 中途崩溃恢复。

### Phase C 严格比较（partial）

`audit/phase21rh/strict_compare.py` 只比较 C harness 与 fmpy 原生输出字段，
删除 `or` 与固定 19800 回退。5 个 false-pass 负例均被拒绝；缺失字段视为
不匹配。FMI 事件 journal 未提供，事件级等价 NOT ESTABLISHED。

### Phase D 自动 reducer

`audit/phase21rh/reducer_auto.py` 真实构建 normal/M3 FMU，并对候选 t1Duration、
failureTime 重执行：

```text
original complexity (t1+t2+failure) = 27000
minimized complexity               = 11400
t1 7200 -> 100
failure 9000 -> 500
signature COMPLETED_TASK_ERASURE preserved
```

证据：`reducer/results.json`、`minimized_contract.json`、`replay.sh`。

### Phase E 边界

3 个原 NOT_REPRESENTABLE 场景全部重新审计，仍不可表示：

```text
previously 3 / newly 0 / still 3
```

## 未完成

- durable checkpoint `load_checkpoint` 与完整 lineage。
- resume 中途崩溃恢复（当前仅 at-most-once / fail-closed）。
- FMI 原生事件 journal 与多 step/多 case importer 测试。
- 真实自动 reducer 与可执行缩减。
- N21-05/07/16 可表示性复核。
- N21RH external certificate、clean check-all。
