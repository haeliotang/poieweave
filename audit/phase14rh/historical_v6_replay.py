#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Replay all 41 v6 claims in the detached historical worktree."""
from __future__ import annotations
import hashlib,json,os,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
HIST=Path("/tmp/n14rh-v6")
OUT=ROOT/"audit/phase14rh/historical-v6"
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 idx=json.loads((HIST/"docs/research/canonical-claim-certificate-index-v6.json").read_text())
 records=[]; failures=[]
 for entry in idx["certificates"]:
  cert=json.loads((HIST/entry["certificate"]).read_text())
  runner=cert["runner"]["binary"]; exp=cert["experiment"]["path"]
  expected=(HIST/cert["experiment"]["expected"]).read_bytes()
  binary=HIST/"target/debug"/runner
  result=subprocess.run([str(binary)],cwd=HIST,capture_output=True)
  ok=result.returncode==0 and result.stdout==expected
  if not ok: failures.append(entry["claim"])
  rec={"claim_id":entry["claim"],"certificate_id":cert["certificate_id"],"experiment":exp,"historical_commit":"161273d4599ff07fdd90f7a4be9e3bd6e221a94f","runner":runner,"runner_sha256":sha(binary),"model":cert["models"][0]["path"] if cert["models"] else None,"expected_sha256":hashlib.sha256(expected).hexdigest(),"actual_output_sha256":hashlib.sha256(result.stdout).hexdigest(),"exit_code":result.returncode,"assertion_result":"PASS" if ok else "FAIL","stdout_stderr_sha256":hashlib.sha256(result.stdout+result.stderr).hexdigest(),"replay_command":f"cd {HIST} && target/debug/{runner}","classification":"CLAIM_SPECIFIC_EXPERIMENT_REPLAY"}
  (OUT/"claims").mkdir(parents=True,exist_ok=True); (OUT/"claims"/f"{entry['claim']}.json").write_text(json.dumps(rec,indent=2)+"\n")
  if not ok: (OUT/f"{entry['claim']}-actual.txt").write_bytes(result.stdout); (OUT/f"{entry['claim']}-stderr.txt").write_bytes(result.stderr)
  records.append(rec)
 summary={"schema":"poie.s5v-n14rh-v6-replay/1","historical_commit":"161273d4599ff07fdd90f7a4be9e3bd6e221a94f","engine_sha256":sha(HIST/"crates/poie-viability/src/engine.rs"),"cargo_lock_sha256":sha(HIST/"Cargo.lock"),"compiler":subprocess.run(["rustc","--version"],capture_output=True,text=True).stdout.strip(),"platform":sys.platform,"claims":len(records),"passed":len(records)-len(failures),"failed":failures,"records":records}
 (OUT/"replay-summary.json").write_text(json.dumps(summary,indent=2)+"\n")
 print(f"v6 replay: {summary['passed']}/{summary['claims']} passed; failed={failures}")
 return 0 if not failures else 1
if __name__=="__main__": raise SystemExit(main())
