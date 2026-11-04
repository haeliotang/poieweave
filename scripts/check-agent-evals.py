#!/usr/bin/env python3
"""Validate deterministic agent eval fixtures and their coverage matrix."""

import pathlib
import sys

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = pathlib.Path(__file__).resolve().parent.parent
EVALS = ROOT / "evals" / "agent"
TOOLS = {"catalog.describe", "state.query", "plan.propose", "plan.preview_sap", "action.draft", "answer.emit"}
REQUIRED = {"schema", "id", "preregistered_at", "prompt", "profile", "script", "expect"}
EXPECTED = 72


def main() -> int:
    if not EVALS.exists():
        print("agent eval fixtures are missing")
        return 1
    problems = []
    seen = set()
    covered = set()
    approval_actions = set()
    files = sorted(EVALS.glob("*.toml"))
    for path in files:
        try:
            data = tomllib.loads(path.read_text(encoding="utf-8"))
        except Exception as error:
            problems.append(f"{path.name}: {error}")
            continue
        missing = REQUIRED - set(data)
        if missing:
            problems.append(f"{path.name}: missing {sorted(missing)}")
        if data.get("schema") != "poie.agent-eval/0.1":
            problems.append(f"{path.name}: wrong schema")
        if path.stem in seen:
            problems.append(f"{path.name}: duplicate id")
        seen.add(path.stem)
        tools = [entry.get("tool") for entry in data.get("script", {}).get("responses", []) if entry.get("tool")]
        for tool in tools:
            if tool not in TOOLS:
                problems.append(f"{path.name}: unknown tool {tool}")
            covered.add(tool)
        approval = data.get("approval")
        if approval is not None:
            approval_actions.add(approval.get("action"))
            if approval.get("action") not in {"approve", "reject", "expire"}:
                problems.append(f"{path.name}: invalid approval action {approval.get('action')!r}")
    if len(files) < EXPECTED:
        problems.append(f"expected at least {EXPECTED} fixtures, found {len(files)}")
    missing_tools = TOOLS - covered
    if missing_tools:
        problems.append(f"missing tool coverage: {sorted(missing_tools)}")
    if approval_actions != {"approve", "reject", "expire"}:
        problems.append(f"missing approval script coverage: {sorted(approval_actions)}")
    if problems:
        for problem in problems:
            print(problem)
        return 1
    print(f"agent evals OK ({len(files)} fixtures)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
