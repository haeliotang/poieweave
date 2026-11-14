# S2 Insert Sharpening — report

- Date: 2026-11-14
- Status: positive; sharpening extends the horizon, after a discovered index bug
- Runner: `cargo run --locked -q -p poie-viability --bin poie-insert-regeneration`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
insert_count	mode	survived_days	blocked_by
2	baseline	40	missing resource cutting-insert: need 1, have 0
2	sharpen	60	cyclic process dependency at turn-sleeve-p1
3	baseline	50	missing resource cutting-insert: need 1, have 0
3	sharpen	60	cyclic process dependency at turn-sleeve-p1
4	baseline	60	cyclic process dependency at turn-sleeve-p1
4	sharpen	60	cyclic process dependency at turn-sleeve-p1
```

## 2. Discovered error

The first run showed no effect. The cause was that `Model.processes` is public
and mutable, but `Model.producers` (used by closure analysis) is precomputed at
load time. Appending to `processes` directly left the producer index stale, so
`sharpen-insert` was silently invisible to S1.

Fix: new `Model::add_process` keeps `producers`/`restorers` consistent, with a
unit test. With the fix, sharpening works as intended.

## 3. Findings

- 2 inserts: horizon 40 -> 60 days with sharpening.
- 3 inserts: horizon 50 -> 60 days with sharpening.
- 4 inserts: already 60 days (cycle-limited), sharpening adds no further gain.
- Regenerating a consumed byproduct (worn insert) visibly moves the survival
  boundary.

## 4. Significance

- This is the first S2 result where a regeneration route actually changes the
  horizon, and it required fixing an S1 data-structure bug to see it.
- It also confirms that byproduct-based recovery is usable by the current plan
  search once the index is correct.

## 5. Limits

- One process definition and one greedy policy.
- Model-level units.
