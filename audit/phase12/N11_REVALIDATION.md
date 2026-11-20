# N11 Revalidation (N12 Phase 0)

Original N11 report/CLAIMS SHA-256 (from the pre-N12 files):

```text
report: f3f418b40c041f532f0323589a1777bcef1aa007476f5ebdd7b7f60237a0faf6
claims: recorded in the PR history and reproducible from the N11 commit
```

## Findings

1. N11 `CLAIMS.json` left N11-0 `IN_PROGRESS` even though N11-0 work was complete.
2. N11 report retained stale statements that N11-C was not established and that
   canonical before/after blocked C; the final registry marked C established with an
   explicit incomplete-projection scope.
3. The N11 external FMU records every `AssetFailure` with target `robot-m1`.
   For N10-U01 (real failure `bus-1`) the journal is identity-wrong.
4. N11 `validate_native.py` does not compare failure target identity, so it accepts
   the wrong target.
5. N11 `validate_native.py` still contained a `N10-OR2` case-specific branch.

## Corrected interpretation

N11 event time/order agreement remains scoped. Event target identity for affected
historical cases is **not established** until N12 native evidence is produced.

## Preservation

N11 raw journals and N10/N11 frozen evidence are not rewritten. N12 adds independent
revalidation and a new N12 FMU.
