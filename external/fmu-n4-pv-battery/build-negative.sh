#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail
cd "$(dirname "$0")"
kind="${1:?usage: build-negative.sh regen|duration}"
case "$(uname -s)" in
  Darwin) platform=darwin; lib=poie_n4_pv_battery.dylib; link="-dynamiclib" ;;
  Linux) platform=linux; lib=poie_n4_pv_battery.so; link="-shared" ;;
  *) exit 1 ;;
esac
case "$(uname -m)" in
  arm64|aarch64) arch=aarch64 ;;
  x86_64|amd64) arch=x86_64 ;;
  *) exit 1 ;;
esac
work="build-negative-$kind"
rm -rf "$work"
mkdir -p "$work/binaries/$arch-$platform" "$work/src"
if [ "$kind" = regen ]; then
  sed 's/fmi3Float64 pv = c->pv_failed ? 0.0 : fmin(c->pv_power, c->max_charge);/fmi3Float64 pv = fmin(c->pv_power, c->max_charge);/' src/model.c > "$work/src/model.c"
elif [ "$kind" = order ]; then
  python3 - "$work/src/model.c" <<'PYEOF'
import pathlib, sys
source = pathlib.Path('src/model.c').read_text()
repair_block = """        if (c->repair_completed_time < 0.0 && c->repair_duration > 0.0 && c->time >= c->repair_duration - 1e-9) {
            c->repair_completed_time = c->repair_duration;
            c->last_event_code = 2.0;
            c->last_event_time = c->repair_duration;
        }
"""
failure_block = """        if (!c->pv_failed && c->pv_failure_time >= 0.0 && c->time >= c->pv_failure_time - 1e-9) {
            c->pv_failed = 1.0;
            c->failure_applied_time = c->pv_failure_time;
            c->last_event_code = 1.0;
            c->last_event_time = c->pv_failure_time;
        }
"""
if repair_block not in source or failure_block not in source:
    raise SystemExit('order tamper anchors not found')
pathlib.Path(sys.argv[1]).write_text(source.replace(repair_block + failure_block, failure_block + repair_block))
PYEOF
elif [ "$kind" = duration ]; then
  sed 's/case VR_REPAIR_DURATION: c->repair_duration = value\[i\]; break;/case VR_REPAIR_DURATION: c->repair_duration = value[i] + 3600.0; break;/' src/model.c > "$work/src/model.c"
else
  echo "unknown kind" >&2
  exit 1
fi
cc -O2 -fPIC "$link" -Iinclude -o "$work/binaries/$arch-$platform/$lib" "$work/src/model.c" src/stubs.c -lm
cp modelDescription.xml "$work/"
( cd "$work" && zip -q -r "../poie-n4-pv-battery-$kind-tampered.fmu" modelDescription.xml binaries )
echo "built poie-n4-pv-battery-$kind-tampered.fmu"
