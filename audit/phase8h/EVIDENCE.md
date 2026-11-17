# N8-H Evidence

```bash
python3 audit/phase8h/reproduce_false_pass.py
python3 audit/phase8h/fmpy_schedule.py --kind pv --schedule 0,3600,4500,7200,10800,14400 --step 1000 --failure 4500 --output audit/phase8h/external/N8H-N8-02.csv --log audit/phase8h/external/N8H-N8-02.log --metadata audit/phase8h/external/N8H-N8-02.meta.json
python3 audit/phase8h/fmpy_schedule.py --kind pv --schedule 0,3600,7200,10800,14400 --step 900 --failure 7200 --output audit/phase8h/external/N8H-N8-01.csv --log audit/phase8h/external/N8H-N8-01.log --metadata audit/phase8h/external/N8H-N8-01.meta.json
python3 audit/phase8h/fmpy_schedule.py --kind repair --schedule 0,3600,7200 --step 900 --robot-failure 7200 --output audit/phase8h/external/N8H-N8-04.csv --log audit/phase8h/external/N8H-N8-04.log --metadata audit/phase8h/external/N8H-N8-04.meta.json
python3 audit/phase8h/compare_strict.py
python3 audit/phase8h/negative_controls.py
python3 audit/phase8h/check.py
```

Old N8 evidence hashes: `audit/phase8h/old-evidence.json`.
