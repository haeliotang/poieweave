#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent fmpy execution of the N5 scheduler FMU."""

from __future__ import annotations

import argparse
import hashlib
import shutil
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fmu", default="external/fmu-n5-scheduler/poie-n5-scheduler.fmu")
    parser.add_argument("--step", type=int, default=900)
    parser.add_argument("--task-mode", type=float, required=True)
    parser.add_argument("--robots", type=float, required=True)
    parser.add_argument("--failure-time", type=float, default=-1.0)
    parser.add_argument("--failure-robot", type=float, default=0.0)
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
    unzipdir = fmpy.extract(str(fmu_path))
    try:
        fmu = FMU3Slave(guid="{d8e3c0a4-5b6f-4c1d-8e4a-n5scheduler001}", modelIdentifier="poie_n5_scheduler",
                        unzipDirectory=unzipdir)
        fmu.instantiate()
        fmu.setFloat64([2, 3, 4, 5, 6],
                       [args.task_mode, args.robots, args.failure_time, args.failure_robot, args.stop])
        fmu.enterInitializationMode()
        fmu.exitInitializationMode()
        vrs = [1, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22]
        rows = [fmu.getFloat64(vrs) + [0]]
        current = 0.0
        while current < args.stop - 1e-9:
            step = min(args.step, args.stop - current)
            _, terminated, _, last = fmu.doStep(current, step)
            current = last if terminated else current + step
            rows.append(fmu.getFloat64(vrs) + [1 if terminated else 0])
            if terminated:
                break
        fmu.terminate()
        fmu.freeInstance()
    finally:
        shutil.rmtree(unzipdir, ignore_errors=True)
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("w") as handle:
        handle.write("time_s,makespan_s,completedCount,interrupted,failureAppliedTime_s,t1Start_s,t1End_s,t1Robot,"
                     "t2Start_s,t2End_s,t2Robot,pumpRestored,latheRestored,lastEventCode,terminated\n")
        for row in rows:
            handle.write(",".join(f"{value:.9f}" for value in row) + "\n")
    last = rows[-1]
    Path(args.log).parent.mkdir(parents=True, exist_ok=True)
    Path(args.log).write_text(
        f"fmpy_version={getattr(fmpy, '__version__', 'unknown')}\n"
        f"fmu_sha256={hashlib.sha256(fmu_path.read_bytes()).hexdigest()}\n"
        f"rows={len(rows)}\nterminal_time_s={last[0]}\nmakespan_s={last[1]}\n"
        f"completed={last[2]}\ninterrupted={last[3]}\nfailure_applied_s={last[4]}\n"
    )
    print(f"ok fmpy N5: rows={len(rows)} terminal={last[0]} makespan={last[1]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
