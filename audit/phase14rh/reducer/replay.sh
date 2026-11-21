#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../../.."
cargo build -q -p poie-viability --bin poie-s5v-n14-canonical-adapter
external/fmu-n14-scheduler/build.sh
python3 audit/phase14rh/reducer/reduce.py
