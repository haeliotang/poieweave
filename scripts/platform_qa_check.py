#!/usr/bin/env python3
"""Validate a manual platform QA receipt.

The tool does not perform the QA. It rejects incomplete receipts and makes the
signing-status claim explicit so release notes cannot describe an ad-hoc or
unsigned build as signed/notarized.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

SCHEMA = "poie.platform-qa/0.1"
RECEIPT_SCHEMA = "poie.platform-qa-receipt/0.1"
SIGNING = {"signed", "notarized", "ad-hoc", "unsigned", "unknown"}
COMMON = ["version_consistency", "sha256sums", "release_notes_signing_status", "no_telemetry"]
PLATFORM = {
    "macos": ["install", "launch", "gatekeeper_notice", "dock_icon", "data_migration"],
    "windows": ["nsis_install", "webview2_launch", "smart_screen_notice", "app_data"],
    "linux": ["appimage_launch", "deb_dependencies", "window_decorations"],
}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args(argv)

    failures: list[str] = []
    warnings: list[str] = []
    try:
        data = json.loads(args.input.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        data = None
        failures.append(f"input_unreadable: {error}")

    receipt = {
        "schema": RECEIPT_SCHEMA,
        "input": str(args.input),
        "version": None,
        "platform": None,
        "signingStatus": None,
        "checks": [],
        "failures": failures,
        "warnings": warnings,
        "ok": False,
    }
    if data is not None:
        version = data.get("version")
        platform = data.get("platform")
        signing = data.get("signingStatus")
        receipt["version"] = version
        receipt["platform"] = platform
        receipt["signingStatus"] = signing
        if data.get("schema") != SCHEMA:
            failures.append(f"schema must be {SCHEMA}")
        if not isinstance(version, str) or not version:
            failures.append("version required")
        if signing not in SIGNING:
            failures.append(f"signingStatus must be one of {sorted(SIGNING)}")
        family = str(platform).split("-", 1)[0] if isinstance(platform, str) else ""
        required = COMMON + PLATFORM.get(family, [])
        if family not in PLATFORM:
            failures.append(f"unknown platform {platform!r}")
        checks = data.get("checks")
        if not isinstance(checks, list):
            failures.append("checks must be an array")
            checks = []
        by_id = {}
        for check in checks:
            if not isinstance(check, dict):
                failures.append("check must be an object")
                continue
            check_id = check.get("id")
            status = check.get("status")
            if not isinstance(check_id, str) or not check_id:
                failures.append("check.id required")
                continue
            if check_id in by_id:
                failures.append(f"duplicate check {check_id}")
                continue
            if status not in {"pass", "fail", "skip"}:
                failures.append(f"{check_id}: status must pass|fail|skip")
                continue
            if status == "skip":
                warnings.append(f"{check_id}: skipped")
            if status == "fail":
                failures.append(f"{check_id}: failed")
            by_id[check_id] = check
        for check_id in required:
            if check_id not in by_id:
                failures.append(f"missing required check {check_id}")
        for check_id, check in by_id.items():
            receipt["checks"].append({"id": check_id, "status": check.get("status"), "evidence": check.get("evidence", "")})
        if signing in {"signed", "notarized"} and any(
            "signing" in failure.lower() or "signature" in failure.lower() for failure in failures
        ):
            failures.append("signing claim contradicts failures")
        if signing == "unsigned":
            warnings.append("unsigned artifact: release notes must state this")

    receipt["ok"] = not failures
    text = json.dumps(receipt, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(text, encoding="utf-8")
    sys.stdout.write(text)
    return 0 if receipt["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
