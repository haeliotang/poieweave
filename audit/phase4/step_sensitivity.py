#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Compare N4 harness and fmpy terminal/event times across communication steps."""

from __future__ import annotations

import csv
import json
from pathlib import Path

PHASE = Path(__file__).resolve().parent
CONFIG = json.loads((PHASE / "config.json").read_text())


def last_row(path: Path) -> dict[str, str]:
    with path.open(newline="") as handle:
        return list(csv.DictReader(handle))[-1]


def main() -> int:
    rows = []
    ok = True
    for case in [entry for entry in CONFIG["cases"] if entry.get("failure_asset", "pv-1") == "pv-1"]:
        cid = case["id"]
        expected_terminal = case["expected_terminal_s"]
        for step in CONFIG["steps_s"]:
            for tool in ("harness", "fmpy"):
                path = PHASE / "results" / f"{cid}-{tool}-{step}.csv"
                if not path.is_file():
                    continue
                row = last_row(path)
                terminal = float(row["time_s"])
                failure = float(row["failureAppliedTime_s"])
                expected_failure = case["failure_s"] if case["failure_s"] is not None else -1.0
                terminal_ok = abs(terminal - expected_terminal) <= 1e-6
                failure_ok = abs(failure - expected_failure) <= 1e-6
                ok = ok and terminal_ok and failure_ok
                rows.append({"case": cid, "step_s": step, "tool": tool, "terminal_s": terminal,
                             "expected_terminal_s": expected_terminal, "terminal_ok": terminal_ok,
                             "failure_s": failure, "expected_failure_s": expected_failure, "failure_ok": failure_ok})
    out = PHASE / "results" / "step-sensitivity.json"
    out.write_text(json.dumps({"schema": "poie.s5v-n4-step-sensitivity/1", "rows": rows}, indent=2) + "\n")
    for row in rows:
        print(row["case"], row["tool"], row["step_s"], "terminal", row["terminal_ok"], "failure", row["failure_ok"])
    return 0 if ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
