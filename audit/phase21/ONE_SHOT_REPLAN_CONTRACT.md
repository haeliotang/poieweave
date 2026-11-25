# One-Shot Replan Contract (not implemented)

State machine: RUNNING -> INTERRUPTED -> CHECKPOINTED -> REPLAN_PROPOSED ->
REPLAN_VALIDATED -> AUTHORIZED -> RESUMED -> COMPLETED/BLOCKED.
No INTERRUPTED -> RESUMED without authorization; max_attempts=1.
