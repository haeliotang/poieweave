#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Differential corpus: add a 0.5 process yield to each Base-05 process."""

from __future__ import annotations

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
OUTPUTS = re.compile(r"outputs = \[(.*?)\]", re.S)
RESOURCE = re.compile(r'resource = "([^"]+)"')


def resource_doc(raw: dict, resource_id: str) -> str:
    for resource in raw.get("resource", []):
        if resource["id"] == resource_id:
            lines = ['[[resource]]', f'id = "yield-reject"', f'unit = "{resource["unit"]}"']
            if "mass_per_unit" in resource:
                lines.append(f'mass_per_unit = "{resource["mass_per_unit"]}"')
            return "\n".join(lines)
    raise ValueError(resource_id)


def with_yield(text: str, process_id: str, raw: dict) -> str:
    target = next(match for match in PROCESS.finditer(text) if f'id = "{process_id}"' in match.group(0))
    outputs = OUTPUTS.search(target.group(0))
    output_resource = RESOURCE.search(outputs.group(1)).group(1)
    line = f'\nyield = {{ output = "{output_resource}", fraction = 0.5, reject = "yield-reject" }}'

    def replace(match: re.Match[str]) -> str:
        block = match.group(0)
        if f'id = "{process_id}"' not in block:
            return block
        block_outputs = OUTPUTS.search(block)
        return block[: block_outputs.end()] + line + block[block_outputs.end() :]

    result = PROCESS.sub(replace, text)
    resources = resource_doc(raw, output_resource)
    return result.replace("# --- Assets", resources + "\n\n# --- Assets", 1)

def main(binary: str) -> int:
    source = ROOT / "scenarios/base-05/model.toml"
    text = source.read_text()
    with source.open("rb") as handle:
        raw = tomllib.load(handle)
    processes = [process["id"] for process in raw.get("process", [])]
    total = 0
    matched = 0
    with tempfile.TemporaryDirectory() as directory:
        for process_id in processes:
            path = Path(directory) / f"{process_id}.toml"
            path.write_text(with_yield(text, process_id, raw))
            output = subprocess.run([binary, str(path)], check=True, capture_output=True, text=True).stdout
            rust = {row["target"]: row["outcome"] for row in json.loads(output)}
            python_model = Model(str(path))
            for target, outcome in rust.items():
                total += 1
                python = "RESTORABLE" if python_model.verdict(target, 24.0 * 3600.0) == "RESTORABLE" else "BLOCKED"
                if python == outcome:
                    matched += 1
                else:
                    print(f"MISMATCH process={process_id} target={target} rust={outcome} python={python}")
    print(f"mutated_models={len(processes)} cases={total} matched={matched} mismatched={total - matched}")
    return 0 if matched == total else 1


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: mutate.py <poie-model-corpus-binary>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))
