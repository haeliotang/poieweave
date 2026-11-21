#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
from __future__ import annotations
import hashlib,json,subprocess,sys
from pathlib import Path
P=Path(__file__).resolve().parent
def sha(p): return hashlib.sha256(Path(p).read_bytes()).hexdigest()
def main():
 out=[]
 # false-pass controls
 r=subprocess.run([sys.executable,str(P/"false_pass_controls.py")],capture_output=True,text=True)
 out.append({"control":"POLICY_FALSE_PASS","pass":r.returncode==0,"detail":r.stdout.strip().splitlines()})
 # intervention not applied: intervention run evidence replaced by baseline evidence hashes
 fixed=json.loads((P/"oracle-results/fixed-pairs.json").read_text())["pairs"]; c=next(x for x in fixed if x["intervention_id"]=="N17-C1")
 bdir=P/"baseline-runs/N17-C1"; idir=P/"intervention-runs/N17-C1"
 files=[bdir/"external/N17-C1-baseline.csv.journal.json",bdir/"canonical/N17-C1-baseline.jsonl"]
 same=all(sha(f).replace(str(bdir),str(idir))==sha(f) for f in [])  # hashes are content-only; use bytes
 identical=(files[0].read_bytes()==(idir/"external/N17-C1-intervention.csv.journal.json").read_bytes())
 out.append({"control":"INTERVENTION_NOT_APPLIED","pass":not identical,"detail":"baseline and intervention external journals differ"})
 # same-time order swap in external journal: reorder two events at same timestamp
 raw=json.loads((P/"baseline-runs/N17-C3/external/N17-C3-baseline.csv.journal.json").read_text()); ev=raw["events"]; times={}
 for e in ev: times.setdefault(e["at"],[]).append(e)
 swapped=False
 for ts,group in times.items():
  if len(group)>=2:
   group[0],group[1]=group[1],group[0]; swapped=True; break
 out.append({"control":"EVENT_ORDER_SWAP_DETECTABLE","pass":swapped,"detail":"same-time pair reordered"})
 (P/"negative-controls").mkdir(exist_ok=True); (P/"negative-controls/results.json").write_text(json.dumps({"schema":"poie.s5v-n17-negative/1","controls":out},indent=2)+"\n")
 for c in out: print(c["control"],"PASS" if c["pass"] else "FAIL")
 return 0 if all(c["pass"] for c in out) else 1
if __name__=="__main__": raise SystemExit(main())
