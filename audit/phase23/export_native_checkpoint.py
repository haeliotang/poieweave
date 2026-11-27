# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
from __future__ import annotations
import hashlib,json,re,subprocess
from pathlib import Path
P=Path(__file__).resolve().parent; ROOT=P.parents[1]
r=subprocess.run(["cargo","run","-q","-p","poie-viability","--example","n23_checkpoint_export"],cwd=ROOT,capture_output=True,text=True,check=True)
projection=json.loads(r.stdout)
commit=projection["checkpoint_commitment"]
assert re.fullmatch(r"[0-9a-f]{64}",commit), commit
assert projection["trusted_provenance_verified"] is True
assert commit!="a"*64
out={"schema":"poie.s5v-n23-native-checkpoint-run/1","projection":projection,"projection_sha256":hashlib.sha256(json.dumps(projection,sort_keys=True,separators=(",",":")).encode()).hexdigest(),"trusted_verified":True}
(P/"native-checkpoints"/"native.json").write_text(json.dumps(out,indent=2,sort_keys=True)+"\n")
print(commit,len(projection["unfinished_tasks"]),projection["failed_assets"])
