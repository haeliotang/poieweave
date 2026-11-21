# Main History Audit — PR #617 / f80bcfc / 9a08498

- `f80bcfc` is the PR #617 merge commit.
- `9a08498` was pushed directly to `main` after the merge and changed:
  - `audit/phase14r/generate_cross_model.py` (relative contract path)
  - `audit/phase14r/generated-cases/results.json`
  - `audit/phase14r/manifest.json` + sidecar
  - 41 v7 certificate files (native N14R manifest hash changed)
- Reason: make generated evidence deterministic and re-freeze after manifest update.
- CI ran on `9a08498` and was green.
- v7 certificate issuance content changed only in the bound native N14R manifest SHA and the resulting certificate bytes; claim/experiment/runtime bindings were not changed.
- Branch protection readback: `gh api repos/haeliotang/poieweave/branches/main/protection` returned `Branch not protected`.
- No history rewrite was performed.
