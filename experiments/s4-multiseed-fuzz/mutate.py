#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Multi-seed combined mutation differential on Base-05."""

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
sys.path.insert(0, str(ROOT / "experiments/s4-random-fuzz"))
from mutate import mutate  # noqa: E402
from solve import Model  # noqa: E402

CAPABILITY = re.compile(r'^capability_verification = \{ (.*) \}$', re.M)


def main(binary: str) -> int:
    source = ROOT / "scenarios/base-05/model.toml"
    text = source.read_text()
    with source.open("rb") as handle:
        raw = tomllib.load(handle)
    assets = [asset["id"] for asset in raw.get("asset", [])]
    lots = [lot["id"] for lot in raw.get("lot", []) if lot.get("quality", "accepted") == "accepted"]
    processes = [process["id"] for process in raw.get("process", [])]
    capabilities = re.findall(r'"([^"]+)" = "', CAPABILITY.search(text).group(1))
    seeds = [20261114, 20261115, 20261116, 20261117, 20261118]
    total = 0
    matched = 0
    with tempfile.TemporaryDirectory() as directory:
        for seed in seeds:
            rng = random.Random(seed)
            for index in range(100):
                path = Path(directory) / f"seed-{seed}-{index:03d}.toml"
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
                        print(f"MISMATCH seed={seed} case={index} target={target} rust={outcome} python={python}")
    print(f"seeds={len(seeds)} mutated_models={len(seeds) * 100} cases={total} matched={matched} mismatched={total - matched}")
    return 0 if matched == total else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: mutate.py <poie-model-corpus-binary>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))
