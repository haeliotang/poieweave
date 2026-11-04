#!/usr/bin/env python3
"""Check that schema literals used by product-layer docs are registered.

The registry is `docs/design/SCHEMAS.md`. This is deliberately a small
documentation gate, not a runtime JSON-Schema validator.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
REGISTRY = ROOT / "docs" / "design" / "SCHEMAS.md"
SCHEMA = re.compile(r"`?(poie\.[a-z0-9-]+/\d+\.\d+)`?")


def main() -> int:
    text = REGISTRY.read_text(encoding="utf-8")
    registered = set(SCHEMA.findall(text))
    problems = []
    for path in (ROOT / "docs" / "design").rglob("*.md"):
        if path.name.startswith("VALIDATION") or path.name in {"DIGEST.md", "PLATFORM.zh-CN.md", "SCHEMAS.md"}:
            continue
        if path.parent.name == "research":
            continue
        for found in SCHEMA.findall(path.read_text(encoding="utf-8")):
            if found not in registered:
                problems.append(f"{path.relative_to(ROOT)}: unregistered schema {found}")
    for root in (ROOT / "crates" / "poie-objects" / "src", ROOT / "crates" / "poie-sdkgen" / "src", ROOT / "apps" / "desktop" / "src" / "sdk"):
        for path in root.rglob("*"):
            if path.suffix not in {".rs", ".ts"}:
                continue
            for found in SCHEMA.findall(path.read_text(encoding="utf-8")):
                if found not in registered:
                    problems.append(f"{path.relative_to(ROOT)}: unregistered schema {found}")
    if problems:
        for problem in problems:
            print(problem)
        return 1
    print(f"schema registry OK ({len(registered)} schemas)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
