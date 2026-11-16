# S2 Certificate Impact Matrix

This matrix records certificate impact only. No certificate is reissued by this document.

| certificate | model | claims | canonical | classification | status | reissued |
| --- | --- | --- | --- | --- | --- | --- |
| `certificate-s2-time` | `multi-model` | C12, C13, C18, C19 | s2-canonical-c12, s2-canonical-c13, s2-canonical-c18, s2-canonical-c19 | `SEMANTIC_CORRECTION` | `SUPERSEDED` | no |
| `certificate-s2-joint` | `scenarios/base-06/model.toml` | C6 | s2-canonical-c6 | `EXPECTED_AGREEMENT` | `HISTORICAL_VALID_WITH_ORIGINAL_SEMANTICS` | no |
| `certificate-s2-load` | `scenarios/base-05/model.toml` | C8 | s2-canonical-c8 | `EXPECTED_AGREEMENT` | `HISTORICAL_VALID_WITH_ORIGINAL_SEMANTICS` | no |
| `certificate-s2-regeneration` | `multi-model` | C7 | s2-canonical-c7 | `EXPECTED_AGREEMENT` | `HISTORICAL_VALID_WITH_ORIGINAL_SEMANTICS` | no |
| `certificate-s2-metrics` | `multi-model` | NONE | NONE | `NOT_MIGRATED` | `BLOCKED` | no |
| `certificate-s2-order` | `scenarios/base-06/model.toml` | NONE | NONE | `NOT_MIGRATED` | `BLOCKED` | no |

## Summary

- certificates: 6
- historical valid with original semantics: 3
- superseded: 1
- invalidated: 0
- reissued: 0
- blocked: 2

## Rationale

### `certificate-s2-time`

canonical t=0 ordering and in-repair failure timing change grease/random-window values

- scope: time semantics and boundaries

### `certificate-s2-joint`

canonical reproduces 25/56 independent and 24/56 joint after the non-service repair fix

- scope: Base-06 joint coverage
- model sha256: `09c637fc4bc3bb8fa7b493d1718e153d5463a18f1b85e96f4e493f6155e4e95c`

### `certificate-s2-load`

canonical reproduces all 11 load rows including 2.6 kW switch and final energy

- scope: Base-08 load threshold
- model sha256: `83b96a0293fb4bf5cc306158bd26bdd15899a8c09251ab746c2b544d5ca57150`

### `certificate-s2-regeneration`

canonical reproduces 6/6 rows including energy and failure times

- scope: insert sharpening 40/50->60

### `certificate-s2-metrics`

contains multiple numeric metrics with no single canonical migration; must be split before reissue

- scope: cross-model metric definitions and values

### `certificate-s2-order`

no formal claim in the claims ledger and no canonical order adapter; do not reissue

- scope: Base-06 repair-order sensitivity
- model sha256: `09c637fc4bc3bb8fa7b493d1718e153d5463a18f1b85e96f4e493f6155e4e95c`

