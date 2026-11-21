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
hashes
78498b0757c2277469650a9a58f6dffde3ab7999a1ad3c5cfe009d7ef8b69d0f  audit/phase13/PREREG.md
f1f8c41401822cff1645e835a8d2f3ec29556d2383d60a2d6ec24833d7209c48  audit/phase13/CLAIMS.json
5d98d6618ebbf01a8c3410a963e71ca72f9a6745ade45095350cb36ee8f10ad5  audit/phase13/REPORT.zh-CN.md
N13 PREREG/CLAIMS updated as versioned errata; N13 manifest re-frozen.
