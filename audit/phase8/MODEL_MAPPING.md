# N8 Cross-Model Event Mapping

Planned shared observable contract:

```text
AssetFailure:
  canonical  AssetFailure -> asset_status Failed
  external   failure flag/status -> unavailable
RepairCompleted:
  canonical  RepairCompleted -> target Available
  external   restoration flag/time
RepairInterrupted:
  canonical  RepairInterrupted -> no restoration
  external   interrupted flag/no completion
```

Status: NOT YET IMPLEMENTED / NOT ESTABLISHED. This file registers the mapping
intent only; no cross-model event claim is made in the current slice.
