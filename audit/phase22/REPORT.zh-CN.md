# S5-V N22 — Recovery Choice-Set Independence & Decision Validity

基线：`6fa4e047`。

```text
N22-0 HISTORY/PROPOSAL        IN PROGRESS — N21RH-0 supplement + v13 reference commitment
N22-A CANDIDATE GENERATION    VERIFIED (BOUNDED PYTHON DOMAIN)
N22-B DECISION VALIDITY       VERIFIED (BOUNDED)
N22-C CHOICE-SET INTEGRITY    VERIFIED (BOUNDED)
N22-D AUTHORIZED EXECUTION    PARTIAL — binding verifier only, no native execution
N22-E CROSS-MODEL             NOT ESTABLISHED
N22-F ADVERSARIAL/CERT        PARTIAL — CANDIDATE-PIPELINE MUTANTS + REDUCER
```

## 已完成

- 冻结 N21-RH 证据与 v12 certificate index。
- 新增 N21-RH 复核：`N21RH-0 = VERIFIED_BY_SUPPLEMENT`，不改写历史 claims。
- 建立 `poie.replan-proposal/v13` 独立承诺参考实现：确定性 JSON + SHA-256；
  5 个 proposal mutation 全部改变 commitment；ID-only 改变不被当作同一内容身份。
- 记录 proposal ID / content commitment / checkpoint hash / authorization ID
  的语义区分。

## 未完成

- canonical `ReplanProposal::compute_hash()` 仍为 64-bit DefaultHasher，
  尚未迁移到 v13/Rust 执行路径。
- 多候选生成器、可行集、排序、choice-set certificate、授权绑定、FMI 多候选执行。
- v12→v13 证书影响分析与迁移。

## 候选集实验结果

```text
generated = 2
distinct  = 2
feasible  = 2   (robot-m2 @19800, robot-m3 @20800)
rejected  = 3   (failed m1, tool AND, horizon)
reference admissible set = 2
soundness = true
completeness = true
preferred = robot-m2 (faster)
selected  = robot-m3 -> AUTHORIZED_NON_OPTIMAL_CHOICE
```

Choice-set certificate、bounded optimality、stale/swap/substitution 负例均通过；
尚无 native canonical/FMI 执行 selected candidate。

### Proposal v13 canonical integration

- `ReplanProposal` schema `poie.replan-proposal/v13`，SHA-256 内容承诺。
- 字段：proposal_id、execution_id、checkpoint_hash、group_id、unfinished_task_ids、task_robots、constraints、choice_set_certificate_hash、selected_candidate_hash。
- Authorization v2 签名绑定 proposal_hash、choice_set_certificate_hash、selected_candidate_hash。
- resume 强制 execution/checkpoint/group/unfinished-task/candidate 绑定。
- 测试：授权后修改 candidate、checkpoint、execution id 全部拒绝；合法 proposal 执行完成。
- v12 41 证书字节冻结；v13 41 个 `AUTHORIZATION_SEMANTIC_CORRECTION` successors 签发并通过 verifier/tamper。
