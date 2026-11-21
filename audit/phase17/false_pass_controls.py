import json,subprocess,pathlib
P=pathlib.Path('audit/phase16'); case='N16-09'; cp=P/'canonical'/f'{case}.jsonl'; ep=P/'external'/f'{case}.csv.journal.json'
cb=cp.read_text(); eb=ep.read_text(); out=[]
def run(name,fn):
 ce=[json.loads(x) for x in cb.splitlines()]; doc=json.loads(eb); ev=doc['events']; fn(ce,ev)
 cp.write_text('\n'.join(json.dumps(x) for x in ce)+'\n'); doc['events']=ev; ep.write_text(json.dumps(doc))
 subprocess.run(['python3','audit/phase16/validate_closure.py'],capture_output=True,text=True)
 r=json.load(open(P/'dependency-trajectories/results.json')); row=next(x for x in r['cases'] if x['case']==case)
 out.append((name,row['status'],row['findings']))
try:
 run('policy+bad-closure',lambda ce,ev:[e.__setitem__('dep_after',e['dep_after']&~(1<<6)) for e in ev if e.get('at',-1)>=7200-1e-9 and 'dep_after' in e])
 run('policy+missing-failure',lambda ce,ev:ev.__setitem__(slice(None),[e for e in ev if e['kind']!='AssetFailure']))
 def early(ce,ev): ev.append({'kind':'RestorationCommitted','at':3600.0,'target':'a-lathe','origin':'NATIVE_EXTERNAL_FMU'})
 run('policy+early-restoration',early)
finally: cp.write_text(cb); ep.write_text(eb)
pathlib.Path('audit/phase17/negative-controls/false-pass.json').write_text(json.dumps({'schema':'poie.s5v-n17-false-pass/1','controls':[{'name':n,'status':s,'findings':f} for n,s,f in out]},indent=2)+'\n')
for x in out: print(x)
raise SystemExit(0 if all(s=='FAIL' for _,s,_ in out) else 1)
