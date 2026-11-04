#!/usr/bin/env python3
"""Check the deterministic agent eval fixture matrix."""
import pathlib, sys

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore

ROOT = pathlib.Path(__file__).resolve().parent.parent
EVALS = ROOT / "evals" / "agent"
TOOLS = {"catalog.describe", "state.query", "plan.propose", "plan.preview_sap", "action.draft", "answer.emit"}
EXPECTED = 72
REQUIRED = {"schema", "id", "preregistered_at", "prompt", "profile", "script", "expect"}


def main() -> int:
    if not EVALS.exists():
        print("agent eval fixtures are missing")
        return 1
    problems, seen, covered, approvals = [], set(), set(), set()
    files = sorted(EVALS.glob("*.toml"))
    for path in files:
        try:
            data = tomllib.loads(path.read_text(encoding="utf-8"))
        except Exception as error:
            problems.append(f"{path.name}: {error}"); continue
        missing = REQUIRED - set(data)
        if missing: problems.append(f"{path.name}: missing {sorted(missing)}")
        if data.get("schema") != "poie.agent-eval/0.1": problems.append(f"{path.name}: schema")
        if path.stem in seen: problems.append(f"{path.name}: duplicate id")
        seen.add(path.stem)
        responses = data.get("script", {}).get("responses", [])
        if not isinstance(responses, list) or not responses:
            problems.append(f"{path.name}: responses"); continue
        tools = [entry.get("tool") for entry in responses]
        for tool in tools:
            if tool not in TOOLS: problems.append(f"{path.name}: unknown tool {tool}")
            else: covered.add(tool)
        approval = data.get("approval")
        if approval is not None:
            if tools != ["action.draft"]: problems.append(f"{path.name}: approval outside action.draft")
            if set(approval) != {"action"} or approval.get("action") not in {"approve", "reject", "expire"}:
                problems.append(f"{path.name}: invalid approval")
            else: approvals.add(approval["action"])
        expect = data.get("expect", {})
        if "error" in expect and not isinstance(expect["error"], str):
            problems.append(f"{path.name}: error must be string")
    if len(files) != EXPECTED: problems.append(f"expected {EXPECTED} fixtures, found {len(files)}")
    if covered != TOOLS: problems.append(f"missing tools: {sorted(TOOLS - covered)}")
    if approvals != {"approve", "reject", "expire"}: problems.append(f"missing approval coverage: {sorted(approvals)}")
    if problems:
        print("\n".join(problems)); return 1
    print(f"agent evals OK ({len(files)} fixtures, 6 tools, approval approve/reject/expire)")
    return 0

if __name__ == "__main__":
    sys.exit(main())
