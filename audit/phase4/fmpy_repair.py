#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent fmpy execution of the N4 repair-interruption FMU."""

from __future__ import annotations

import argparse
import hashlib
import shutil
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fmu", default="external/fmu-n4-repair/poie-n4-repair.fmu")
    parser.add_argument("--step", type=int, required=True)
    parser.add_argument("--robot-failure", type=float, default=7200.0)
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
    try:
        fmu = FMU3Slave(guid="{c7d2b9f3-4a5e-4b0c-9d3f-n4repair00001}", modelIdentifier="poie_n4_repair",
                        unzipDirectory=unzipdir)
        fmu.instantiate()
        fmu.setFloat64([3, 4, 5, 6, 7, 8, 9, 15, 21],
                       [2500.0, 3000.0, 720e6, 10000.0, 144e6, 1.8e6, 10800.0, -1.0, args.robot_failure])
        fmu.enterInitializationMode()
        fmu.exitInitializationMode()
        vrs = [1, 2, 10, 11, 12, 13, 14, 16, 17, 18, 19, 20, 22, 23, 24]
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
        handle.write("time_s,energy_J,generated_J,load_J,repair_J,curtailed_J,depleted,pvAvailable,"
                     "failureAppliedTime_s,repairCompletedTime_s,lastEventCode,lastEventTime_s,"
                     "repairState,robotAvailable,robotFailureAppliedTime_s,terminated\n")
        for row in rows:
            handle.write(",".join(f"{value:.9f}" for value in row) + "\n")
    last = rows[-1]
    Path(args.log).parent.mkdir(parents=True, exist_ok=True)
    Path(args.log).write_text(
        f"fmpy_version={getattr(fmpy, '__version__', 'unknown')}\n"
        f"fmu_sha256={hashlib.sha256(fmu_path.read_bytes()).hexdigest()}\n"
        f"rows={len(rows)}\nterminal_time_s={last[0]}\nterminal_energy_J={last[1]}\n"
        f"repair_state={last[12]}\nrobot_available={last[13]}\nrobot_failure_applied_s={last[14]}\n"
    )
    print(f"ok fmpy repair: rows={len(rows)} terminal={last[0]} repair_state={last[12]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
