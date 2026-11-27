#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
set -euo pipefail
cd "$(dirname "$0")/.."

cargo fmt --all -- --check
cargo clippy --workspace --all-targets --offline -- -D warnings
cargo test --workspace --locked --offline
reuse lint
python3 scripts/check-test-matrix.py
python3 scripts/check-experiment-replays.py
python3 scripts/check-status-counts.py
python3 scripts/check-claims.py
python3 scripts/build-s2-inventory.py --check
python3 scripts/check-certificate-invariants.py
python3 scripts/build-certificate-impact.py --check
python3 scripts/build-metrics-split.py --check
python3 scripts/verify-canonical-claim-certificates-v6-historical.py
python3 scripts/verify-canonical-claim-certificates-v7-historical.py
python3 scripts/verify-canonical-claim-certificates-v8-historical.py
python3 scripts/verify-canonical-claim-certificates-v9-historical.py
python3 scripts/verify-canonical-claim-certificates-v10.py
python3 scripts/build-canonical-claim-certificates-v17.py --check
python3 scripts/verify-canonical-claim-certificates-v11.py
python3 scripts/verify-canonical-claim-certificates-v12.py
python3 scripts/verify-canonical-claim-certificates-v13.py
python3 scripts/verify-canonical-claim-certificates-v14.py
python3 scripts/verify-canonical-claim-certificates-v15.py
python3 scripts/verify-canonical-claim-certificates-v16.py
python3 scripts/verify-canonical-claim-certificates-v17.py
python3 scripts/check-canonical-claim-certificate-tamper-v11.py
python3 scripts/check-canonical-claim-certificate-tamper-v12.py
python3 scripts/check-canonical-claim-certificate-tamper-v13.py
python3 scripts/check-canonical-claim-certificate-tamper-v14.py
python3 scripts/check-canonical-claim-certificate-tamper-v15.py
python3 scripts/check-canonical-claim-certificate-tamper-v16.py
python3 scripts/check-canonical-claim-certificate-tamper-v17.py
python3 scripts/check-canonical-claim-certificate-tamper-v10.py
python3 scripts/build-claim-certificate-coverage.py --check
python3 audit/phase1/check.py
python3 audit/phase2/check.py
python3 audit/phase2/check_observatory.py
python3 audit/phase3/check_native.py
python3 audit/phase4/check.py
python3 audit/phase5/check.py
python3 audit/phase6/check.py
python3 audit/phase6/check_evidence.py
python3 audit/phase7/compare_derived.py
python3 audit/phase7/check.py
python3 audit/phase8/check.py
python3 audit/phase8/negative_controls.py
python3 audit/phase8/compare_trajectory.py
python3 audit/phase8/negative_trajectory.py
python3 audit/phase8h/reproduce_false_pass.py
python3 audit/phase8h/compare_strict.py
python3 audit/phase8h/negative_controls.py
python3 audit/phase8h/check.py --frozen
python3 audit/phase8h/verify_evidence.py --frozen --claim
python3 audit/phase8h/negative_integrity.py
python3 audit/phase9/check.py --frozen
python3 audit/phase9/verify_evidence.py --frozen --claim
python3 audit/phase9/negative_resources.py
python3 audit/phase9/negative_integrity.py
python3 audit/phase10/compare_boundaries.py
python3 audit/phase10/negative_boundaries.py
python3 audit/phase10/compare_causal.py
python3 audit/phase10/negative_causal.py
python3 audit/phase10/compare_trajectory.py
python3 audit/phase10/negative_trajectory.py
python3 audit/phase11/validate_native.py
python3 audit/phase11/negative_native.py
python3 audit/phase11/check.py
python3 audit/phase12/validate_identity.py
python3 audit/phase12/validate_projection.py
python3 audit/phase12/negative_projection.py
python3 audit/phase12/validate_stress.py
python3 audit/phase12/check.py
python3 audit/phase12/verify_evidence.py --frozen --claim
python3 audit/phase12/negative_evidence.py
python3 audit/phase13/validate_multifailure.py
python3 audit/phase13/negative_multifailure.py
python3 audit/phase13/validate_metamorphic.py
python3 audit/phase13/check.py
python3 audit/phase13/verify_evidence.py --frozen --claim
python3 audit/phase13/negative_evidence.py
python3 audit/phase14/validate_composition.py
python3 audit/phase14/check.py
python3 audit/phase14/verify_evidence.py --frozen --claim
python3 audit/phase14/negative_evidence.py
python3 audit/phase14r/check.py
python3 audit/phase14r/verify_evidence.py --frozen --claim
python3 audit/phase15/check.py
python3 audit/phase16/check.py
python3 audit/phase18/check.py
python3 audit/phase19/check.py
python3 audit/phase20/check.py
python3 audit/phase21/check.py
python3 audit/phase14rh/check.py
python3 audit/phase10/check.py --frozen
python3 audit/phase7/negative_controls.py
python3 audit/certificates/verify_v1.py
python3 audit/certificates/verify_v1_historical.py
python3 audit/certificates/verify_v2.py
python3 audit/certificates/verify_v3.py
python3 audit/certificates/verify_v4.py
python3 audit/certificates/verify_v5.py
python3 scripts/check-docs.py

echo "ok all checks"
