# Pre-start continuation failure false pass

N23 FMU ignored a robot-m3 failure at 9500 because the failure was earlier than the
scheduled start 10000, then completed at 20800. N24 FMU applies the failure, marks
preStartInvalidated=1, does not restore, and keeps robot-m3 (no automatic switch).
