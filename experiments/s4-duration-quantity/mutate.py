#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Differential corpus: one doubled process duration + one halved lot."""

from __future__ import annotations

import itertools
import json
import re
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "experiments/s4-recursive-closure"))
from solve import Model  # noqa: E402

PROCESS = re.compile(r"\[\[process\]\]\n(.*?)(?=\n\[\[|\Z)", re.S)
LOT = re.compile(r"\[\[lot\]\]\n(.*?)(?=\n\[\[|\Z)", re.S)
DURATION = re.compile(r'^duration = "([0-9.]+) ([^"]+)"$', re.M)
QUANTITY = re.compile(r'^quantity = "([0-9.]+) ([^"]+)"$', re.M)


def mutate(text: str, process_id: str, lot_id: str) -> str:
    def double_duration(match: re.Match[str]) -> str:
        block = match.group(0)
        if f'id = "{process_id}"' not in block:
            return block

        def duration(match: re.Match[str]) -> str:
            value = float(match.group(1)) * 2.0
            return f'duration = "{value:g} {match.group(2)}"'

        return DURATION.sub(duration, block, count=1)

    def halve_lot(match: re.Match[str]) -> str:
        block = match.group(0)
        if f'id = "{lot_id}"' not in block:
            return block

        def quantity(match: re.Match[str]) -> str:
            value = float(match.group(1)) / 2.0
            return f'quantity = "{value:g} {match.group(2)}"'

        return QUANTITY.sub(quantity, block, count=1)

    return LOT.sub(halve_lot, PROCESS.sub(double_duration, text))


def main(binary: str) -> int:
    source = ROOT / "scenarios/base-05/model.toml"
    text = source.read_text()
    with source.open("rb") as handle:
        raw = tomllib.load(handle)
    processes = [process["id"] for process in raw.get("process", [])]
    lots = [lot["id"] for lot in raw.get("lot", []) if lot.get("quality", "accepted") == "accepted"]
    total = 0
    matched = 0
    with tempfile.TemporaryDirectory() as directory:
        for process_id, lot_id in itertools.product(processes, lots):
            path = Path(directory) / f"{process_id}__{lot_id}.toml"
            path.write_text(mutate(text, process_id, lot_id))
            output = subprocess.run([binary, str(path)], check=True, capture_output=True, text=True).stdout
            rust = {row["target"]: row["outcome"] for row in json.loads(output)}
            python_model = Model(str(path))
            for target, outcome in rust.items():
                total += 1
                python = "RESTORABLE" if python_model.verdict(target, 24.0 * 3600.0) == "RESTORABLE" else "BLOCKED"
                if python == outcome:
                    matched += 1
                else:
                    print(f"MISMATCH process={process_id} lot={lot_id} target={target} rust={outcome} python={python}")
    print(f"mutated_models={len(processes) * len(lots)} cases={total} matched={matched} mismatched={total - matched}")
    return 0 if matched == total else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: mutate.py <poie-model-corpus-binary>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))
