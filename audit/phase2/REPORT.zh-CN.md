# S5-V Phase 2 — Executable External Model Challenge

- Gate 2: `PASS`
- external FMU: `external/fmu-pv-battery/poie-pv-battery.fmu` (FMI 3.0)
- FMU SHA-256: `d12072af690746dfabed67b26c6c01b03988bc2ed52aead53a086fe9ec7da24d`
- independent model: C99 PV + battery + load + initial repair draw
- depletion time: `284400.0` s
- coarse and fine max absolute error: `0.0` J
- tampered FMU max absolute error: `35550000.0` J
- tampered detector: `True`

## Scope

This is a single reduced energy-subsystem challenge. It is not a physical
validation and not a full S2 cross-model validation.

## Pre-registered case

- PV 2500 W, load 3000 W, battery 200 kWh / 10 kW charge
- initial energy 40 kWh, initial repair draw 0.5 kWh over 3 h
- expected depletion 284400 s (79 h)
- coarse step 3600 s, fine step 900 s
- energy relative tolerance 1e-9, absolute 1 J, conservation 1e-3 J

## Independent implementation

The FMU is hand-written C99, packaged as FMI 3.0 Co-Simulation, and does not
link or call the Poieweave Rust engine. The harness unpacks the FMU, dlopens
the native model, calls `fmi3InstantiateCoSimulation`, initialization, variable
exchange and `fmi3DoStep`, then terminates the instance.

## Comparison result

The FMU output is compared every communication step against an independent
closed-form reference. Energy conservation, battery capacity, depletion time,
cumulative generated/load/repair energy and step sensitivity all pass.
The tampered FMU (+100 W PV) is detected by the same comparator.

## Limitations

- one parameter set and one energy-only scenario
- no failures beyond the initial repair, no curtailment/capacity saturation
- step sizes 3600/900 s only
- executed on the local macOS arm64 host; source and build script support Linux
- this does not establish physical-world correctness

## Claim status

`S5 CROSS-MODEL VALIDITY — ESTABLISHED (SCOPED: PV/battery/load energy boundary)`

The broader S2/model-wide cross-model validity remains NOT ESTABLISHED.

## Evidence

- config: `audit/phase2/config.json`
- result: `audit/phase2/result.json`
- manifest: `audit/phase2/manifest.json`
- raw CSVs: `audit/phase2/results`
- logs: `audit/phase2/logs`
