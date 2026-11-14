#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Differential corpus: scale accepted Base-05 lots to 25/50/75 percent."""

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

LOT = re.compile(r"\[\[lot\]\]\n(.*?)(?=\n\[\[|\Z)", re.S)
QUANTITY = re.compile(r'^quantity = "([0-9.]+) ([^"]+)"$', re.M)


def scale(text: str, lot_id: str, ratio: float) -> str:
    def replace(match: re.Match[str]) -> str:
        block = match.group(0)
        if f'id = "{lot_id}"' not in block:
            return block

        def quantity(match: re.Match[str]) -> str:
            value = float(match.group(1)) * ratio
            return f'quantity = "{value:g} {match.group(2)}"'

        return QUANTITY.sub(quantity, block, count=1)

    return LOT.sub(replace, text)


def main(binary: str) -> int:
    source = ROOT / "scenarios/base-05/model.toml"
    text = source.read_text()
    with source.open("rb") as handle:
        raw = tomllib.load(handle)
    lots = [lot["id"] for lot in raw.get("lot", []) if lot.get("quality", "accepted") == "accepted"]
    ratios = [0.25, 0.5, 0.75]
    total = 0
    matched = 0
    with tempfile.TemporaryDirectory() as directory:
        for lot_id, ratio in itertools.product(lots, ratios):
            path = Path(directory) / f"{lot_id}-{ratio}.toml"
            path.write_text(scale(text, lot_id, ratio))
            output = subprocess.run([binary, str(path)], check=True, capture_output=True, text=True).stdout
            rust = {row["target"]: row["outcome"] for row in json.loads(output)}
            python_model = Model(str(path))
            for target, outcome in rust.items():
                total += 1
                python = "RESTORABLE" if python_model.verdict(target, 24.0 * 3600.0) == "RESTORABLE" else "BLOCKED"
                if python == outcome:
                    matched += 1
                else:
                    print(f"MISMATCH lot={lot_id} ratio={ratio} target={target} rust={outcome} python={python}")
    print(f"mutated_models={len(lots) * len(ratios)} cases={total} matched={matched} mismatched={total - matched}")
    return 0 if matched == total else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: mutate.py <poie-model-corpus-binary>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))
