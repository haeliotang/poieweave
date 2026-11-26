# Checkpoint Integrity Contract

Schema `poie.repair-checkpoint/v12` uses deterministic canonical encoding and
SHA-256 domain-separated commitment over all recovery-relevant fields.
`state_hash` is not recursively included. NaN/Infinity are rejected and -0.0
is normalized. Content integrity is distinct from trusted provenance.
