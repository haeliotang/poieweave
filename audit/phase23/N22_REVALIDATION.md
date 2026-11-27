# N22 Revalidation

## Original preregistration
Preserved in `audit/phase22/PREREG.md`.

## Intermediate report
`audit/phase22/REPORT.zh-CN.md` accumulated intermediate status text before final
claims; the final claims file marks all gates VERIFIED.

## Final claims
`audit/phase22/CLAIMS.json`: N22-0/A/B/C/D/E/F/G VERIFIED, ESTABLISHED (SCOPED).

## Post-hoc revalidation
The final claims are consistent with the recorded evidence, but two limitations
were not fully reflected in early N22 narrative:

1. Candidate input used a synthetic checkpoint hash `"a"*64`, so N22 establishes
   bounded algorithmic choice-set validity, not trusted native-checkpoint-bound
   validity.
2. robot-m3 temporal commitment (20800) diverged from N22 FMI native output
   (19800); N22 compared C/fmpy consistency, not candidate-to-execution time
   conformance.

N22 history is frozen; these are corrected by N23 evidence, not by editing N22
certificates.
