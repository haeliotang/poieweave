# N13 Metamorphic Properties

- Independent failure permutation: N13-02 vs N13-03 have the same final failed
  mask and both complete.
- Irrelevant failure invariance: N13-00 and N13-02 both complete at 7200.
- Dependency monotonicity: adding unrelated bus-2 before active bus-1 still
  interrupts (N13-07/N13-08).
- Step invariance: N13-02 native journal identical at 1/900/1000 s.
- Real overflow: capacity-3 FMU truncates at 3 with overflow=true.
- Generated cases: fixed seed 20261121, 12 cases, zero property failures.
