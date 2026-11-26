# N21-R Revalidation

N21-R claimed ESTABLISHED (SCOPED) with 15/18 PASS, 3 NOT_REPRESENTABLE.
N21-RH audits three gaps that N21-R did not establish:

1. `compute_state_hash` does not commit all recovery-relevant fields and uses
   64-bit `DefaultHasher`, not a cryptographic digest.
2. Authorization consumption is process-local only; cross-process replay is
   not prevented.
3. `cross_model_fmi.py` contains permissive `or` comparisons and fixed
   fallback values; reducer is fixed-scenario, not automatic reduction.

Those claims are downgraded to their actual scope and addressed here.
