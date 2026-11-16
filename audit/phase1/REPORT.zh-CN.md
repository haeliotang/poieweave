# S5-V Phase 1 — Independent Certificate Audit

- baseline requested: `acdef3ffb52fd255c5c33567ee6cb94a3e0a60c1`
- baseline actual HEAD: `acdef3ffb52fd255c5c33567ee6cb94a3e0a60c1`
- selected sample: 12 / 36 certificates
- all-certificate hash checks: 36 checked, 0 failures
- Gate 1: `PASS_FOR_EXTERNAL_MODEL_PHASE`

## Method

The audit harness is independent of the repository certificate generator and verifier:

- re-executes each fixed-commit runner and compares stdout byte-for-byte with `expected.txt`
- recomputes all certificate SHA-256 bindings with a separate implementation
- parses output rows and checks claim arithmetic/constraints directly
- runs four negative certificate mutations (expected, engine, claim, model)
- records raw runner output under `audit/phase1/evidence/` with SHA-256

No historical expected output or certificate was modified.

## Status counts

```text
VERIFIED                   9
VERIFIED_WITH_LIMITATIONS  3
```

## Claim matrix

| claim | certificate | runner | status | first divergence |
| --- | --- | --- | --- | --- |
| C56 | `certificate-canonical-c56/certificate.json` | `poie-migration-c5` | VERIFIED | none: legacy/canonical days, energy and failure times agree in all 4 rows |
| C59 | `certificate-canonical-c59/certificate.json` | `poie-migration-c8` | VERIFIED | none: legacy/canonical days and energy agree in all 11 rows |
| C62 | `certificate-canonical-c62/certificate.json` | `poie-migration-c11` | VERIFIED | t=0 canonical minimum 0.125 d vs legacy 0; day 10 legacy ignores at_day and reports 0 vs canonical 10 d |
| C63 | `certificate-canonical-c63/certificate.json` | `poie-migration-c12` | VERIFIED | grease 0.10: instant 4 d, event-driven 10 d, canonical 2 d; canonical terminates as repair interrupted at 172800 s |
| C64 | `certificate-canonical-c64/certificate.json` | `poie-migration-c13` | VERIFIED | load 3.0: event-driven 3.292 d vs canonical 3.333 d (repair schedule); day granularity unchanged |
| C65 | `certificate-canonical-c65/certificate.json` | `poie-migration-c14` | VERIFIED | baseline: instant 12 d, event-driven 31 d, canonical 2 d; redundant pump 90 d in all time semantics |
| C66 | `certificate-canonical-c66/certificate.json` | `poie-migration-c18` | VERIFIED_WITH_LIMITATIONS | aggregate: legacy zero 129 -> canonical 0; mean 1.4 -> 9.9 (event) -> 5.4 (canonical); raw trial stream not exported |
| C67 | `certificate-canonical-c67/certificate.json` | `poie-migration-c19` | VERIFIED_WITH_LIMITATIONS | aggregate: legacy zero 13 -> canonical 0; mean 4.4 -> 14.0 (event) -> 2.2 (canonical); raw trial stream not exported |
| C73 | `certificate-canonical-c73/certificate.json` | `poie-migration-c68` | VERIFIED_WITH_LIMITATIONS | aggregate: median 72 -> 68, mean 64.8 -> 64.0, 90-day survivors 622 -> 611; raw 2000 schedules not exported |
| C81 | `certificate-canonical-c81/certificate.json` | `poie-migration-aux-batch5` | VERIFIED | coverage definition: envelope 5 -> 9 and pairs 11 -> 33; regeneration threshold 60/61 unchanged |
| C94 | `certificate-canonical-c94/certificate.json` | `poie-migration-aux-batch18` | VERIFIED | pump-p1+robot-m1: day 0 legacy 60 -> canonical 0.125; day 10 legacy 60 -> canonical 10; repair executor failed at event time |
| C95 | `certificate-canonical-c95/certificate.json` | `poie-migration-aux-batch19` | VERIFIED | Base-07 joint coverage 25 -> 24 under canonical t=0 repair ordering; independent coverage unchanged |

## Limitations

- **C66**: random aggregate only: per-trial samples are not exported, so the mean/zero counts are reproduced but cannot be recomputed from a raw sample stream
- **C67**: random aggregate only: per-trial samples are not exported, so the mean/zero counts are reproduced but cannot be recomputed from a raw sample stream
- **C73**: Monte Carlo aggregate only: quantiles/mean/buckets are reproduced; the 2000 individual schedules are not exported for independent recomputation

## Negative tests

| mutation | rejected |
| --- | --- |
| expected hash | True |
| engine hash | True |
| claim presence | True |
| model hash | True |

## Gate 1 ruling

`PASS_FOR_EXTERNAL_MODEL_PHASE`

The 12 sampled claims have no unexplained mismatch. Three claims are aggregate-only
and therefore `VERIFIED_WITH_LIMITATIONS`: their aggregate statistics reproduce but
the raw random trial/schedule stream is not exported for independent re-derivation.
The other 24 certificates have only independent hash checks in this phase, not
runner re-execution, and remain a residual audit risk.

Independence here is implementation-level: the harness does not reuse the repository
certificate generator or verifier, but it was executed by the same agent, not by an
independent third-party reviewer.
