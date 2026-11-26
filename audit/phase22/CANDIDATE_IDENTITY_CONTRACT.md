# Candidate Identity Contract

A recovery candidate is identified by a SHA-256 canonical commitment over:
checkpoint_hash, proposal content, unfinished tasks, selected robots, required
tools, dependency paths, schedule, makespan and remaining horizon.
Candidate ID alone is not identity. Candidates differing only in ID are
equivalent and must be deduplicated.
