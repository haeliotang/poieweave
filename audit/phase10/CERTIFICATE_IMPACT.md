# N10 Certificate Impact

N10 adds observation-only `RepairObservation.task_resources`, populated from the
actual schedule. This changes engine bytes, so v5 is frozen at `492e20f` and
current certificates migrate to v6 with `TASK_RESOURCE_OBSERVATION_ONLY_SUCCESSOR`.
No execution ordering, resource-hold, interruption or restoration rule changes.
