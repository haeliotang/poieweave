# N8-H Comparison Contract

Alignment modes:

```text
EXACT_SAMPLE            actual timestamps identical within 1e-9 s
EVENT_BOUNDARY_SAMPLE   exact event-side sample
EVENT_SAFE_INTERPOLATION not used in this slice
NOT_COMPARABLE          no exact sample / after termination / phase mismatch
```

No nearest-neighbor fallback. No interpolation across events or termination.

Observation validity:

```text
OBSERVED
UNOBSERVABLE_AFTER_TERMINATION
UNOBSERVABLE_NOT_MATERIALIZED
```

Only `OBSERVED` energy values enter residual statistics. Excluded samples remain in
the audit list with reasons.

Energy decomposition:

```text
raw = external - canonical
explained = model_assumption_delta
unexplained = raw - explained
```

Registered assumption: canonical commits repair energy at completion; FMU draws it
uniformly over 10800 s. For N8-04 the repair is interrupted at 7200 s, so only
pre-interruption partial draw is compared.

Conservation is checked separately per model and per sample.
