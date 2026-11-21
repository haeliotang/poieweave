# Independence

Canonical: explicit repair group API reusing the canonical event loop, scheduler,
closure and atomic commit.

N15 FMU: independent C99 implementation that computes task schedule, task
completion, release mask, dep mask and interruption itself; it does not read
canonical release times or dependencies.

Comparator: independent contract checks per model plus cross-model event/time/
release comparison.
