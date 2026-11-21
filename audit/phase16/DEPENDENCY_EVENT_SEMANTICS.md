# Dependency Event Semantics

- Selected robot/tool leaving ownership: `ResourceReleased`.
- Named upstream leaving C(t): `DependencyDeactivated`.
- Initial upstream entries: native dependency state in InitialRepairStarted.
- Failure of a deactivated upstream is still recorded as `AssetFailure` but does
  not interrupt through that deactivated dependency.
