#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; problems=[]
w=json.loads((P/"failing-witnesses/shared-tool/result.json").read_text())
if not (w["v1_parallel_candidate"] and not w["v2_parallel_candidate"]): problems.append("shared tool")
r=json.loads((P/"historical-revalidation/n20-revalidation.json").read_text())["stats"]
if r["revalidated_accepted"]!=64 or r["revalidated_rejected"]!=0: problems.append("n20 revalidation")
if "BLOCKED" not in (P/"REPLANNING_REPRESENTABILITY_AUDIT.md").read_text(): problems.append("blocked audit")
if problems: print("N21 FAIL: "+", ".join(problems)); sys.exit(1)
print("ok N21 phase0/shared-tool/revalidation; replanning BLOCKED")
