#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
"""N19 four-run executor: one model per run_id, no evidence overwrite."""
from __future__ import annotations
import hashlib,json,subprocess,sys
from datetime import datetime,timezone
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]; sys.path.insert(0,str(ROOT/"audit/phase18"))
import run_n18 as R
CANON=ROOT/"target/debug/poie-s5v-n14-canonical-adapter"; H=ROOT/"external/fmu-n16-scheduler/build/harness"; FMU=ROOT/"external/fmu-n16-scheduler/poie-n16-scheduler.fmu"
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def execute_once(case,label,model,spec,run_id=None):
 default_id=f"{case}-{label}-{model}"; run_id=run_id or default_id
 root=P/"counterfactual-pairs"/case/label/model/(f"replays/{run_id}" if run_id!=default_id else ""); root.mkdir(parents=True,exist_ok=True)
 manifest_path=root/"run.json"
 if manifest_path.exists(): raise SystemExit(f"refusing overwrite existing run {run_id}")
 js=R.js_for(case,label,spec); jp=root/f"{run_id}.json"; cp=root/f"{run_id}.contract"; jp.write_text(json.dumps(js,indent=2)+"\n"); R.write_contract(js,cp)
 start=datetime.now(timezone.utc).isoformat()
 if model=="canonical":
  out=subprocess.run([str(CANON),str(jp)],cwd=ROOT,capture_output=True,text=True)
  (root/f"{run_id}.jsonl").write_text(out.stdout); raw=root/f"{run_id}.jsonl"; journal=None
 else:
  out=subprocess.run([str(H),str(FMU),"900",str(cp),str(root/f"{run_id}.csv")],cwd=ROOT,capture_output=True,text=True)
  raw=root/f"{run_id}.csv"; journal=root/f"{run_id}.csv.journal.json"
 end=datetime.now(timezone.utc).isoformat()
 man={"run_id":run_id,"case":case,"label":label,"model_id":model,"input_json":str(jp.relative_to(P)),"contract":str(cp.relative_to(P)),"contract_sha256":sha(cp),"binary":{"path":str((CANON if model=="canonical" else FMU).relative_to(ROOT)),"sha256":sha(CANON if model=="canonical" else FMU)},"started_at":start,"completed_at":end,"exit_status":out.returncode,"raw_output":str(raw.relative_to(P)),"raw_output_sha256":sha(raw),"event_journal":str(journal.relative_to(P)) if journal else None,"event_journal_sha256":sha(journal) if journal and journal.exists() else None,"stderr":out.stderr[-1000:]}
 (root/"run.json").write_text(json.dumps(man,indent=2)+"\n"); return man
def validate_existing(case,label,model):
 root=P/"counterfactual-pairs"/case/label/model; man=json.loads((root/"run.json").read_text())
 if sha(root/Path(man["raw_output"]).name)!=man["raw_output_sha256"]: raise SystemExit("raw hash mismatch")
 if man["event_journal"] and sha(root/Path(man["event_journal"]).name)!=man["event_journal_sha256"]: raise SystemExit("journal hash mismatch")
 return man
