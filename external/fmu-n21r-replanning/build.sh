#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
cc -O2 -o external_replan external_replan.c -lm
echo built
