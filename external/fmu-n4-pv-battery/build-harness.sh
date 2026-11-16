#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: Apache-2.0
set -euo pipefail
cd "$(dirname "$0")"
cc -O2 -Iinclude -o build/harness src/harness.c -ldl
