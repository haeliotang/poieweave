# S2 claim → canonical experiment → certificate 覆盖

> 机器生成。`NOT_CERTIFIED` 是显式缺口，不是已认证声明。

## 汇总

```text
claims              53
有 canonical 实验   53
旧 claim 被新 claim 取代 13
CANONICAL_ISSUED    36
REISSUED            5
SUPERSEDED          4
SUPERSEDED_BY_CLAIM 8
```

## 逐项

| claim | canonical experiment | certificate | status |
| --- | --- | --- | --- |
| C5 → C56 | s2-canonical-c5 | — | SUPERSEDED_BY_CLAIM |
| C6 → C57 | s2-canonical-c6 | certificate-s2-joint | REISSUED |
| C7 → C58 | s2-canonical-c7 | — | SUPERSEDED_BY_CLAIM |
| C8 → C59 | s2-canonical-c8 | — | SUPERSEDED_BY_CLAIM |
| C9 → C60 | s2-canonical-c9 | — | SUPERSEDED_BY_CLAIM |
| C10 → C61 | s2-canonical-c10 | — | SUPERSEDED_BY_CLAIM |
| C11 → C62 | s2-canonical-c11 | — | SUPERSEDED_BY_CLAIM |
| C12 → C63 | s2-canonical-c12 | certificate-s2-time | SUPERSEDED |
| C13 → C64 | s2-canonical-c13 | certificate-s2-time | SUPERSEDED |
| C14 → C65 | s2-canonical-c14 | — | SUPERSEDED_BY_CLAIM |
| C18 → C66 | s2-canonical-c18 | certificate-s2-time | SUPERSEDED |
| C19 → C67 | s2-canonical-c19 | certificate-s2-time | SUPERSEDED |
| C56 | s2-canonical-c5 | canonical-c56 | CANONICAL_ISSUED |
| C57 | s2-canonical-c6 | canonical-c57 | CANONICAL_ISSUED |
| C58 | s2-canonical-c7 | canonical-c58 | CANONICAL_ISSUED |
| C59 | s2-canonical-c8 | canonical-c59 | CANONICAL_ISSUED |
| C60 | s2-canonical-c9 | canonical-c60 | CANONICAL_ISSUED |
| C61 | s2-canonical-c10 | canonical-c61 | CANONICAL_ISSUED |
| C62 | s2-canonical-c11 | canonical-c62 | CANONICAL_ISSUED |
| C63 | s2-canonical-c12 | canonical-c63 | CANONICAL_ISSUED |
| C64 | s2-canonical-c13 | canonical-c64 | CANONICAL_ISSUED |
| C65 | s2-canonical-c14 | canonical-c65 | CANONICAL_ISSUED |
| C66 | s2-canonical-c18 | canonical-c66 | CANONICAL_ISSUED |
| C67 | s2-canonical-c19 | canonical-c67 | CANONICAL_ISSUED |
| C68 → C73 | s2-canonical-c68-monte-carlo | — | SUPERSEDED_BY_CLAIM |
| C69 | s2-canonical-load-base05 | certificate-s2-load | REISSUED |
| C70 | s2-canonical-regen-horizon | certificate-s2-regeneration | REISSUED |
| C71 | s2-canonical-metrics-groups | certificate-s2-metrics | REISSUED |
| C72 | s2-canonical-order | certificate-s2-order | REISSUED |
| C73 | s2-canonical-c68-monte-carlo | canonical-c73 | CANONICAL_ISSUED |
| C74 | s2-canonical-c12-components | canonical-c74 | CANONICAL_ISSUED |
| C75 | s2-canonical-energy-interventions | canonical-c75 | CANONICAL_ISSUED |
| C76 | s2-canonical-redundancy-restorers | canonical-c76 | CANONICAL_ISSUED |
| C77 | s2-canonical-aux-batch1 | canonical-c77 | CANONICAL_ISSUED |
| C78 | s2-canonical-aux-batch2 | canonical-c78 | CANONICAL_ISSUED |
| C79 | s2-canonical-aux-batch3 | canonical-c79 | CANONICAL_ISSUED |
| C80 | s2-canonical-aux-batch4 | canonical-c80 | CANONICAL_ISSUED |
| C81 | s2-canonical-aux-batch5 | canonical-c81 | CANONICAL_ISSUED |
| C82 | s2-canonical-aux-batch6 | canonical-c82 | CANONICAL_ISSUED |
| C83 | s2-canonical-aux-batch7 | canonical-c83 | CANONICAL_ISSUED |
| C84 | s2-canonical-aux-batch8 | canonical-c84 | CANONICAL_ISSUED |
| C85 | s2-canonical-aux-batch9 | canonical-c85 | CANONICAL_ISSUED |
| C86 | s2-canonical-aux-batch10 | canonical-c86 | CANONICAL_ISSUED |
| C87 | s2-canonical-aux-batch11 | canonical-c87 | CANONICAL_ISSUED |
| C88 | s2-canonical-aux-batch12 | canonical-c88 | CANONICAL_ISSUED |
| C89 | s2-canonical-aux-batch13 | canonical-c89 | CANONICAL_ISSUED |
| C90 | s2-canonical-aux-batch14 | canonical-c90 | CANONICAL_ISSUED |
| C91 | s2-canonical-aux-batch15 | canonical-c91 | CANONICAL_ISSUED |
| C92 | s2-canonical-aux-batch16 | canonical-c92 | CANONICAL_ISSUED |
| C93 | s2-canonical-aux-batch17 | canonical-c93 | CANONICAL_ISSUED |
| C94 | s2-canonical-aux-batch18 | canonical-c94 | CANONICAL_ISSUED |
| C95 | s2-canonical-aux-batch19 | canonical-c95 | CANONICAL_ISSUED |
| C96 | s2-canonical-aux-batch20 | canonical-c96 | CANONICAL_ISSUED |

## 解释

- `REISSUED`：已有 canonical 证书。
- `SUPERSEDED`：历史证书有效于原始语义，但已被 canonical 语义取代。
- `NOT_CERTIFIED`：有 canonical 实验，但尚未签发新证书；不得当作已认证。
- `SUPERSEDED_BY_CLAIM`：历史 claim 已由带 canonical 实验的新 claim 取代；新 claim 的证书状态见对应行。
- 本表不改变既有证书，也不把辅助覆盖自动升级为正式认证。
