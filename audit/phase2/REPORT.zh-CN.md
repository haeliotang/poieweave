# S5-V Phase 2 — Conformance and Comparator Closure

- Gate 2: `PASS_SCOPED_EXECUTION_AND_ENDPOINTS`
- external execution: `ESTABLISHED (SCOPED)`
- endpoint agreement: `ESTABLISHED (SINGLE ENERGY CASE)`
- trajectory-level cross-model validity: `NOT ESTABLISHED`
- external FMU: `external/fmu-pv-battery/poie-pv-battery.fmu` (FMI 3.0)
- FMU SHA-256: `78032dbf275bfd08cf761fbb17006cbc55d3cb47ba4036cd854f26015c919797`
- packaging: FMI 3.0 platform tuple directories with modelIdentifier filename
- independent importer: fmpy 0.3.32, rows=80, max error vs C harness=0.0 J, CSV SHA=1ed5ce31ee805744f60dc7c76d4134005c6f299b2d186a9cdf97375d29dc6cfe
- analytic-reference max error: `0.0` J
- canonical-exported boundary max error: `0.0` J
- initial-condition max error: `0.0` J
- terminal-energy constraint max error: `0.0` J
- tampered FMU max error: `28440000.0` J, detected=True

## What is compared

The FMU is executed through the C harness and through the independent
fmpy 0.3.32 importer. The C-harness trajectory is compared with an
independent analytic reference that uses the same uniform repair-draw
assumption as the FMU. Canonical is checked only on evidence it actually
exports: the repair-completion record and the depletion time. Initial
energy is a configured initial condition, and terminal energy zero is a
constraint; neither is presented as a canonical runtime export.

## Repair-window model difference

The canonical engine commits repair energy when the repair completes; the
external FMU spreads the same energy over the repair window. Canonical
interior repair-window states are not exported, so no trajectory error is
claimed there. This is registered as a model-assumption difference, not
hidden by tolerance changes:

```text
{
  "status": "MODEL_ASSUMPTION_DIFFERENCE_NOT_COMPARED_AS_TRAJECTORY",
  "fmu": "repair energy spread uniformly over 0..10800 s",
  "canonical": "repair energy committed at repair completion; interior repair states not exported",
  "action": "comparison restricted to t=0, repair completion boundary and depletion; no threshold inflation"
}
```

## Standard packaging

The frozen canonical reference is `audit/phase2/canonical-reference.txt` with
SHA-256 `e4675ad7611341844bcc72066df34dc92c88aef15ac61ac787cbb3ef53b87b5c` and source commit
`7d198d90e729f40faafbe21e4de77311c361dc22`; its SHA is verified before comparison.

The FMU now uses FMI 3.0 platform-tuple directories and modelIdentifier
filenames, e.g. `binaries/aarch64-darwin/poie_pv_battery.dylib` and
`binaries/x86_64-linux/poie_pv_battery.so`. Generic FMI 3.0 importers
require unsupported-interface symbols to be present; the FMU includes
explicit fail-closed stubs for non-Co-Simulation APIs.

## Claims

```text
S5-V EXTERNAL EXECUTION — ESTABLISHED (SCOPED)
S5-V ENDPOINT AGREEMENT — ESTABLISHED (SINGLE ENERGY CASE)
S5-V TRAJECTORY-LEVEL CROSS-MODEL VALIDITY — NOT ESTABLISHED
S5 CROSS-MODEL VALIDITY — NOT ESTABLISHED (full S2)
```

## Limitations

- one parameter set and one energy-only scenario
- no failures beyond the initial repair, no curtailment/capacity saturation
- step sizes 3600/900 s only
- local execution on macOS arm64; committed FMU binary is macOS arm64
- canonical interior repair-window states are not exported
- this does not establish physical-world correctness

## CI evidence binding

The Linux CI job passes its freshly generated fmpy CSV and log path to
`compare.py`; the comparator fails if that CSV is missing. The result
records the current FMU SHA, fmpy CSV/log SHA and platform tuple. The job
then copies the current FMU, harness CSV, fmpy CSV/log, canonical reference
and result into `s5v-linux-evidence` with a SHA256SUMS file and uploads it
as a GitHub Actions artifact.

## Evidence

- config: `audit/phase2/config.json`
- result: `audit/phase2/result.json`
- manifest: `audit/phase2/manifest.json`
- raw CSVs: `audit/phase2/results`
- logs: `audit/phase2/logs`
