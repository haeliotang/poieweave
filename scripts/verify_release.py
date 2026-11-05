#!/usr/bin/env python3
"""Verify a Poieweave GitHub Release asset set without network access.

Checks:
- SHA256SUMS covers exactly the release artifacts (minus SHA256SUMS itself)
- every digest matches the file on disk
- artifact names carry the expected version
- optional macOS app signature status is reported, not trusted
- no obvious secret material is present in file names

The receipt is JSON; exit code 1 on any hard failure. This is evidence, not a
claim of signing/notarization or reproducibility.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_sums(text: str) -> dict[str, str]:
    entries: dict[str, str] = {}
    for line in text.splitlines():
        line = line.strip()
        if not line:
            continue
        parts = line.split()
        if len(parts) != 2 or len(parts[0]) != 64:
            raise ValueError(f"malformed SHA256SUMS line: {line!r}")
        name = Path(parts[1].lstrip("./")).name
        if name in entries and entries[name] != parts[0].lower():
            raise ValueError(f"duplicate basename in SHA256SUMS: {name}")
        entries[name] = parts[0].lower()
    return entries


def macos_signature(path: Path) -> dict[str, object]:
    if not path.exists():
        return {"present": False, "status": "missing"}
    try:
        result = subprocess.run(
            ["codesign", "-dv", "--verbose=2", str(path)],
            capture_output=True,
            text=True,
            timeout=30,
            check=False,
        )
    except (OSError, subprocess.TimeoutExpired) as error:
        return {"present": True, "status": "unknown", "detail": str(error)}
    text = (result.stderr or "") + (result.stdout or "")
    if "code object is not signed at all" in text:
        status = "unsigned"
    elif "adhoc" in text or "Signature=adhoc" in text:
        status = "ad-hoc"
    elif result.returncode == 0:
        status = "signed"
    else:
        status = "unknown"
    return {"present": True, "status": status, "detail": text.strip().splitlines()[:8]}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--artifacts", required=True, type=Path)
    parser.add_argument("--sums", type=Path)
    parser.add_argument("--expected-version", required=True)
    parser.add_argument("--tag", default=None)
    parser.add_argument("--macos-app", type=Path, default=None)
    parser.add_argument("--output", type=Path, default=None)
    args = parser.parse_args(argv)

    failures: list[str] = []
    warnings: list[str] = []
    artifacts = args.artifacts
    sums_path = args.sums or artifacts / "SHA256SUMS"

    if not artifacts.is_dir():
        failures.append(f"artifacts directory missing: {artifacts}")
        entries: dict[str, str] = {}
    else:
        files = sorted(
            p.name for p in artifacts.rglob("*") if p.is_file() and p.name != "SHA256SUMS"
        )
        try:
            entries = parse_sums(sums_path.read_text(encoding="utf-8"))
        except (OSError, UnicodeDecodeError, ValueError) as error:
            failures.append(f"SHA256SUMS unreadable: {error}")
            entries = {}
            files = []
        sums_names = set(entries)
        file_names = set(files)
        for missing in sorted(sums_names - file_names):
            failures.append(f"SHA256SUMS names missing artifact: {missing}")
        for extra in sorted(file_names - sums_names):
            failures.append(f"artifact missing from SHA256SUMS: {extra}")
        for name, expected in sorted(entries.items()):
            path = artifacts / name
            if not path.is_file():
                continue
            actual = sha256(path)
            if actual != expected:
                failures.append(f"digest mismatch: {name}")
            if args.expected_version not in name:
                warnings.append(f"artifact name does not contain {args.expected_version}: {name}")
            if any(token in name.lower() for token in ("secret", "private", "key", ".env")):
                failures.append(f"artifact name looks sensitive: {name}")

    signature = macos_signature(args.macos_app) if args.macos_app else {"present": False, "status": "not_checked"}
    receipt = {
        "schema": "poie.release-verification/0.1",
        "tag": args.tag,
        "expectedVersion": args.expected_version,
        "artifacts": sorted(set(entries)),
        "sha256Sums": len(entries),
        "macosSignature": signature,
        "failures": failures,
        "warnings": warnings,
        "ok": not failures,
    }
    text = json.dumps(receipt, ensure_ascii=False, indent=2, sort_keys=True) + "\n"
    if args.output:
        args.output.write_text(text, encoding="utf-8")
    sys.stdout.write(text)
    return 0 if receipt["ok"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
