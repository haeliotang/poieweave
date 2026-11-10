# Poieweave licensing

Poieweave uses a layered licence model so that applications and runtime code
stay open under AGPL-3.0-or-later, while the contracts and data needed to
implement or interoperate with the project remain permissively licensed.

## Layers

| Path | SPDX licence | Why |
| --- | --- | --- |
| `apps/desktop/` | `AGPL-3.0-or-later` | Desktop application and host; runtime behaviour, not a contract. |
| `crates/poie-delivery/`, `poie-exec/`, `poie-expansion/`, `poie-federation/`, `poie-governance/`, `poie-plan/`, `poie-process/`, `poie-runtime/`, `poie-sim/`, `poie-state/`, `poie-viability/`, `poie-whatif/` | `AGPL-3.0-or-later` | Runtime, engines, services and internal execution state. |
| `crates/poie-ontology/` | `Apache-2.0` | Normative model types and capability-graph contract types; third parties need to implement them. |
| `crates/poie-objects/` | `Apache-2.0` | Stable object/link/action DTOs and generated SDK surface for third-party clients. |
| `crates/poie-digest/` | `Apache-2.0` | Canonical digest contract used by independent implementations. |
| `crates/poie-sdkgen/` | `Apache-2.0` | Generates the public TypeScript SDK from the contract catalog. |
| `python/poie_models/` | `AGPL-3.0-or-later` | Reference process-model runtime, not a contract twin. |
| `scripts/` | `AGPL-3.0-or-later` | Build, release and verification tooling. |
| `scenarios/` | `Apache-2.0` | Pre-registered test models and cases; independent implementations need them. |
| `fixtures/` | `Apache-2.0` | Frozen interchange fixtures and test vectors. |
| `evals/` | `Apache-2.0` | Frozen agent-evaluation fixtures. |
| `experiments/**/runner/`, `experiments/**/baseline/`, `experiments/**/*.sh`, `experiments/**/Dockerfile`, `experiments/**/Dockerfile.dockerignore` | `AGPL-3.0-or-later` | Experiment runners, baselines and orchestration containers are executable tooling. |
| `experiments/` (results, raw data, reports) | `Apache-2.0` | Frozen experiment results and test data; evidence must remain reusable. |
| `apps/desktop/src/sdk/` | `Apache-2.0` | Generated public TypeScript SDK; no SPDX header so `poie-sdkgen --check` stays byte-stable. |
| `docs/` | `CC-BY-4.0` | Documentation, design notes, status and research text. |
| `specifications/` | `CC-BY-4.0` | Normative contract prose; byte-frozen for this change. |
| `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, `NOTICE`, `LICENSING.md`, `CLA.md` | `CC-BY-4.0` | Project documentation and contribution terms. |
| `.github/` | `AGPL-3.0-or-later` | CI, release and issue infrastructure. |
| `apps/desktop/screenshots/`, `docs/status/assets/` | `CC-BY-4.0` | Documentation screenshots. |
| `apps/desktop/src-tauri/icons/`, `apps/desktop/src/assets/poieweave-mono.svg` | `LicenseRef-Inceptless-Brand` | Project logos and application icons. |
| `LICENSES/` | Their own texts | Official licence texts; see below. |

`LICENSES/LicenseRef-Inceptless-Brand.txt` states the brand boundary. The name
and marks may be used to refer to this project, but not to imply official
status, sponsorship or endorsement. Questions: support@inceptless.com.

## Why AGPL for the application and runtime

AGPL-3.0-or-later keeps modifications to the runtime and any network-facing
service open, including when the service is operated for remote users. The
contract crates and test data remain Apache-2.0 so that a closed product can
implement the same contracts and interoperate without adopting this codebase.

## Commercial licensing

Inceptless may offer the project under terms other than AGPL-3.0-or-later. If
your organisation needs different terms, write to support@inceptless.com.

## Licence history

| Versions or commits | Licence |
| --- | --- |
| `v0.4.19` and every earlier tag or release | Apache-2.0 |
| From merge commit `5d47bb3` onward (source `0.4.20`, unreleased) | Layered model in this file |

Published versions are not retroactively relicensed. A tag, release asset or
commit that was published under Apache-2.0 remains available under Apache-2.0.

## Third-party components

Third-party dependencies keep their own licences. They are declared in the
lockfiles and manifests; redistributors must preserve their notices. The
`cargo-deny` policy allows the licences compatible with this model and rejects
unreviewed copyleft or proprietary terms.

## Official texts

| Licence | Source | SHA-256 |
| --- | --- | --- |
| AGPL-3.0-or-later | https://www.gnu.org/licenses/agpl-3.0.txt | `0d96a4ff68ad6d4b6f1f30f713b18d5184912ba8dd389f86aa7710db079abcb0` |
| Apache-2.0 | https://raw.githubusercontent.com/spdx/license-list-data/main/text/Apache-2.0.txt | `074e6e32c86a4c0ef8b3ed25b721ca23aca83df277cd88106ef7177c354615ff` |
| CC-BY-4.0 | https://raw.githubusercontent.com/spdx/license-list-data/main/text/CC-BY-4.0.txt | `d557539df68e771cc1eedcc91d13f70fca930e508d11eedcafa4b15db49e3744` |

`LICENSE` is the verbatim AGPL-3.0 text. `LICENSES/` contains the REUSE copies
of every licence used in this repository.
