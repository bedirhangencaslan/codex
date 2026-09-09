import sqlite3,os,json,io,sys,textwrap
slug,dst,jsonl=sys.argv[1],sys.argv[2],sys.argv[3]
c=sqlite3.connect(os.path.join(os.environ['TMP'],'oc_copy.db')); c.row_factory=sqlite3.Row
sid,=c.execute('select id from session where slug=?',(slug,)).fetchone()
rows=[json.loads(l) for l in open(jsonl,encoding='utf-8') if l.strip()]
ok=[r for r in rows if r['status']==200]
f=sum(r['prompt_tokens']-r['cached_tokens'] for r in ok); ca=sum(r['cached_tokens'] for r in ok)
o=sum(r['completion_tokens'] for r in ok); cost=(f*0.075+ca*0.015+o*0.25)/1e6
def wrap(text, keep_lines, width=160):
    out=[]
    for line in text.split('\n')[:keep_lines]:
        out.extend(textwrap.wrap(line, width, drop_whitespace=False) or [''])
    return '\n'.join(out)
buf=[]
buf.append(f"# OpenCode oturum dokumu - {slug} ({sid})\n")
buf.append(f"- istek: {len(ok)} (200) + {len(rows)-len(ok)} hatali")
buf.append(f"- fresh {f} / cached {ca} / output {o} token")
buf.append(f"- fiyat: ${cost:.4f}")
buf.append(f"- prompt serisi: {[r['prompt_tokens'] for r in ok]}\n\n---\n")
for r in c.execute('select data from part where session_id=? order by time_created,id',(sid,)):
    d=json.loads(r['data']); t=d.get('type')
    if t=='text':
        buf.append("## mesaj\n\n"+wrap(d.get('text',''),200)+"\n")
    elif t=='reasoning':
        buf.append("### dusunce\n\n"+wrap(d.get('text','').strip(),200)+"\n")
    elif t=='tool':
        st=d.get('state',{}); inp=st.get('input',{}) or {}; out=st.get('output') or ''
        pretty=json.dumps(inp,ensure_ascii=False,indent=1)
        buf.append(f"### arac: {d.get('tool')}\n\n```json\n"+wrap(pretty,60)+"\n```\n")
        total=len(out.split('\n'))
        more='' if total<=50 else f"\n... [{total-50} satir daha, toplam {len(out)} karakter]"
        buf.append("#### cikti\n\n```\n"+wrap(out,50)+more+"\n```\n")
    elif t=='step-start':
        buf.append("---\n")
io.open(dst,'w',encoding='utf-8',newline='\n').write('\n'.join(buf))
print(dst, os.path.getsize(dst))
