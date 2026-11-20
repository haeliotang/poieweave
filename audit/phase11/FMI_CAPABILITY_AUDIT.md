# FMI Capability Audit

Current N11 FMU declares Co-Simulation without Event Mode
(`hasEventMode="false"`), matching the implementation. Discrete events are
handled inside `fmi3DoStep`; no Event Mode state machine is claimed.

Journal transport uses a standard FMI 3.0 String output. fmpy and the C harness
both read the same String variable. This is a portable scalar-output design;
no custom C API is exposed.

Known limits:

- journal capacity 32
- no Event Mode
- no Early Return
- journal is a snapshot of the current run, not externally persisted by FMI
