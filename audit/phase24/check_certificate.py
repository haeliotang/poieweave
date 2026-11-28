# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import hashlib,json,pathlib
R=pathlib.Path(__file__).resolve().parent.parent.parent
c=json.loads((R/'audit/phase24/N24_EXTERNAL_VALIDATION_CERTIFICATE.json').read_text()); bad=[]
hist=json.loads((R/'audit/phase25/historical/n24-input-hashes.json').read_text())['sha256']
for p,h in c['inputs'].items():
 cur=hashlib.sha256((R/p).read_bytes()).hexdigest()
 if cur!=h and hist.get(p)!=h: bad.append(p)
if bad: print('N24 cert FAIL',bad[:5]); raise SystemExit(1)
print('ok N24 external certificate')
