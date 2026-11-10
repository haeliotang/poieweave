#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Verify that Apache-2.0 contract crates only pull permissive dependencies.

`cargo deny` uses one workspace-wide allow list, so it can permit a GPL/AGPL
dependency that would make an Apache-2.0 contract crate impossible to use in a
closed product. This check walks the dependency closure of the Apache crates
and rejects every licence outside a deliberately narrow permissive set.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from dataclasses import dataclass

ROOTS = (
    "poie-ontology",
    "poie-objects",
    "poie-digest",
    "poie-sdkgen",
)

PERMISSIVE = {
    "0BSD",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "BSL-1.0",
    "CC0-1.0",
    "ISC",
    "MIT",
    "MIT-0",
    "Unicode-3.0",
    "Unicode-DFS-2016",
    "Unlicense",
    "Zlib",
}
PERMISSIVE_EXCEPTIONS = {"LLVM-exception"}
TOKEN = re.compile(r"\(|\)|AND|OR|WITH|[A-Za-z0-9.+-]+")


class ParseError(ValueError):
    pass


@dataclass
class Expression:
    tokens: list[str]
    pos: int = 0

    def peek(self) -> str | None:
        return self.tokens[self.pos] if self.pos < len(self.tokens) else None

    def take(self, expected: str | None = None) -> str:
        token = self.peek()
        if token is None or (expected is not None and token != expected):
            raise ParseError(f"expected {expected or 'token'}, got {token!r}")
        self.pos += 1
        return token

    def parse_or(self) -> bool:
        value = self.parse_and()
        while self.peek() == "OR":
            self.take("OR")
            value = self.parse_and() or value
        return value

    def parse_and(self) -> bool:
        value = self.parse_factor()
        while self.peek() == "AND":
            self.take("AND")
            value = self.parse_factor() and value
        return value

    def parse_factor(self) -> bool:
        if self.peek() == "(":
            self.take("(")
            value = self.parse_or()
            self.take(")")
            return value
        name = self.take()
        if name in {"AND", "OR", "WITH", ")"}:
            raise ParseError(f"unexpected {name}")
        if self.peek() == "WITH":
            self.take("WITH")
            exception = self.take()
            return name in PERMISSIVE and exception in PERMISSIVE_EXCEPTIONS
        return name in PERMISSIVE


def allowed(expression: str | None) -> bool:
    if not expression:
        return False
    text = expression.replace("/", " OR ")
    tokens = TOKEN.findall(text)
    if not tokens:
        return False
    parser = Expression(tokens)
    try:
        value = parser.parse_or()
    except ParseError:
        return False
    return parser.pos == len(tokens) and value


def main() -> int:
    try:
        raw = subprocess.check_output(
            ["cargo", "metadata", "--format-version", "1", "--offline"],
            text=True,
        )
    except (OSError, subprocess.CalledProcessError) as error:
        print(f"check-apache-licenses: cargo metadata failed: {error}", file=sys.stderr)
        return 2
    metadata = json.loads(raw)
    packages = {package["id"]: package for package in metadata["packages"]}
    by_name = {package["name"]: package for package in metadata["packages"]}
    nodes = {node["id"]: node for node in metadata["resolve"]["nodes"]}

    missing = [name for name in ROOTS if name not in by_name]
    if missing:
        print(f"check-apache-licenses: missing roots: {', '.join(missing)}", file=sys.stderr)
        return 2

    seen: set[str] = set()
    problems: list[str] = []
    queue = [by_name[name]["id"] for name in ROOTS]
    while queue:
        package_id = queue.pop()
        if package_id in seen:
            continue
        seen.add(package_id)
        package = packages[package_id]
        if not allowed(package.get("license")):
            problems.append(
                f"{package['name']} {package.get('version', '?')}: "
                f"{package.get('license') or 'NO LICENSE FIELD'}"
            )
        node = nodes.get(package_id)
        if node is None:
            continue
        for dependency in node.get("deps", []):
            queue.append(dependency["pkg"])

    if problems:
        print("Apache contract closure contains non-permissive licences:", file=sys.stderr)
        for problem in sorted(set(problems)):
            print(f"  {problem}", file=sys.stderr)
        return 1
    print(f"apache-license closure ok: {len(seen)} packages checked from {', '.join(ROOTS)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
