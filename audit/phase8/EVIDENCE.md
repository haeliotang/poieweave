# N8 Evidence

```bash
cargo run -q -p poie-viability --bin poie-s5v-n8-events
python3 audit/phase8/check.py
python3 scripts/build-canonical-claim-certificates-v4.py --check
python3 scripts/verify-canonical-claim-certificates-v4.py
python3 scripts/check-canonical-claim-certificate-tamper-v4.py
python3 audit/certificates/verify_v3.py
python3 audit/certificates/verify_v2.py
python3 audit/certificates/verify_v1_historical.py
```

Frozen manifest: `audit/phase8/manifest.json`.
