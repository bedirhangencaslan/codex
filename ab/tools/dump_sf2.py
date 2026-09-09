import json,sys,os,io,textwrap
src,dst,jsonl,note=sys.argv[1],sys.argv[2],sys.argv[3],sys.argv[4]
rows=[json.loads(l) for l in open(jsonl,encoding='utf-8') if l.strip()]
ok=[r for r in rows if r['status']==200]
f=sum(r['prompt_tokens']-r['cached_tokens'] for r in ok); c=sum(r['cached_tokens'] for r in ok)
o=sum(r['completion_tokens'] for r in ok); cost=(f*0.075+c*0.015+o*0.25)/1e6
def wrap(t,keep,width=160):
    out=[]
    for line in t.split('\n')[:keep]:
        out.extend(textwrap.wrap(line,width,drop_whitespace=False) or [''])
    return '\n'.join(out)
b=[f"# {note}\n",
   f"- kaynak: {os.path.basename(src)}",
   f"- istek: {len(ok)} (200) + {len(rows)-len(ok)} hatali",
   f"- fresh {f} / cached {c} / output {o} token",
   f"- fiyat: ${cost:.4f}",
   f"- prompt serisi: {[r['prompt_tokens'] for r in ok]}\n\n---\n"]
for l in open(src,encoding='utf-8'):
    d=json.loads(l); pl=d.get('payload') or {}
    if not isinstance(pl,dict): continue
    t=pl.get('type')
    if t=='message':
        txt='\n'.join(x.get('text','') for x in pl.get('content',[]) or [])
        b.append(f"## {pl.get('role','?')}\n\n"+wrap(txt,200)+"\n")
    elif t=='reasoning':
        s=''.join(x.get('text','') for x in pl.get('content',[]) or []).strip()
        if s: b.append("### dusunce\n\n"+wrap(s,200)+"\n")
    elif t in ('function_call','custom_tool_call'):
        raw=pl.get('arguments') if t=='function_call' else str(pl.get('input'))
        try: pretty=json.dumps(json.loads(raw),ensure_ascii=False,indent=1)
        except Exception: pretty=raw or ''
        b.append(f"### arac: {pl.get('name')}\n\n```json\n"+wrap(pretty,60)+"\n```\n")
    elif t in ('function_call_output','custom_tool_call_output'):
        ov=pl.get('output'); s=ov if isinstance(ov,str) else json.dumps(ov,ensure_ascii=False)
        total=len(s.split('\n'))
        more='' if total<=50 else f"\n... [{total-50} satir daha, toplam {len(s)} karakter]"
        b.append("#### cikti\n\n```\n"+wrap(s,50)+more+"\n```\n")
    elif t=='compacted':
        b.append("---\n\n## [COMPACTION]\n\n---\n")
io.open(dst,'w',encoding='utf-8',newline='\n').write('\n'.join(b))
print(dst,os.path.getsize(dst))
