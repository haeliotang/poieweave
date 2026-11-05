#!/usr/bin/env python3
"""Generate a non-secret signing recovery card.

The card records *where* the encrypted signing archive and passphrase live; it
must never contain the archive bytes, password, private key, or keychain secret.
It is a checklist for a recovery drill, not proof that recovery works.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path

CARD_SCHEMA = "poie.signing-custody-card/0.1"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def render(archive: Path, digest: str, keychain_service: str, keychain_account: str, passphrase_location: str, created: str) -> str:
    return f"""# Signing recovery card

- schema: `{CARD_SCHEMA}`
- generated: {created}
- archive: `{archive.name}`
- archive sha256: `{digest}`
- keychain service: `{keychain_service}`
- keychain account: `{keychain_account}`
- offline passphrase location: {passphrase_location}

## Rule

The archive and the passphrase must be stored separately. Losing the passphrase
makes the archive useless; losing the archive can be recovered only if the
passphrase and a fresh archive export are available.

## Quarterly drill

1. Confirm the archive digest above matches the file in the off-device copy.
2. Read the passphrase from the offline location (not from chat, repo, logs, or screenshots).
3. Run the project's recovery drill script against a throwaway artifact.
4. Record the date and result below. Only `verified: true` counts.

## Validation log

| date | result | operator | notes |
| --- | --- | --- | --- |
|  |  |  |  |

## Explicit boundaries

- This card contains no password, private key, or archive payload.
- The card being present does not prove the archive is recoverable.
- Rotate/re-export after a machine change, passphrase move, key rotation, or channel change.
"""


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--archive", required=True, type=Path)
    parser.add_argument("--keychain-service", required=True)
    parser.add_argument("--keychain-account", required=True)
    parser.add_argument("--passphrase-location", required=True)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--expected-sha256", default=None)
    parser.add_argument("--created", default="")
    args = parser.parse_args(argv)

    if not args.archive.is_file():
        print(json.dumps({"ok": False, "error": "archive_missing"}), file=sys.stderr)
        return 1
    digest = sha256(args.archive)
    if args.expected_sha256 and digest != args.expected_sha256.lower():
        print(json.dumps({"ok": False, "error": "digest_mismatch", "actual": digest}), file=sys.stderr)
        return 1
    card = render(
        args.archive,
        digest,
        args.keychain_service,
        args.keychain_account,
        args.passphrase_location,
        args.created,
    )
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(card, encoding="utf-8")
    print(json.dumps({"ok": True, "card": str(args.output), "archiveSha256": digest}, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
