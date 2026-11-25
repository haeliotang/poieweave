# Resource Contention Contract

Robot, tool and named upstream resources are exclusive while held.
Two tasks may run in parallel only if they share no robot and neither
requires the shared tool. Shared tool-t1 serializes tasks even when robots
differ. Reference v2 excludes illegal parallel candidates; legal A→B and B→A
serial schedules remain.
