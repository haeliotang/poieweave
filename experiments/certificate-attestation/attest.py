#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Detached-signature plumbing test for the bundle.

By default an ephemeral test key is generated. `--key` selects an
operator-provided private key, which exercises the same path a persistent
release key would use; the key itself is never copied or logged. This proves
signing and tamper detection, not a trust anchor or third-party attestation.
"""

from __future__ import annotations

import argparse
import hashlib
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BUNDLE = ROOT / "experiments/certificate-bundle/bundle.json"


def run(*args: str) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(args, capture_output=True)


def keypair(directory: Path) -> tuple[Path, Path]:
    key = directory / "key.pem"
    public_key = directory / "pub.pem"
    for command in (
        ("openssl", "genpkey", "-algorithm", "RSA", "-pkeyopt", "rsa_keygen_bits:2048", "-out", str(key)),
        ("openssl", "pkey", "-in", str(key), "-pubout", "-out", str(public_key)),
    ):
        result = run(*command)
        if result.returncode != 0:
            raise RuntimeError(result.stderr.decode(errors="replace"))
    return key, public_key


def public_key_for(private_key: Path, directory: Path) -> Path:
    public_key = directory / "pub.pem"
    result = run("openssl", "pkey", "-in", str(private_key), "-pubout", "-out", str(public_key))
    if result.returncode != 0:
        raise RuntimeError(result.stderr.decode(errors="replace"))
    return public_key


def sign(key: Path, digest_path: Path, signature: Path) -> bool:
    return run("openssl", "dgst", "-sha256", "-sign", str(key), "-out", str(signature), str(digest_path)).returncode == 0


def verify(public_key: Path, digest_path: Path, signature: Path) -> bool:
    return (
        run(
            "openssl",
            "dgst",
            "-sha256",
            "-verify",
            str(public_key),
            "-signature",
            str(signature),
            str(digest_path),
        ).returncode
        == 0
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--key", type=Path, default=None, help="operator-provided private key (optional)")
    args = parser.parse_args(argv)
    if not BUNDLE.is_file():
        print("missing bundle", file=sys.stderr)
        return 1
    digest = hashlib.sha256(BUNDLE.read_bytes()).hexdigest()
    with tempfile.TemporaryDirectory() as raw:
        directory = Path(raw)
        if args.key is not None:
            if not args.key.is_file():
                print("operator key missing", file=sys.stderr)
                return 1
            key = args.key
            public_key = public_key_for(key, directory)
        else:
            key, public_key = keypair(directory)
        digest_path = directory / "bundle.sha256"
        signature = directory / "sig.bin"
        digest_path.write_text(digest + "\n", encoding="utf-8")
        if not sign(key, digest_path, signature):
            print("signing failed", file=sys.stderr)
            return 1
        if not verify(public_key, digest_path, signature):
            print("clean verification failed", file=sys.stderr)
            return 1
        digest_path.write_text(hashlib.sha256(b"tampered").hexdigest() + "\n", encoding="utf-8")
        if verify(public_key, digest_path, signature):
            print("tampered digest unexpectedly verified", file=sys.stderr)
            return 1
    print("attestation=ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
