# S5-V N22 — Recovery Choice-Set Independence & Decision Validity

基线：`6fa4e047`。

```text
N22-0 HISTORY/PROPOSAL        IN PROGRESS — N21RH-0 supplement + v13 reference commitment
N22-A CANDIDATE GENERATION    NOT ESTABLISHED
N22-B DECISION VALIDITY       NOT ESTABLISHED
N22-C CHOICE-SET INTEGRITY    NOT ESTABLISHED
N22-D AUTHORIZED EXECUTION    NOT ESTABLISHED
N22-E CROSS-MODEL             NOT ESTABLISHED
N22-F ADVERSARIAL/CERT        NOT ESTABLISHED
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
