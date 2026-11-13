#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Differential corpus over one-lot + one-process removals from Base-05."""

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

BLOCK = re.compile(r"\[\[(lot|process)\]\]\n(.*?)(?=\n\[\[|\Z)", re.S)


def remove(text: str, lot_id: str | None, process_id: str | None) -> str:
    def replace(match: re.Match[str]) -> str:
        kind, block = match.group(1), match.group(2)
        if kind == "lot" and lot_id and f'id = "{lot_id}"' in block:
            return ""
        if kind == "process" and process_id and f'id = "{process_id}"' in block:
            return ""
        return match.group(0)

    return BLOCK.sub(replace, text)


def main(binary: str) -> int:
    source = ROOT / "scenarios/base-05/model.toml"
    text = source.read_text()
    with source.open("rb") as handle:
        raw = tomllib.load(handle)
    lots = [lot["id"] for lot in raw.get("lot", []) if lot.get("quality", "accepted") == "accepted"]
    processes = [process["id"] for process in raw.get("process", [])]
    total = 0
    matched = 0
    with tempfile.TemporaryDirectory() as directory:
        for lot_id, process_id in itertools.product(lots, processes):
            path = Path(directory) / f"{lot_id}__{process_id}.toml"
            path.write_text(remove(text, lot_id, process_id))
            output = subprocess.run([binary, str(path)], check=True, capture_output=True, text=True).stdout
            rust = {row["target"]: row["outcome"] for row in json.loads(output)}
            python_model = Model(str(path))
            for target, outcome in rust.items():
                total += 1
                python = "RESTORABLE" if python_model.verdict(target, 24.0 * 3600.0) == "RESTORABLE" else "BLOCKED"
                if python == outcome:
                    matched += 1
                else:
                    print(f"MISMATCH lot={lot_id} process={process_id} target={target} rust={outcome} python={python}")
    print(f"mutated_models={len(lots) * len(processes)} cases={total} matched={matched} mismatched={total - matched}")
    return 0 if matched == total else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: mutate.py <poie-model-corpus-binary>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))
