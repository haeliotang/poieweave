# Reference Independence Audit

`candidate_generator/generator.py` and `reference-oracle/reference.py` import
only standard library and the proposal commitment module. They do not call
canonical engine, canonical scheduler, resume runner, FMU, or canonical
outcome labels. Inputs are the formal contract and verified checkpoint fields.
