# Reference Independence

`reference_oracle/reference.py` imports only standard library modules and
defines its own durations, upstream map, scheduling enumeration and failure
semantics. It does not invoke canonical engine, scheduler, FMU harness,
`model_outcome()` or `R.classify()`.

Reference predictions are written before the four model runs and hashed.
