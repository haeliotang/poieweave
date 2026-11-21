# Upstream Identity Contract

Assets: robot-m1, robot-m2, tool-t1, bus-1, bus-2, charger-1, bat-1, sensor-x.

`dependency_users(asset,t)` is derived from unfinished tasks' selected paths.
`DependencyDeactivated(asset,t)` is valid only when users become empty.
The physical asset status remains independent after deactivation.
