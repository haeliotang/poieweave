# N24 Revalidation

## Confirmed N24 defects

1. **Premature restoration state.** In the N24 two-task trajectory,
   `TaskCompleted C @12600` recorded `a-lathe = available` while B was
   unfinished until 19800 and `RestorationCommitted` was only at 19800.
   See `failing-witness/PREMATURE_RESTORATION_STATE/witness.json`.
2. **Post-hoc full schedule authorization.** N24 verified the full schedule
   only in Python after execution; canonical did not enforce it before resume.
3. **Hard-coded matrix PASS.** `audit/phase24/fixed-cases/run_matrix.py` assigns
   `True` for several cases (N24-06/08/09/11/13/16/17/18/19/20/21) without
   canonical/FMI execution.
4. **Dictionary-only mutants.** Several N24 mutants mutated Python schedule
   dictionaries without any native execution.
5. **State label conflict.** N24 REPORT claimed ESTABLISHED (SCOPED) while the
   certificate limitations still described partial enforcement.

These are recorded as errata; original N24 files and certificates remain frozen.
