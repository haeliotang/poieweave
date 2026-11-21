#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")"
case "$(uname -s)" in Darwin) platform=darwin; lib=poie_n15_scheduler.dylib; link="-dynamiclib";; Linux) platform=linux; lib=poie_n15_scheduler.so; link="-shared";; *) exit 1;; esac
case "$(uname -m)" in arm64|aarch64) arch=aarch64;; x86_64|amd64) arch=x86_64;; *) exit 1;; esac
rm -rf build_mutant; mkdir -p "build_mutant/binaries/$arch-$platform"
cc -O2 -fPIC -DMUTANT_PREMATURE_RELEASE "$link" -Iinclude -o "build_mutant/binaries/$arch-$platform/$lib" src/model.c src/stubs.c -lm
cp modelDescription.xml build_mutant/
( cd build_mutant && zip -q -r ../poie-n15-mutant.fmu modelDescription.xml binaries )
echo built-mutant
