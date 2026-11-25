# One-Shot Authorization Contract

authorization_id, execution_id, checkpoint_hash, proposal_hash, scope,
max_attempts=1. Missing, stale, forged, duplicate or cross-checkpoint
authorization is rejected. N21R-C requires one valid authorization and at
most one resume attempt.
