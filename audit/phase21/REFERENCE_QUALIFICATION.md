# Reference Oracle Qualification

- v1 shared-tool false pass reproduced and corrected in reference_v2.
- Raw-evidence validator `validate_execution_bundle` computes schedule
  admissibility, contention, tool AND, event order, closure, completion and
  provenance from raw native evidence; it ignores caller `valid=True` flags.
- Reference mutation suite covers 10 mutations, each passing the official
  validator: event order, shared tool, shared/future dependency loss, tool
  missing, premature commit, forged schedule, forged evidence flag,
  counterfactual direction.
- Canonical and external shared-tool witness both classify VALID under v2.
- N20 64 accepted inputs revalidated: 64 accepted, 0 rejected.
