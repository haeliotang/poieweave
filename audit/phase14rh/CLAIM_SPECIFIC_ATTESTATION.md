# Claim-Specific v7 Attestation

For every v7 claim:

- v7 certificate SHA and predecessor v6 SHA;
- independent historical-v6 replay record;
- current-v7 replay record (runner, expected, actual output);
- legacy expected unchanged flag;
- classification `CLAIM_SPECIFIC_EXPERIMENT_REPLAY`.

The global `check-experiment-replays.py` output is **not** used as per-claim
execution evidence.
