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
python3 scripts/check-certificate-invariants.py
python3 scripts/check-docs.py

echo "ok all checks"
