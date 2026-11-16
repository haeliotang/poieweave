#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: Apache-2.0
# Build a deliberately wrong FMU: +100 W PV, used only as a negative test.
set -euo pipefail
cd "$(dirname "$0")"
case "$(uname -s)" in
  Darwin) platform=darwin; lib=poie_pv_battery.dylib; link="-dynamiclib" ;;
  Linux) platform=linux; lib=poie_pv_battery.so; link="-shared" ;;
  *) echo "unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  arm64|aarch64) arch=aarch64 ;;
  x86_64|amd64) arch=x86_64 ;;
  *) echo "unsupported arch: $(uname -m)" >&2; exit 1 ;;
esac
rm -rf build-tampered
mkdir -p "build-tampered/binaries/$arch-$platform" build-tampered/src
sed 's/c->pv_power = value\[i\];/c->pv_power = value[i] + 100.0;/' src/model.c > build-tampered/src/model.c
cc -O2 -fPIC "$link" -Iinclude -o "build-tampered/binaries/$arch-$platform/$lib" build-tampered/src/model.c src/stubs.c -lm
cp modelDescription.xml build-tampered/
( cd build-tampered && zip -q -r ../poie-pv-battery-tampered.fmu modelDescription.xml binaries )
echo "built external/fmu-pv-battery/poie-pv-battery-tampered.fmu ($arch-$platform)"
