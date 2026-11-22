#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import json,sys
from pathlib import Path
P=Path(__file__).resolve().parent; sys.path.insert(0,str(P)); import four_run as F
CASES=json.loads((P/"oracle-results/phaseb.json").read_text())["cases"]
def main():
 problems=[]; count=0
 for c in CASES:
  cid=c["case_id"]
  for lab in ("baseline","intervention"):
   for model in ("canonical","external"):
    root=P/"counterfactual-pairs"/cid/lab/model
    if not (root/"run.json").is_file(): problems.append(f"{cid}-{lab}-{model}-MISSING"); continue
    man=json.loads((root/"run.json").read_text()); count+=1
    if man["run_id"]!=f"{cid}-{lab}-{model}": problems.append("run id")
    try: F.validate_existing(cid,lab,model)
    except SystemExit: problems.append(f"{cid}-{lab}-{model}-HASH")
 # deterministic replay with new run_id for one case
 import run_n18 as R
 cid="N19-01"; sp=R.spec(failures=())
 try:
  man1=F.validate_existing(cid,"baseline","canonical")
  man2=F.execute_once(cid,"baseline","canonical",sp,run_id=f"{cid}-baseline-canonical-replay")
  h1=man1["raw_output_sha256"]; h2=man2["raw_output_sha256"]; replay_ok=h1==h2
 except SystemExit as e: replay_ok=False
 (P/"oracle-results/provenance.json").write_text(json.dumps({"schema":"poie.s5v-n19-provenance/1","runs":count,"problems":problems,"deterministic_replay":bool(replay_ok)},indent=2)+"\n")
 print("runs",count,"replay",replay_ok,"problems",problems)
 return 0 if count==40 and not problems and replay_ok else 1
if __name__=="__main__": raise SystemExit(main())
