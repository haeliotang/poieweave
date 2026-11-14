#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Deterministic random combined mutations of Base-05, Rust vs Python."""

from __future__ import annotations

import json
import random
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
ASSET = re.compile(r"\[\[asset\]\]\n(.*?)(?=\n\[\[|\Z)", re.S)
PROCESS = re.compile(r"\[\[process\]\]\n(.*?)(?=\n\[\[|\Z)", re.S)
CAPABILITY = re.compile(r'^capability_verification = \{ (.*) \}$', re.M)


def mutate(rng: random.Random, text: str, assets: list[str], lots: list[str], processes: list[str], capabilities: list[str]) -> str:
    failed = set(rng.sample(assets, rng.randint(0, 3)))
    dropped = set(rng.sample(lots, rng.randint(0, 2)))
    downgraded = set(rng.sample(processes, rng.randint(0, 2)))
    weak_caps = set(rng.sample(capabilities, rng.randint(0, 2)))

    def asset_replace(match: re.Match[str]) -> str:
        block = match.group(0)
        if not any(f'id = "{asset_id}"' in block for asset_id in failed):
            return block
        return re.sub(r'^status = ".*"$', 'status = "failed"', block, count=1, flags=re.M)

    def lot_replace(match: re.Match[str]) -> str:
        block = match.group(0)
        return "" if any(f'id = "{lot_id}"' in block for lot_id in dropped) else block

    def process_replace(match: re.Match[str]) -> str:
        block = match.group(0)
        if not any(f'id = "{process_id}"' in block for process_id in downgraded):
            return block
        return re.sub(r'^verification = ".*"$', 'verification = "declared"', block, count=1, flags=re.M)

    def capability_replace(match: re.Match[str]) -> str:
        entries = []
        for name, value in re.findall(r'"([^"]+)" = "([^"]+)"', match.group(1)):
            entries.append(f'"{name}" = "{"declared" if name in weak_caps else value}"')
        return "capability_verification = { " + ", ".join(entries) + " }"

    text = ASSET.sub(asset_replace, text)
    text = LOT.sub(lot_replace, text)
    text = PROCESS.sub(process_replace, text)
    return CAPABILITY.sub(capability_replace, text, count=1)


def main(binary: str) -> int:
    source = ROOT / "scenarios/base-05/model.toml"
    text = source.read_text()
    with source.open("rb") as handle:
        raw = tomllib.load(handle)
    assets = [asset["id"] for asset in raw.get("asset", [])]
    lots = [lot["id"] for lot in raw.get("lot", []) if lot.get("quality", "accepted") == "accepted"]
    processes = [process["id"] for process in raw.get("process", [])]
    capabilities = re.findall(r'"([^"]+)" = "', CAPABILITY.search(text).group(1))
    rng = random.Random(20261114)
    total = 0
    matched = 0
    with tempfile.TemporaryDirectory() as directory:
        for index in range(100):
            path = Path(directory) / f"case-{index:03d}.toml"
            path.write_text(mutate(rng, text, assets, lots, processes, capabilities))
            output = subprocess.run([binary, str(path)], check=True, capture_output=True, text=True).stdout
            rust = {row["target"]: row["outcome"] for row in json.loads(output)}
            python_model = Model(str(path))
            for target, outcome in rust.items():
                total += 1
                python = "RESTORABLE" if python_model.verdict(target, 24.0 * 3600.0) == "RESTORABLE" else "BLOCKED"
                if python == outcome:
                    matched += 1
                else:
                    print(f"MISMATCH case={index} target={target} rust={outcome} python={python}")
    print(f"mutated_models=100 cases={total} matched={matched} mismatched={total - matched}")
    return 0 if matched == total else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: mutate.py <poie-model-corpus-binary>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))
