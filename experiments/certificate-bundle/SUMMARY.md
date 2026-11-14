# Viability Certificate Bundle — Summary

- Bundle schema: `poie.certificate-bundle/1`
- Claim scope: model/simulation certificates only; no physical safety or authority claim

| Layer | Schema | SHA-256 (prefix) | Key result |
| --- | --- | --- | --- |
| S1 | `poie.closure-certificate/1` | `2dddca964d33c747` | RESTORABLE lathe-1 (7200.0s) |
| S2 | `poie.viability-certificate/1` | `7608e4f828cac760` | 5 load rows; 0-2 kW material-limited, 3-4 kW energy-limited |
| S2-joint | `poie.coverage-certificate/1` | `c11ed96a3a3df05c` | independent 25 vs joint 24; gap 1 |
| S2-regen | `poie.regeneration-certificate/1` | `c0aed8992e7dea7a` | 40.0d → 60.0d |
| S2-metrics | `poie.metrics-certificate/1` | `9619e9aca53ce82c` |  |
| S2-order | `poie.order-certificate/1` | `36bde32ac53520e4` |  |
| S2-time | `poie.time-certificate/1` | `bdffb0243778a485` |  |
| S3 | `poie.recovery-certificate/1` | `a006ed1f57273efb` | 2 receipts; approved 8 h lease |
| S3-multi | `poie.multi-recovery-certificate/1` | `a399132eb44e7e7e` |  |
| S4 | `poie.s4-certificate/1` | `0413d9a9c7c47d09` | cases=112 matched=112 mismatched=0; plans=26 verified=26 |
| S4-approval | `poie.approval-certificate/1` | `74b214e10a9dcf8b` |  |
| S3-concurrency | `poie.concurrency-certificate/1` | `d95134d6c2f170e8` |  |
| S4-independent | `poie.independent-certificate/1` | `cf34ff2e56f86ab2` |  |

> Model-level and isolated-simulation evidence only. This summary is not a
> physical safety, lifetime, authority or industrial-autonomy certificate.

