# Authorized Selection Contract

Authorization binds checkpoint_hash, choice_set_certificate_hash,
selected_candidate_hash and authorized_candidate_hash. Execution must reject
stale choice set, checkpoint swap, candidate not in set, and authorization for
a different candidate. One selected candidate, one continuation attempt.
