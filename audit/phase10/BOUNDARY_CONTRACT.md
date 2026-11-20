# N10 Boundary Contract (preregistered)

Boundary evidence must distinguish:

- nominal step end
- repair group completion
- resource dependency still active
- failure applied
- restoration committed
- horizon
- run termination
- native resource release

`RunTerminated` is not a native release observation. Event before/after states are
separate; an external endpoint row is `ENDPOINT_ONLY`, not a full trajectory.
