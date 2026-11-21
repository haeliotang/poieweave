# Phase Alignment

External dep_before/dep_after is read per native event, not from a single
last-state map. Strict cases align events by normalized (time, kind, target)
order; policy cases validate each model's closure against its own selected
resources. A missing before-state remains BEFORE_NOT_OBSERVED.
