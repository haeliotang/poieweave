#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail
cd "$(dirname "$0")"
case "$(uname -s)" in
  Darwin) platform=darwin; lib=poie_n4_pv_battery.dylib; link="-dynamiclib" ;;
  Linux) platform=linux; lib=poie_n4_pv_battery.so; link="-shared" ;;
  *) echo "unsupported OS" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  arm64|aarch64) arch=aarch64 ;;
  x86_64|amd64) arch=x86_64 ;;
  *) echo "unsupported arch" >&2; exit 1 ;;
esac
rm -rf build
mkdir -p "build/binaries/$arch-$platform"
cc -O2 -fPIC "$link" -Iinclude -o "build/binaries/$arch-$platform/$lib" src/model.c src/stubs.c -lm
cp modelDescription.xml build/
( cd build && zip -q -r ../poie-n4-pv-battery.fmu modelDescription.xml binaries )
echo "built external/fmu-n4-pv-battery/poie-n4-pv-battery.fmu ($arch-$platform)"
