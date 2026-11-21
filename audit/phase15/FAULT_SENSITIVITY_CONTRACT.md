# Fault Sensitivity

A failure interrupts only while its target is an active dependency of unfinished
work:

- selected robot of an unfinished task;
- required tool of an unfinished task;
- named upstream dependency of an unfinished selected robot;
- group restoration target.

A failure of a resource released after its last user completed does not interrupt
solely through that released dependency. Shared and future resources remain
sensitive.
