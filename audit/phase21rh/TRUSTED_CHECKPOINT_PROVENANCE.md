# Trusted Provenance

Content hash alone cannot prevent attacker recomputing a modified checkpoint.
N21-RH introduces a separate checkpoint ledger/commitment record; verify
requires the commitment to exist in trusted storage independent of the
checkpoint file.
