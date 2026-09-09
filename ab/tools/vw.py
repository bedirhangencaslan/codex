import os,re,sys
BSL=chr(92)
PAT=re.compile(r"([A-Za-z0-9_./"+BSL+r"-]+\.rs)[:\s]+(?:line\s*)?(\d+)")
root=sys.argv[1]
idx={}
for dp,_d,fs in os.walk(os.path.join(root,'codex-api')):
    for f in fs: idx.setdefault(f,os.path.join(dp,f))
for p in sys.argv[2:]:
    txt=open(p,encoding='utf-8',errors='replace').read()
    ok=bad=miss=0; files=set()
    for m in PAT.finditer(txt):
        name=os.path.basename(m.group(1).replace(BSL,'/')); line=int(m.group(2))
        fp=idx.get(name)
        if not fp: miss+=1; continue
        n=sum(1 for _ in open(fp,encoding='utf-8',errors='replace'))
        if 1<=line<=n: ok+=1; files.add(name)
        else: bad+=1
    print(f"{os.path.basename(p):28s} cites={ok+bad+miss:3d} valid={ok:3d} overshoot={bad:2d} nofile={miss:2d} distinct_files={len(files)}")
