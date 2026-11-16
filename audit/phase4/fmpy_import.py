#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent fmpy execution of the N4 FMU for one scenario."""

from __future__ import annotations

import argparse
import hashlib
import shutil
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fmu", default="external/fmu-n4-pv-battery/poie-n4-pv-battery.fmu")
    parser.add_argument("--step", type=int, required=True)
    parser.add_argument("--failure", type=float, default=-1.0)
    parser.add_argument("--stop", type=float, default=72000.0)
    parser.add_argument("--output", required=True)
    parser.add_argument("--log", required=True)
    args = parser.parse_args()
    try:
        import fmpy
        from fmpy.fmi3 import FMU3Slave
    except Exception as error:
        print(f"fmpy unavailable: {error}")
        return 2
    fmu_path = Path(args.fmu)
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    unzipdir = fmpy.extract(str(fmu_path))
    lines = [f"fmpy_version={getattr(fmpy, '__version__', 'unknown')}", f"platform_tuple={fmpy.platform_tuple}",
             f"fmu_sha256={hashlib.sha256(fmu_path.read_bytes()).hexdigest()}",
             f"failure_s={args.failure}", f"step_s={args.step}"]
    try:
        fmu = FMU3Slave(guid="{b6c1a8e2-3f4d-4a9b-8c2e-n4challenge0001}", modelIdentifier="poie_n4_pv_battery",
                        unzipDirectory=unzipdir)
        fmu.instantiate()
        fmu.setFloat64([3, 4, 5, 6, 7, 8, 9, 15],
                       [2500.0, 3000.0, 720e6, 10000.0, 144e6, 1.8e6, 10800.0, args.failure])
        fmu.enterInitializationMode()
        fmu.exitInitializationMode()
        vrs = [1, 2, 10, 11, 12, 13, 14, 16, 17, 18, 19, 20]
        rows = [fmu.getFloat64(vrs) + [0]]
        current = 0.0
        while current < args.stop - 1e-9:
            step_now = min(args.step, args.stop - current)
            _, terminated, _, last = fmu.doStep(current, step_now)
            current = last if terminated else current + step_now
            rows.append(fmu.getFloat64(vrs) + [1 if terminated else 0])
            if terminated:
                break
        fmu.terminate()
        fmu.freeInstance()
    finally:
        shutil.rmtree(unzipdir, ignore_errors=True)
    with output.open("w") as handle:
        handle.write("time_s,energy_J,generated_J,load_J,repair_J,curtailed_J,depleted,pvAvailable,failureAppliedTime_s,repairCompletedTime_s,lastEventCode,lastEventTime_s,terminated\n")
        for row in rows:
            handle.write(",".join(f"{value:.9f}" for value in row) + "\n")
    last = rows[-1]
    lines += [f"rows={len(rows)}", f"terminal_time_s={last[0]}", f"terminal_energy_J={last[1]}",
              f"generated_J={last[2]}", f"failure_applied_s={last[8]}", f"terminated={last[12]}"]
    Path(args.log).parent.mkdir(parents=True, exist_ok=True)
    Path(args.log).write_text("\n".join(lines) + "\n")
    print(f"ok fmpy N4: rows={len(rows)} terminal={last[0]} failure={last[8]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
