# Model Mapping

Canonical `execution_dependencies` after each event is projected to named
upstream assets; canonical upstream `ResourceReleased` is normalized to
`DependencyDeactivated`.

N16 FMU computes its dep mask from per-task robot dependency masks and emits
`DependencyDeactivated` events with dep_before/dep_after.

Canonical robot-m2 split profile requires bus-2 and provides a task-B-specific
capability; this is a research input profile, not a canonical semantic change.
