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
ERROR_CODES = {
    "invalid_tool_arguments", "budget_exceeded", "unauthorized", "approval_rejected",
    "approval_expired", "citation_missing", "unsupported_schema", "not_implemented",
}
REQUIRED_ARGS = {
    "catalog.describe": set(),
    "state.query": {"subject"},
    "plan.propose": {"goal", "horizonS"},
    "plan.preview_sap": {"planFingerprint"},
    "action.draft": {"planFingerprint", "actionId"},
    "answer.emit": {"claims"},
}


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
        responses = data.get("script", {}).get("responses", [])
        if not isinstance(responses, list) or not responses:
            problems.append(f"{path.name}: script.responses must be a non-empty array")
            continue
        tools = [entry.get("tool") for entry in responses if entry.get("tool")]
        for tool in tools:
            if tool not in TOOLS:
                problems.append(f"{path.name}: unknown tool {tool}")
            covered.add(tool)
        approval = data.get("approval")
        if approval is not None:
            if tools != ["action.draft"]:
                problems.append(f"{path.name}: approval is only valid for action.draft")
            approval_actions.add(approval.get("action"))
            if set(approval) != {"action"} or approval.get("action") not in {"approve", "reject", "expire"}:
                problems.append(f"{path.name}: invalid approval table {approval!r}")
        expect = data.get("expect", {})
        if "error" in expect and expect["error"] not in ERROR_CODES:
            problems.append(f"{path.name}: unknown expected error {expect['error']!r}")
        if expect.get("error") == "approval_rejected" and not (approval and approval.get("action") == "reject"):
            problems.append(f"{path.name}: approval_rejected without reject script")
        if expect.get("error") == "approval_expired" and not (approval and approval.get("action") == "expire"):
            problems.append(f"{path.name}: approval_expired without expire script")
        for entry in responses:
            tool = entry.get("tool")
            if tool not in REQUIRED_ARGS:
                continue
            args = entry.get("args", {})
            if not isinstance(args, dict):
                problems.append(f"{path.name}: args must be a table")
                continue
            if "bad-args" not in path.stem and "bad-schema" not in path.stem and "tool-error" not in path.stem:
                missing = REQUIRED_ARGS[tool] - set(args)
                if missing:
                    problems.append(f"{path.name}: missing args {sorted(missing)} for {tool}")
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
