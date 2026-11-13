#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Independent verifier for exported S3 recovery receipts.

Reimplements poie-digest's canonical JSON (sorted keys, compact separators,
zero canonicalised to 0) and the receipt domain separator in Python. This is a
different implementation, not the Rust verifier.
"""

from __future__ import annotations

import hashlib
import json
import sys


def canonical(value: object) -> str:
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, float):
        if value == 0.0:
            return "0"
        return json.dumps(value, allow_nan=False)
    if isinstance(value, int):
        return str(value)
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False)
    if isinstance(value, list):
        return "[" + ",".join(canonical(item) for item in value) + "]"
    if isinstance(value, dict):
        keys = sorted(value)
        return "{" + ",".join(json.dumps(key) + ":" + canonical(value[key]) for key in keys) + "}"
    raise TypeError(f"unsupported JSON value: {type(value)!r}")


def verify(receipt: dict) -> bool:
    body = {key: value for key, value in receipt.items() if key != "digest"}
    encoded = canonical(body)
    digest = hashlib.sha256(("poie.recovery.receipt/1\n" + encoded).encode()).hexdigest()
    return digest == receipt.get("digest")


def main(path: str) -> int:
    with open(path, encoding="utf-8") as handle:
        receipts = json.load(handle)
    if not isinstance(receipts, list):
        print("ERROR: expected a JSON array of receipts", file=sys.stderr)
        return 2
    verified = sum(1 for receipt in receipts if isinstance(receipt, dict) and verify(receipt))
    print(f"receipts={len(receipts)} verified={verified}")
    if verified != len(receipts) or not receipts:
        print("ERROR: not all receipts verified", file=sys.stderr)
        return 1
    print("OK")
    return 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("usage: verify.py <receipts.json>", file=sys.stderr)
        raise SystemExit(2)
    raise SystemExit(main(sys.argv[1]))
