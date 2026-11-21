# S5-V N12 — Native State Projection & Causal Identity Integrity

基线：`7b7a8373a6ca5e5065d921035e438c82368a0e58`。

## 状态

```text
N12-0 HISTORICAL EVIDENCE CORRECTION
  VERIFIED

N12-A NATIVE EVENT IDENTITY INTEGRITY
  ESTABLISHED (SCOPED)

N12-B BIDIRECTIONAL NATIVE STATE PROJECTION
  ESTABLISHED (SCOPED: target asset health dimensions; repair-state NOT_COMPARABLE)

N12-C BIDIRECTIONAL CAUSAL STATE-TRANSITION
  ESTABLISHED (SCOPED: 14 cases; target identity/time/order/asset health)

N12-D ADVERSARIAL AND STRESS VALIDITY
  ESTABLISHED (SCOPED: native mutations, overflow=3 execution, 1/900/1000 step invariance, multi-instance journal)

N12-E EVIDENCE AND CERTIFICATE INTEGRITY
  ESTABLISHED (SCOPED)
```

## 本轮完成

- N11 `CLAIMS.json` N11-0 修正为 VERIFIED；报告/C 的过期说明修正。
- 原始 N11 报告、journal、证据未改写；新增 [N11_REVALIDATION.md](N11_REVALIDATION.md)。
- 复现 N11 事件目标身份 false pass：U01/U02/U03/U04 的 `AssetFailure` 全部记录为 `robot-m1`，旧验证器 ACCEPTED。
- N11 validator 删除 `N10-OR2` case-specific 分支。
- 新 N12 FMU 从真实 failure 输入解析目标：
  - kind1 → robot-m{index}
  - kind2 → tool-t1
  - kind3 → contract failureAssetId 对应命名资产
- N12 journal 新增 `target_id`；`AssetFailure.target` 为目标名称。
- 14 个案例身份验证：失败目标、horizon 未应用、harness/fmpy 一致全部通过。
- 修复 `fmi3GetString` 的静态共享缓冲区，改为实例拥有的输出缓冲区。

## 限制

```text
canonical repair-state before/after = NOT_OBSERVED
restoration 维度 = NOT_COMPARABLE
N12 FMU 仍仅有单故障输入
N12 both-invalid 为 native-evidence mutation，不是真实双模型共同缺陷执行
```
N11 历史 journal 的身份错误保留为研究反例；受影响历史声明的目标身份范围收窄。
