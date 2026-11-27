#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
case "$(uname -s)" in Darwin) platform=darwin; lib=poie_n24_multitask_recovery.dylib; link="-dynamiclib";; Linux) platform=linux; lib=poie_n24_multitask_recovery.so; link="-shared";; *) exit 1;; esac
case "$(uname -m)" in arm64|aarch64) arch=aarch64;; x86_64|amd64) arch=x86_64;; *) exit 1;; esac
rm -rf build; mkdir -p "build/binaries/$arch-$platform"
cc -O2 -fPIC ${EXTRA_CFLAGS:-} "$link" -Iinclude -o "build/binaries/$arch-$platform/$lib" model.c stubs.c -lm
cp modelDescription.xml build/
( cd build && zip -q -r ../${FMU_NAME:-poie-n24-multitask-recovery.fmu} modelDescription.xml binaries )
echo built
