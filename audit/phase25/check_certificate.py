# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import hashlib,json,pathlib
R=pathlib.Path(__file__).resolve().parents[2]
c=json.loads((R/'audit/phase25/N25_EXTERNAL_VALIDATION_CERTIFICATE.json').read_text()); bad=[]
for p,h in c['inputs'].items():
 if hashlib.sha256((R/p).read_bytes()).hexdigest()!=h: bad.append(p)
if bad: print('N25 cert FAIL',bad[:5]); raise SystemExit(1)
print('ok N25 external certificate')
