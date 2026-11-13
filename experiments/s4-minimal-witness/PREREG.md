# S4 Minimal Witness Fixtures — pre-registration

- Experiment ID: `s4-minimal-witness`
- Date: 2026-11-14
- Status: completed; see [REPORT.md](REPORT.md)
- Scope: exact fixture blocker values, not general minimal witnesses.

## Question

Can the independent checker reproduce the exact blocking values, not only the
reason class, for the missing-resource and unverified-process fixtures?

## Method

For Base-09 pump:

- parse `missing resource R: need N, have H`;
- independently find the process input requiring R;
- compare N and H to the model stock.

For Base-10 lathe:

- parse `unverified process P: Actual < Required`;
- independently read P's verification;
- check Actual < Required under the modeled gate.

Prediction: both witnesses match exactly.

## Limitation

Two fixtures only; no general minimal-witness search.
