# S2 Certificate Impact Matrix

This matrix records certificate impact only. No certificate is reissued by this document.

| certificate | model | claims | canonical | classification | status | reissued |
| --- | --- | --- | --- | --- | --- | --- |
| `certificate-s2-time` | `multi-model` | C12, C13, C18, C19 | s2-canonical-c12, s2-canonical-c13, s2-canonical-c18, s2-canonical-c19 | `SEMANTIC_CORRECTION` | `SUPERSEDED` | no |
| `certificate-s2-joint` | `scenarios/base-06/model.toml` | C6 | s2-canonical-c6 | `EXPECTED_AGREEMENT` | `REISSUED` | yes |
| `certificate-s2-load` | `scenarios/base-05/model.toml` | C69 | s2-canonical-load-base05 | `EXPECTED_AGREEMENT` | `REISSUED` | yes |
| `certificate-s2-regeneration` | `multi-model` | C70 | s2-canonical-regen-horizon | `EXPECTED_AGREEMENT` | `REISSUED` | yes |
| `certificate-s2-metrics` | `multi-model` | C71 | s2-canonical-metrics-groups | `MIXED_AGREEMENT_AND_DEFINITION_CHANGE` | `REISSUED` | yes |
| `certificate-s2-order` | `scenarios/base-06/model.toml` | C72 | s2-canonical-order | `EXPECTED_AGREEMENT` | `REISSUED` | yes |

## Summary

- certificates: 6
- historical valid with original semantics: 0
- superseded: 1
- invalidated: 0
- reissued: 5
- blocked: 0

## Rationale

### `certificate-s2-time`

canonical t=0 ordering and in-repair failure timing change grease/random-window values

- scope: time semantics and boundaries

### `certificate-s2-joint`

canonical reproduces 25/56 independent and 24/56 joint after the non-service repair fix; canonical certificate issued

- scope: Base-06 joint coverage
- model sha256: `09c637fc4bc3bb8fa7b493d1718e153d5463a18f1b85e96f4e493f6155e4e95c`
- canonical certificate: `experiments/certificate-canonical-c6-v4/certificate.json`

### `certificate-s2-load`

canonical reproduces all five Base-05 load rows with final energy and termination; canonical certificate issued

- scope: Base-05 load 0-4 kW
- model sha256: `83b96a0293fb4bf5cc306158bd26bdd15899a8c09251ab746c2b544d5ca57150`
- canonical certificate: `experiments/certificate-canonical-c69-v4/certificate.json`

### `certificate-s2-regeneration`

canonical reproduces 8/8 rows including final energy and termination; canonical certificate issued

- scope: Base-05/08 regeneration intervals
- canonical certificate: `experiments/certificate-canonical-c70-v4/certificate.json`

### `certificate-s2-metrics`

scoped v4 certificate: 10 canonical groups, coverage definition service-impact-v1, 2 partial and 1 superseded group explicitly excluded; historical certificate preserved

- scope: cross-model metric definitions and values
- canonical certificate: `experiments/certificate-canonical-c71-v4/certificate.json`

### `certificate-s2-order`

canonical permutation search reproduces 24/25 and 40/44 with order_sensitive=0; canonical certificate issued

- scope: Base-06 repair-order sensitivity
- model sha256: `09c637fc4bc3bb8fa7b493d1718e153d5463a18f1b85e96f4e493f6155e4e95c`
- canonical certificate: `experiments/certificate-canonical-c72-v4/certificate.json`

