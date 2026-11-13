#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Differential corpus over all three-lot removals from Base-05."""

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


def remove_lots(text: str, lot_ids: set[str]) -> str:
    def replace(match: re.Match[str]) -> str:
        block = match.group(0)
        return "" if any(f'id = "{lot_id}"' in block for lot_id in lot_ids) else block

    return LOT.sub(replace, text)


def main(binary: str) -> int:
    source = ROOT / "scenarios/base-05/model.toml"
    text = source.read_text()
    with source.open("rb") as handle:
        raw = tomllib.load(handle)
    accepted = [lot["id"] for lot in raw.get("lot", []) if lot.get("quality", "accepted") == "accepted"]
    triples = list(itertools.combinations(accepted, 3))
    total = 0
    matched = 0
    with tempfile.TemporaryDirectory() as directory:
        for triple in triples:
            path = Path(directory) / f"{'__'.join(triple)}.toml"
            path.write_text(remove_lots(text, set(triple)))
            output = subprocess.run([binary, str(path)], check=True, capture_output=True, text=True).stdout
            rust = {row["target"]: row["outcome"] for row in json.loads(output)}
            python_model = Model(str(path))
            for target, outcome in rust.items():
                total += 1
                python = "RESTORABLE" if python_model.verdict(target, 24.0 * 3600.0) == "RESTORABLE" else "BLOCKED"
                if python == outcome:
                    matched += 1
                else:
                    print(f"MISMATCH triple={triple} target={target} rust={outcome} python={python}")
    print(f"mutated_models={len(triples)} cases={total} matched={matched} mismatched={total - matched}")
    return 0 if matched == total else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: mutate.py <poie-model-corpus-binary>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))
