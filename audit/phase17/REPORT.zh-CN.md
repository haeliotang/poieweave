# S5-V N17 — Counterfactual Intervention & Causal Oracle Independence

基线：`e2a5a925370ba7745040a7baaba2413862ab3200`。

```text
N17-0 HISTORICAL ORACLE INTEGRITY       VERIFIED
N17-A CONTROLLED INTERVENTION           ESTABLISHED (SCOPED)
N17-B CAUSAL ORACLE INDEPENDENCE        VERIFIED (SCOPED)
N17-C COUNTERFACTUAL RELATIONS          ESTABLISHED (SCOPED)
N17-D GENERATED/ADVERSARIAL             VERIFIED (TESTED DOMAIN)
N17-E EVIDENCE/CERTIFICATE              VERIFIED
N17-F REPRODUCIBILITY                   VERIFIED
```

## 固定干预对（10）

- 故障目标：sensor-x vs bus-1
- 故障时间：bus-1 7199 vs 7201
- 依赖边：split vs shared，bus-1@7201
- 资源资格：Task A m1 vs m2
- 故障插入：无故障 vs bus-1@7201
- 同刻边界：7199/7200、7200/7201
- 共享上游：bus-1 7199 vs 7201
- Tool AND：机器人变换但仍要求 tool-t1
- 组提交边界：bus-2@10799 vs 10801

每组四次真实执行；10/10 ACCEPTED，关系包括
CHANGED_OUTCOME_EXPLAINED / VALID_POLICY_DEPENDENT_DIVERGENCE。

## 生成式

两固定种子，32 个非重复干预对，64 canonical runs + 64 external runs；
32/32 valid/accepted，0 invalid，0 unexplained。

## 对抗与 mutant

- policy false-pass 三个控制均被拒绝；
- 干预未应用检测；
- 同刻事件顺序交换可检测；
- N16 mutant 提前 deactivate 共享 bus-1 被 oracle 拒绝：
  `SHARED_DEPENDENCY_LOSS`；
- reducer 保留双任务与 bus-1@7201，最小 1 故障。

## 限制

- 仿真内受控反事实，不是现实因果识别。
- 2 tasks / 2 robots / 固定命名上游图。
- 一个 eligibility+split 生成对暴露 canonical OR 上游释放缺口，已排除并记录。
- v9 证书不变，无 v10。
