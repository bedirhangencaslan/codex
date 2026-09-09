import json,sys,os
p=sys.argv[1]
def key(x): 
    x=x.replace(chr(92),'/'); i=x.find('codex-api/'); return x[i+10:] if i>=0 else x
buf=[]
for l in open(p,encoding='utf-8'):
    d=json.loads(l); pl=d.get('payload') or {}
    if not isinstance(pl,dict): continue
    t=pl.get('type')
    if t=='reasoning':
        s=''.join(c.get('text','') for c in pl.get('content',[]) or []).strip()
        if s: print('\n>>> THINK:',s)
    elif t=='function_call':
        try: a=json.loads(pl.get('arguments') or '{}')
        except Exception: a={}
        if pl.get('name')=='read':
            fp=a.get('filePath') or a.get('path') or ''
            print(f"      read {key(str(fp)):52s} offset={a.get('offset','-')!s:>6s} limit={a.get('limit','YOK')}")
        else:
            print(f"      {pl.get('name')} {json.dumps(a)[:90]}")
    elif t=='message' and pl.get('role')=='assistant':
        c=' '.join(x.get('text','') for x in pl.get('content',[]) or [])
        print('  [MESAJ]',c[:160])
