#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Run the N4 FMUs with an explicit sampling schedule and exact event-time splits."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
from pathlib import Path

PV_VRS = [1, 2, 10, 11, 12, 13, 14, 16, 17, 18, 19, 20]
REPAIR_VRS = [1, 2, 10, 11, 12, 13, 14, 16, 17, 18, 19, 20, 22, 23, 24]
PV_HEADER = ("time_s,energy_J,generated_J,load_J,repair_J,curtailed_J,depleted,pvAvailable,"
             "failureAppliedTime_s,repairCompletedTime_s,lastEventCode,lastEventTime_s,terminated")
REPAIR_HEADER = ("time_s,energy_J,generated_J,load_J,repair_J,curtailed_J,depleted,pvAvailable,"
                 "failureAppliedTime_s,repairCompletedTime_s,lastEventCode,lastEventTime_s,"
                 "repairState,robotAvailable,robotFailureAppliedTime_s,terminated")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--kind", choices=["pv", "repair"], required=True)
    parser.add_argument("--schedule", required=True)
    parser.add_argument("--step", type=float, required=True)
    parser.add_argument("--failure", type=float, default=-1.0)
    parser.add_argument("--robot-failure", type=float, default=-1.0)
    parser.add_argument("--output", required=True)
    parser.add_argument("--log", required=True)
    parser.add_argument("--metadata", required=True)
    args = parser.parse_args()
    import fmpy
    from fmpy.fmi3 import FMU3Slave
    schedule = sorted({float(value) for value in args.schedule.split(",")})
    fmu_path = (Path("external/fmu-n4-pv-battery/poie-n4-pv-battery.fmu") if args.kind == "pv"
                else Path("external/fmu-n4-repair/poie-n4-repair.fmu"))
    unzipdir = fmpy.extract(str(fmu_path))
    rows = []
    actual_steps = []
    event_split_times = []
    terminated = False
    try:
        if args.kind == "pv":
            fmu = FMU3Slave(guid="{b6c1a8e2-3f4d-4a9b-8c2e-n4challenge0001}", modelIdentifier="poie_n4_pv_battery",
                            unzipDirectory=unzipdir)
            vrs = PV_VRS
        else:
            fmu = FMU3Slave(guid="{c7d2b9f3-4a5e-4b0c-9d3f-n4repair00001}", modelIdentifier="poie_n4_repair",
                            unzipDirectory=unzipdir)
            vrs = REPAIR_VRS
        fmu.instantiate()
        if args.kind == "pv":
            fmu.setFloat64([3, 4, 5, 6, 7, 8, 9, 15],
                           [2500.0, 3000.0, 720e6, 10000.0, 144e6, 1.8e6, 10800.0, args.failure])
        else:
            fmu.setFloat64([3, 4, 5, 6, 7, 8, 9, 15, 21],
                           [2500.0, 3000.0, 720e6, 10000.0, 144e6, 1.8e6, 10800.0, -1.0, args.robot_failure])
        fmu.enterInitializationMode()
        fmu.exitInitializationMode()
        rows.append(list(fmu.getFloat64(vrs)) + [0.0])
        current = 0.0
        for target in schedule:
            if target <= current + 1e-9:
                continue
            while current < target - 1e-9:
                step_now = min(args.step, target - current)
                _, terminated, _, last = fmu.doStep(current, step_now)
                current = last if terminated else current + step_now
                actual_steps.append(step_now)
                if abs(current - target) <= 1e-9:
                    break
            if target in (args.failure, args.robot_failure) and abs(current - target) <= 1e-9:
                event_split_times.append(current)
            rows.append(list(fmu.getFloat64(vrs)) + [1.0 if terminated else 0.0])
            if terminated:
                break
        fmu.terminate()
        fmu.freeInstance()
    finally:
        shutil.rmtree(unzipdir, ignore_errors=True)
    header = PV_HEADER if args.kind == "pv" else REPAIR_HEADER
    with Path(args.output).open("w") as handle:
        handle.write(header + "\n")
        for row in rows:
            handle.write(",".join(f"{value:.9f}" for value in row) + "\n")
    metadata = {
        "kind": args.kind, "fmu": str(fmu_path),
        "fmu_sha256": hashlib.sha256(fmu_path.read_bytes()).hexdigest(),
        "nominal_communication_step_s": args.step,
        "sampling_schedule_s": schedule, "actual_steps": actual_steps,
        "max_actual_step_s": max(actual_steps) if actual_steps else 0.0,
        "event_split_times_s": event_split_times, "rows": len(rows),
    }
    Path(args.metadata).write_text(json.dumps(metadata, indent=2) + "\n")
    Path(args.log).write_text(
        f"fmpy_version={getattr(fmpy, '__version__', 'unknown')}\n"
        f"fmu_sha256={metadata['fmu_sha256']}\nrows={len(rows)}\n"
        f"nominal_step_s={args.step}\nschedule={schedule}\n")
    print(f"ok fmpy {args.kind}: rows={len(rows)} last={rows[-1][0]}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
