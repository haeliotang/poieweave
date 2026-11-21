#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
cargo build -q -p poie-viability --bin poie-s5v-n14-canonical-adapter
target/debug/poie-s5v-n14-canonical-adapter audit/phase18/failing-witness/original_contract.json
