# S2 Insert Sharpening — report

- Date: 2026-11-14
- Status: negative result; sharpening did not change the horizon
- Runner: `cargo run --locked -q -p poie-viability --bin poie-insert-regeneration`
- Output: [expected.txt](expected.txt)

## 1. Result

```text
insert_count	mode	survived_days	blocked_by
2	baseline	40	missing resource cutting-insert: need 1, have 0
2	sharpen	40	missing resource cutting-insert: need 1, have 0
3	baseline	50	missing resource cutting-insert: need 1, have 0
3	sharpen	50	missing resource cutting-insert: need 1, have 0
4	baseline	60	cyclic process dependency at turn-sleeve-p1
4	sharpen	60	cyclic process dependency at turn-sleeve-p1
```

## 2. Findings

- Adding a modeled sharpening process leaves every horizon unchanged.
- The process was not selected by the greedy recovery policy in these states;
  the cause is not yet resolved.
- This is a negative result: a declared regeneration route does not
  automatically become part of the recovery plan.

## 3. Next probes

- Trace the S1 plan at the blocked state to see whether `sharpen-insert` is
  considered and why it loses to the direct path.
- Check whether byproduct `worn-insert` is visible as an available input at the
  decision point.
- Add an S1 unit test for regenerating a consumed input from a byproduct.

## 4. Limits

- One process definition and one policy.
- Model-level units.
