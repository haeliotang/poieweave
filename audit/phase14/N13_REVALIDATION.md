# N13 Revalidation (N14 Phase 0)

The N14 contract-independent revalidation of all 10 N13 cases found a real
N13 external journal defect: `RepairCompleted`/`RestorationCommitted` recorded
`failed_before` from the previous failure event instead of the current
`failed_mask`. N13-02, N13-03 and N13-04 therefore fail strict state-history
continuity. N13 completion/restoration outcome agreement remains valid, but
the N13 "state history persistence" claim is narrowed.

Also fixed in the N14 validator:
- contract-independent event-time checks (both models changed together is rejected);
- failure_id identity and one-execution checks;
- exact bit-set state conservation, no cleared/unsourced bits;
- independent outcome/restoration preconditions.
