#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Differential corpus: one degraded asset + one doubled process duration."""

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
ASSET = re.compile(r"\[\[asset\]\]\n(.*?)(?=\n\[\[|\Z)", re.S)
DURATION = re.compile(r'^duration = "([0-9.]+) ([^"]+)"$', re.M)


def mutate(text: str, asset_id: str, process_id: str) -> str:
    def degrade(match: re.Match[str]) -> str:
        block = match.group(0)
        if f'id = "{asset_id}"' not in block:
            return block
        return re.sub(r'^status = ".*"$', 'status = "degraded"', block, count=1, flags=re.M)

    def double_duration(match: re.Match[str]) -> str:
        block = match.group(0)
        if f'id = "{process_id}"' not in block:
            return block
        return DURATION.sub(
            lambda m: f'duration = "{float(m.group(1)) * 2.0:g} {m.group(2)}"',
            block,
            count=1,
        )

    return PROCESS.sub(double_duration, ASSET.sub(degrade, text))


def main(binary: str) -> int:
    source = ROOT / "scenarios/base-05/model.toml"
    text = source.read_text()
    with source.open("rb") as handle:
        raw = tomllib.load(handle)
    assets = [asset["id"] for asset in raw.get("asset", [])]
    processes = [process["id"] for process in raw.get("process", [])]
    total = 0
    matched = 0
    with tempfile.TemporaryDirectory() as directory:
        for asset_id, process_id in itertools.product(assets, processes):
            path = Path(directory) / f"{asset_id}__{process_id}.toml"
            path.write_text(mutate(text, asset_id, process_id))
            output = subprocess.run([binary, str(path)], check=True, capture_output=True, text=True).stdout
            rust = {row["target"]: row["outcome"] for row in json.loads(output)}
            python_model = Model(str(path))
            for target, outcome in rust.items():
                total += 1
                python = "RESTORABLE" if python_model.verdict(target, 24.0 * 3600.0) == "RESTORABLE" else "BLOCKED"
                if python == outcome:
                    matched += 1
                else:
                    print(f"MISMATCH asset={asset_id} process={process_id} target={target} rust={outcome} python={python}")
    print(f"mutated_models={len(assets) * len(processes)} cases={total} matched={matched} mismatched={total - matched}")
    return 0 if matched == total else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: mutate.py <poie-model-corpus-binary>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))
