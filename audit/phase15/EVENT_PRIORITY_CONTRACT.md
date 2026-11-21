# Event Priority

At one timestamp:

1. TaskCompleted for tasks ending at that time;
2. ResourceReleased for resources no unfinished task needs;
3. AssetFailure batch;
4. RepairInterrupted if the failure hits an unfinished task dependency;
5. group RepairCompleted only at group makespan.

At 7200: TaskCompleted A and ResourceReleased robot-m1 precede a 7200 failure.
At 10800: TaskCompleted B, ResourceReleased robot-m2, then group completion.
