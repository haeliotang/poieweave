#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent fmpy execution of the N12 identity-correct journal scheduler."""

from __future__ import annotations

import argparse
import hashlib
import shutil
from pathlib import Path

KEYS = ["taskCount", "t1Duration_s", "t1Priority", "t1RobotMask", "t1RequiresTool", "t2Duration_s", "t2Priority",
        "t2RobotMask", "t2RequiresTool", "t3Duration_s", "t3Priority", "t3RobotMask", "t3RequiresTool", "robotCount",
        "failureTime_s", "failureKind", "failureIndex", "horizon_s", "preferredRobot", "failureAssetId",
        "robot1DependencyMask", "robot2DependencyMask", "failureCount", "f2Time_s", "f2Kind", "f2Index", "f2AssetId", "f3Time_s", "f3Kind", "f3Index", "f3AssetId", "f4Time_s", "f4Kind", "f4Index", "f4AssetId", "releaseMode"]
VRS = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 43, 44, 45, 46, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73]
OUT = [1] + list(range(20, 43)) + list(range(43, 51))


def contract(path: Path) -> dict[str, float]:
    data = {}
    for line in path.read_text().splitlines():
        if "=" in line:
            key, value = line.split("=", 1)
            data[key.strip()] = float(value)
    return data


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fmu", default="external/fmu-n16-scheduler/poie-n16-scheduler.fmu")
    parser.add_argument("--contract", required=True)
    parser.add_argument("--step", type=int, default=900)
    parser.add_argument("--output", required=True)
    parser.add_argument("--log", required=True)
    args = parser.parse_args()
    try:
        import fmpy
        from fmpy.fmi3 import FMU3Slave
    except Exception as error:
        print(f"fmpy unavailable: {error}")
        return 2
    data = contract(Path(args.contract))
    fmu_path = Path(args.fmu)
    unzipdir = fmpy.extract(str(fmu_path))
    try:
        fmu = FMU3Slave(guid="{e9f4d1b5-6c7a-4d2e-9f5b-n16scheduler01}", modelIdentifier="poie_n16_scheduler",
                        unzipDirectory=unzipdir)
        fmu.instantiate()
        fmu.setFloat64(VRS, [data.get(key, {"preferredRobot":0.0,"failureAssetId":-1.0,"robot1DependencyMask":0.0,"robot2DependencyMask":0.0}.get(key, 0.0)) for key in KEYS])
        fmu.enterInitializationMode()
        fmu.exitInitializationMode()
        rows = [fmu.getFloat64(OUT) + [0]]
        current = 0.0
        stop = data["horizon_s"]
        while current < stop - 1e-9:
            step = min(args.step, stop - current)
            _, terminated, _, last = fmu.doStep(current, step)
            current = last if terminated else current + step
            rows.append(fmu.getFloat64(OUT) + [1 if terminated else 0])
            if terminated:
                break
        journal = fmu.getString([100])[0] if hasattr(fmu, "getString") else ""
        fmu.terminate()
        fmu.freeInstance()
    finally:
        shutil.rmtree(unzipdir, ignore_errors=True)
    Path(args.output).parent.mkdir(parents=True, exist_ok=True)
    with Path(args.output).open("w") as handle:
        handle.write("time_s,makespan_s,completedCount,interrupted,failureAppliedTime_s,t1Start_s,t1End_s,t1Robot,"
                     "t2Start_s,t2End_s,t2Robot,t3Start_s,t3End_s,t3Robot,restored1,restored2,restored3,lastEventCode,"
                     "toolAvailable,toolFailureAppliedTime_s,release_s,toolAcquired_s,toolReleased_s,toolTaskMask,"
                     "preferredRobot,failureAssetId,robot1DependencyMask,robot2DependencyMask,selectedDependencyMask,"
                     "failureTargetId,dependencyPathCode,affectedSelectedResource,terminated\n")
        for row in rows:
            handle.write(",".join(f"{value:.9f}" for value in row) + "\n")
    last = rows[-1]
    Path(args.log).parent.mkdir(parents=True, exist_ok=True)
    Path(args.log).write_text(
        f"fmpy_version={getattr(fmpy, '__version__', 'unknown')}\n"
        f"fmu_sha256={hashlib.sha256(fmu_path.read_bytes()).hexdigest()}\nrows={len(rows)}\n"
        f"terminal_time_s={last[0]}\nmakespan_s={last[1]}\ncompleted={last[2]}\ninterrupted={last[3]}\n"
    )
    jpath = Path(args.output + ".journal.json")
    if journal:
        jpath.write_text(journal + "\n")
    print(f"ok fmpy N16: rows={len(rows)} makespan={last[1]} journal={bool(journal)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
