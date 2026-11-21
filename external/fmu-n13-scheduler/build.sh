#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
case "$(uname -s)" in Darwin) platform=darwin; lib=poie_n13_scheduler.dylib; link="-dynamiclib";; Linux) platform=linux; lib=poie_n13_scheduler.so; link="-shared";; *) exit 1;; esac
case "$(uname -m)" in arm64|aarch64) arch=aarch64;; x86_64|amd64) arch=x86_64;; *) exit 1;; esac
rm -rf build; mkdir -p "build/binaries/$arch-$platform"
cc -O2 -fPIC "$link" -Iinclude -o "build/binaries/$arch-$platform/$lib" src/model.c src/stubs.c -lm
cp modelDescription.xml build/
( cd build && zip -q -r ../poie-n13-scheduler.fmu modelDescription.xml binaries )
cc -O2 -Iinclude -o build/harness src/harness.c -ldl
echo built
