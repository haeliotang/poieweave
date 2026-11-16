#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent FMI 3.0 importer check using fmpy (not the project harness)."""

from __future__ import annotations

import argparse
import hashlib
import shutil
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fmu", default="external/fmu-pv-battery/poie-pv-battery.fmu")
    parser.add_argument("--step", type=int, default=3600)
    parser.add_argument("--output", default="audit/phase2/results/fmpy-3600.csv")
    parser.add_argument("--log", default="audit/phase2/logs/fmpy.log")
    args = parser.parse_args()
    try:
        import fmpy
        from fmpy.fmi3 import FMU3Slave
    except Exception as error:  # pragma: no cover - environment dependent
        print(f"fmpy unavailable: {error}")
        return 2
    fmu_path = Path(args.fmu)
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    unzipdir = fmpy.extract(str(fmu_path))
    lines = [f"fmpy_version={getattr(fmpy, '__version__', 'unknown')}", f"platform_tuple={fmpy.platform_tuple}",
             f"fmu={fmu_path}", f"fmu_sha256={hashlib.sha256(fmu_path.read_bytes()).hexdigest()}"]
    try:
        fmu = FMU3Slave(
            guid="{9a4f79b0-5d4b-4f4e-a6b1-s5vchallenge01}",
            modelIdentifier="poie_pv_battery",
            unzipDirectory=unzipdir,
        )
        fmu.instantiate()
        fmu.setFloat64([3, 4, 5, 6, 7, 8, 9], [2500.0, 3000.0, 720e6, 10000.0, 144e6, 1.8e6, 10800.0])
        fmu.enterInitializationMode()
        fmu.exitInitializationMode()
        initial = fmu.getFloat64([1, 2, 10, 11, 12, 13, 14])
        rows = [initial + [0]]
        current = 0.0
        for _ in range(400):
            event, terminated, early, last = fmu.doStep(current, args.step)
            current = last if terminated else current + args.step
            values = fmu.getFloat64([1, 2, 10, 11, 12, 13, 14])
            rows.append(values + [1 if terminated else 0])
            if terminated:
                break
        fmu.terminate()
        fmu.freeInstance()
    finally:
        shutil.rmtree(unzipdir, ignore_errors=True)
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w") as handle:
        handle.write("time_s,energy_J,generated_J,load_J,repair_J,curtailed_J,depleted,terminated\n")
        for row in rows:
            handle.write(",".join(f"{value:.9f}" for value in row) + "\n")
    last = rows[-1]
    lines += [f"rows={len(rows)}", f"terminal_time_s={last[0]}", f"terminal_energy_J={last[1]}",
              f"terminated={last[7]}", f"output={output}"]
    Path(args.log).parent.mkdir(parents=True, exist_ok=True)
    Path(args.log).write_text("\n".join(lines) + "\n")
    if abs(last[0] - 284400.0) > 1e-6 or abs(last[1]) > 1.0 or last[7] != 1:
        print("fmpy importer FAILED endpoint check")
        return 1
    print(f"ok fmpy importer: {len(rows)} rows, terminal {last[0]} s")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
