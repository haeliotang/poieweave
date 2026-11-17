# S5-V N9 — Independent Resource-Causal Trajectory Challenge

Baseline: `60e2150a5a6e5e5f7cb9fa06e68bffb5e71b1c71`.

## Research question

Can two independently implemented executable models preserve causally valid
resource dependencies and repair commitments under the same task contracts and
failure events, and can an independent validator locate the first resource-level
causal divergence?

## Gate N9-0

Close the remaining N8-H comparator mutation gaps, keep N8-H positive controls
passing, and separate evidence-integrity verification from numerical comparison.

## Planned cases

| Case | Scenario | Primary challenge |
| --- | --- | --- |
| N9-00 | single task no failure | acquire/hold/complete |
| N9-01 | two tasks one robot | serialization |
| N9-02 | two tasks two robots | parallel + group commit |
| N9-03 | selected robot failure 7200 | execution dependency interruption |
| N9-04 | unselected robot failure 7200 | irrelevant failure must not interrupt |
| N9-05 | tool-t1 failure 9000 | required-tool AND dependency |
| N9-06 | bus-1 failure 9000 | upstream dependency |
| N9-07 | tool/device failure 10800 | completion/failure boundary |
| N9-08 | failure at horizon | outside-window event |
| N9-09 | robot failure 7199/7200/7201 | group-hold boundary |
| N9-10 | tool failure 12000 | post-commit dependency |
| N9-11 | priority/mask variation | independent scheduling validity |

N9-00..N9-06 are the minimum core set. N9-07..N9-11 strengthen boundaries.

## Not claimed

- step-level resource release
- event-engine priority input
- post-interruption replanning
- Level 3 planning
- autonomous recovery
- general resource-causal validity
- physical-world correctness
