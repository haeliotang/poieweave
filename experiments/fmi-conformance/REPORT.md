# FMI 3.0 Conformance — report

- Date: 2026-11-15
- Status: description/container checks pass; co-simulation not run
- Runner: `python3 experiments/fmi-conformance/validate.py`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
fmi-version=3.0
co-simulation=declared
variables=2
container=zip
fmi-model-description=ok
fmi-cosimulation=not_run
```

## 2. Findings

- A minimal FMI 3.0 description with `CoSimulation` and two variables passes
  attribute, causality, variability and uniqueness checks.
- The description can be packaged and re-read as an FMU ZIP container.
- No FMU binary is loaded; `fmi-cosimulation=not_run` is mandatory output.

## 3. Significance

- This is the container/description half of S5's external-model track.
- It deliberately does **not** claim that an external model challenged existing
  survival results; that requires a working FMU and a stepping loop.

## 4. Limits

- No C ABI, no numerical integration, no unit or stability checks.
