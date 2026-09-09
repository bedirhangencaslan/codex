import json,sys,os,io,textwrap
src,dst,jsonl,note=sys.argv[1],sys.argv[2],sys.argv[3],sys.argv[4]
rows=[json.loads(l) for l in open(jsonl,encoding='utf-8') if l.strip()]
ok=[r for r in rows if r['status']==200]
f=sum(r['prompt_tokens']-r['cached_tokens'] for r in ok); c=sum(r['cached_tokens'] for r in ok)
o=sum(r['completion_tokens'] for r in ok); cost=(f*0.075+c*0.015+o*0.25)/1e6
def wr(t,w=100):
    out=[]
    for line in t.split('\n'):
        out.extend(textwrap.wrap(line,w) or [''])
    return '\n'.join(out)
b=[note,'='*len(note),'',
   f"istek {len(ok)} | fresh {f} | cached {c} | output {o} | ${cost:.4f}",
   f"prompt serisi: {[r['prompt_tokens'] for r in ok]}",'',
   "(sadece dusunceler ve arac cagrilari; arac ciktilari boyut ozeti olarak)",'','-'*70,'']
calls={}
for l in open(src,encoding='utf-8'):
    d=json.loads(l); pl=d.get('payload') or {}
    if not isinstance(pl,dict): continue
    t=pl.get('type')
    if t=='message':
        txt='\n'.join(x.get('text','') for x in pl.get('content',[]) or [])
        b.append(f"[{pl.get('role','?').upper()}]"); b.append(wr(txt[:1200])); b.append('')
    elif t=='reasoning':
        s=''.join(x.get('text','') for x in pl.get('content',[]) or []).strip()
        if s: b.append('DUSUNCE:'); b.append(wr(s)); b.append('')
    elif t in ('function_call','custom_tool_call'):
        raw=pl.get('arguments') if t=='function_call' else str(pl.get('input'))
        calls[pl.get('call_id')]=pl.get('name')
        b.append(f"  -> {pl.get('name')}  {wr((raw or '')[:300],100)}")
    elif t in ('function_call_output','custom_tool_call_output'):
        ov=pl.get('output'); s=ov if isinstance(ov,str) else json.dumps(ov,ensure_ascii=False)
        b.append(f"  <- cikti: {len(s)} karakter, {len(s.split(chr(10)))} satir"); b.append('')
    elif t=='compacted':
        b.append(''); b.append('*** COMPACTION ***'); b.append('')
io.open(dst,'w',encoding='utf-8',newline='\n').write('\n'.join(b))
print(dst, os.path.getsize(dst))
