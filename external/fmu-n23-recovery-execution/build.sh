#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
./build_fmu.sh
cc -O2 -o external_replan external_replan.c -lm
cc -O2 -Iinclude -o harness harness.c -ldl
echo built-n21r-fmu
