#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: Apache-2.0
# Negative test: repair duration +1 h, so the event boundary must not match.
set -euo pipefail
cd "$(dirname "$0")"
case "$(uname -s)" in
  Darwin) platform=darwin; lib=poie_pv_battery.dylib; link="-dynamiclib" ;;
  Linux) platform=linux; lib=poie_pv_battery.so; link="-shared" ;;
  *) echo "unsupported OS" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  arm64|aarch64) arch=aarch64 ;;
  x86_64|amd64) arch=x86_64 ;;
  *) echo "unsupported arch" >&2; exit 1 ;;
esac
rm -rf build-tampered-duration
mkdir -p "build-tampered-duration/binaries/$arch-$platform" build-tampered-duration/src
sed 's/case VR_REPAIR_DURATION: c->repair_duration = value\[i\]; break;/case VR_REPAIR_DURATION: c->repair_duration = value[i] + 3600.0; break;/' src/model.c > build-tampered-duration/src/model.c
cc -O2 -fPIC "$link" -Iinclude -o "build-tampered-duration/binaries/$arch-$platform/$lib" build-tampered-duration/src/model.c src/stubs.c -lm
cp modelDescription.xml build-tampered-duration/
( cd build-tampered-duration && zip -q -r ../poie-pv-battery-duration-tampered.fmu modelDescription.xml binaries )
echo "built external/fmu-pv-battery/poie-pv-battery-duration-tampered.fmu ($arch-$platform)"
