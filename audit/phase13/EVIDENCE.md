# N13 Evidence

- `contracts/N13-*.contract`
- `canonical/N13-*-events.jsonl`
- `external/N13-*.csv.journal.json`
- `trajectory-comparisons/multifailure.json`
- `N12_REVALIDATION.md`
- `external/fmu-n13-scheduler/`

Replay:

```bash
cargo run -p poie-viability --bin poie-s5v-n13-multifailure
python3 audit/phase13/validate_multifailure.py
```
