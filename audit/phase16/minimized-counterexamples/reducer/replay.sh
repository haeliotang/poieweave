#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
cargo build -q -p poie-viability --bin poie-s5v-n14-canonical-adapter
external/fmu-n16-scheduler/build_mutant.sh
python3 audit/phase16/reduce_dependency.py
