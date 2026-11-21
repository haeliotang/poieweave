# Importer Independence

Same N14 FMU binary and same `N14R-00..09` inputs are executed twice:

- C harness (`external/fmu-n14-scheduler/build/harness`)
- fmpy 0.3.32 through `FMU3Slave`

Compared: all harness CSV output fields, final numeric values, and complete
native journal events. Discrete events are compared exactly; numeric values use
1e-9 tolerance.
