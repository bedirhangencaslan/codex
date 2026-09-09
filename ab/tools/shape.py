"""Print the read-shape and price of one archived run: python shape.py <tag-dir> <tag>"""
import json,os,sys,statistics,collections
d,tag=sys.argv[1],sys.argv[2]
js=os.path.join(d,tag+'.jsonl'); ro=os.path.join(d,tag+'.rollout.jsonl')
rows=[json.loads(l) for l in open(js,encoding='utf-8') if l.strip()]
ok=[r for r in rows if r.get('status')==200]
f=sum((r['prompt_tokens']-r['cached_tokens']) for r in ok); c=sum(r['cached_tokens'] for r in ok)
o=sum(r['completion_tokens'] for r in ok); peak=max(r['prompt_tokens'] for r in ok)
cost=(f*0.075+c*0.015+o*0.25)/1e6
prompts=[r['prompt_tokens'] for r in ok]
comp=sum(1 for a,b in zip(prompts,prompts[1:]) if b < a*0.7)
print(f"requests={len(ok)}  peak={peak}  compactions={comp}  fresh={f}  cached={c}  out={o}  ${cost:.4f}")
print("prompt series:",prompts)
lims=[];nolim=0;seen=collections.Counter();calls=collections.Counter();par=collections.Counter()
outbytes=0
cur=None
for l in open(ro,encoding='utf-8'):
    p=(json.loads(l).get('payload') or {})
    if not isinstance(p,dict): continue
    if p.get('type')=='function_call':
        n=p.get('name'); calls[n]+=1
        try: a=json.loads(p.get('arguments') or '{}')
        except Exception: a={}
        if n=='read':
            fp=a.get('filePath') or a.get('path') or (a.get('paths') or [None])[0]
            if isinstance(fp,dict): fp=fp.get('path')
            if fp: seen[os.path.basename(str(fp).replace(chr(92),'/'))]+=1
            if a.get('limit'): lims.append(a['limit'])
            else: nolim+=1
    if p.get('type')=='function_call_output':
        out=p.get('output'); s=out if isinstance(out,str) else json.dumps(out); outbytes+=len(s)
print("tool calls:",dict(calls))
print(f"read: {len(lims)+nolim} calls, limit set on {len(lims)}, none on {nolim}, median limit={statistics.median(lims) if lims else '-'} range={min(lims) if lims else '-'}-{max(lims) if lims else '-'}")
print(f"distinct files={len(seen)} slices={sum(seen.values())} re-reads={sum(v-1 for v in seen.values() if v>1)}")
print(f"tool output bytes total={outbytes}")
