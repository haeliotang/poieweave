# Dependency Closure Contract

```text
direct_resources(task) = selected robot(s) + required tool(s)
upstream_dependencies(task) = named assets reachable from selected resources
dependency_closure(task) = upstream_dependencies ∪ direct_resources
C(t) = union dependency_closure(task) for unfinished tasks at t
```

Resource ownership (robot/tool) and causal upstream dependency are distinct.
`ResourceReleased` for a robot/tool is not a physical upstream release.
Upstream exit is represented as `DependencyDeactivated`, not physical shutdown.

A shared upstream stays in C(t) while any unfinished task depends on it.
