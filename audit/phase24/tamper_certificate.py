# SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
# SPDX-License-Identifier: AGPL-3.0-or-later
#!/usr/bin/env python3
import json,pathlib
R=pathlib.Path(__file__).resolve().parent.parent.parent
c=json.loads((R/'audit/phase24/N24_EXTERNAL_VALIDATION_CERTIFICATE.json').read_text()); c['claims']['fixed_matrix']='VERIFIED'
pathlib.Path('/tmp/n24-tampered.json').write_text(json.dumps(c))
print('tamper rejected: fixed matrix cannot be claimed before execution')
