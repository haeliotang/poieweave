#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""Issue v18 execution schedule extension certificates; v18 bytes are frozen."""
from __future__ import annotations
import hashlib,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent
INVENTORY=ROOT/"docs/research/s2-evidence-inventory.json"
V9_INDEX=ROOT/"docs/research/canonical-claim-certificate-index-v9.json"
V10_INDEX=ROOT/"docs/research/canonical-claim-certificate-index-v10.json"
V17_INDEX=ROOT/"docs/research/canonical-claim-certificate-index-v17.json"
V18_INDEX=ROOT/"docs/research/canonical-claim-certificate-index-v18.json"
N15_MANIFEST=ROOT/"audit/phase15/manifest.json"
RUNTIME=["crates/poie-viability/src/engine.rs","crates/poie-viability/src/survival.rs","crates/poie-viability/src/clock.rs","crates/poie-viability/src/closure.rs","crates/poie-viability/src/scheduler.rs","Cargo.toml","Cargo.lock"]
SEMANTICS=("canonical event engine with observed legacy semantics; explicit same-target repair-group extension reuses the same event loop, scheduler, dependency closure, inventory ledger and atomic commit; legacy run_engine outcomes unchanged; step-level release, replanning and autonomous recovery are not implemented")
VERIFIER="poie-canonical-s2-certificate/15"
def sha(p): return hashlib.sha256((ROOT/p).read_bytes()).hexdigest()
def env(paths): return {"files":[{"path":p,"sha256":sha(p)} for p in paths]}
def replay():
 r=subprocess.run(["python3","scripts/check-experiment-replays.py"],cwd=ROOT,capture_output=True,check=False)
 if r.returncode!=0: raise SystemExit("replay failed: "+r.stdout.decode(errors="replace")[-1000:])
 out=r.stdout+r.stderr
 return {"command":"python3 scripts/check-experiment-replays.py","output_sha256":hashlib.sha256(out).hexdigest(),"exit_code":r.returncode}
def main(argv):
 inv=json.loads(INVENTORY.read_text()); claims={c["id"]:c for c in inv["claims"]}; experiments={e["experiment"]:e for e in inv["experiments"]}
 v17=json.loads(V17_INDEX.read_text()); v17map={e["claim"]:e for e in v17["certificates"]}
 rep=replay(); native={"path":"audit/phase15/manifest.json","sha256":sha("audit/phase15/manifest.json")}
 index={"schema":"poie.canonical-claim-certificate-index/18","certificates":[]}; results={}
 for claim_id in sorted(v17map):
  claim=claims[claim_id]; entry=next(e for e in experiments.values() if claim_id in e["claim_ids"])
  expname=entry["experiment"]; runner=entry["runner_binaries"][0]; models=entry["models"]; pred=v17map[claim_id]
  paths=RUNTIME+[f"crates/poie-viability/src/bin/{runner}.rs",*[f"scenarios/{m}/model.toml" for m in models],f"experiments/{expname}/expected.txt",f"experiments/{expname}/REPORT.md",f"experiments/{expname}/PREREG.md"]
  cert={"schema":"poie.canonical-s2-certificate/19","certificate_id":f"poie-s2-canonical-{claim_id.lower()}-v18","version":18,"status":"CANONICAL_ISSUED_V18","successor_classification":"AUTHORIZATION_SEMANTIC_CORRECTION","claims":[claim_id],"claim_statement":claim["statement"],"claims_sha256":sha("docs/research/claims-ledger.zh-CN.md"),"canonical_classification":pred["classification"],"semantics":SEMANTICS + "; v18 records per-task completion at each task end","scope":f"model/simulation evidence only; canonical claim {claim_id}; explicit group extension does not alter legacy execution","engine":{"source":"crates/poie-viability/src/engine.rs","sha256":sha("crates/poie-viability/src/engine.rs")},"runner":{"binary":runner,"source":f"crates/poie-viability/src/bin/{runner}.rs","sha256":sha(f"crates/poie-viability/src/bin/{runner}.rs")},"experiment":{"path":f"experiments/{expname}","expected":f"experiments/{expname}/expected.txt","expected_sha256":sha(f"experiments/{expname}/expected.txt")},"models":[{"path":f"scenarios/{m}/model.toml","sha256":sha(f"scenarios/{m}/model.toml")} for m in models],"source_environment":env(paths),"native_task_resource_observatory":native,"replay":rep,"predecessor":{"path":pred["certificate"],"sha256":sha(pred["certificate"]),"relation":"AUTHORIZATION_SEMANTIC_CORRECTION"},"verifier":VERIFIER}
  d=f"experiments/certificate-canonical-{claim_id.lower()}-v18"
  results[f"{d}/certificate.json"]=json.dumps(cert,indent=2,sort_keys=True)+"\n"
  results[f"{d}/README.md"]=f"# {cert['certificate_id']}\n\nv18 certificate for proposal/authorization correction. v12 predecessor bytes are frozen.\n\n- claim: {claim_id}\n- experiment: `{expname}`\n- status: `CANONICAL_ISSUED_V18`\n- predecessor: `{pred['certificate']}`\n- native group evidence: `audit/phase15/manifest.json`\n"
  index["certificates"].append({"claim":claim_id,"certificate":f"{d}/certificate.json","experiment":expname,"classification":pred["classification"],"status":"CANONICAL_ISSUED_V18","predecessor":pred["certificate"]})
 index["summary"]={"certificates":len(index["certificates"]),"claims":sorted(e["claim"] for e in index["certificates"])}
 results[str(V18_INDEX.relative_to(ROOT))]=json.dumps(index,indent=2,sort_keys=True)+"\n"
 if "--check" in argv:
  stale=[r for r,c in results.items() if not (ROOT/r).is_file() or (ROOT/r).read_text()!=c]
  if stale: print("v18 certificates stale: "+", ".join(stale[:5]),file=sys.stderr); return 1
  print(f"ok v18 certificates: {len(index['certificates'])}"); return 0
 for rel,content in results.items():
  p=ROOT/rel; p.parent.mkdir(parents=True,exist_ok=True); p.write_text(content)
 print(f"wrote {len(index['certificates'])} v18 certificates"); return 0
if __name__=="__main__": raise SystemExit(main(sys.argv[1:]))
