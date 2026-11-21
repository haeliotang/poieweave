#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
case "$(uname -s)" in Darwin) platform=darwin; lib=poie_n13_scheduler.dylib; link="-dynamiclib";; Linux) platform=linux; lib=poie_n13_scheduler.so; link="-shared";; *) exit 1;; esac
case "$(uname -m)" in arm64|aarch64) arch=aarch64;; x86_64|amd64) arch=x86_64;; *) exit 1;; esac
rm -rf build_overflow; mkdir -p "build_overflow/binaries/$arch-$platform"
cc -O2 -fPIC -DJOURNAL_CAPACITY=3 "$link" -Iinclude -o "build_overflow/binaries/$arch-$platform/$lib" src/model.c src/stubs.c -lm
cp modelDescription.xml build_overflow/
( cd build_overflow && zip -q -r ../poie-n13-overflow.fmu modelDescription.xml binaries )
echo built-overflow
