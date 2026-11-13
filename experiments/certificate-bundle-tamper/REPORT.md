# Certificate Bundle Tamper Test — report

- Date: 2026-11-14
- Status: negative integrity test
- Runner: `python3 experiments/certificate-bundle-tamper/tamper.py`
- Expected output: [expected.txt](expected.txt)

## 1. Result

```text
tamper=detected
```

## 2. What happens

1. Bundle + six certificates copied to a temp root.
2. Clean copy verifies `bundle=ok`.
3. One byte is appended to the S1 certificate.
4. Verification now fails because the certificate SHA-256 no longer matches the
   bundle.

## 3. Limits

- Content-integrity test only.
- No signature, key management, author identity, freshness or rollback proof.
- Not a security audit.
