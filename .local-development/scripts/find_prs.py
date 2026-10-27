import json
import subprocess
import re

cmd = ['gh', 'pr', 'list', '--repo', 'storytold/filmcraft', '--state', 'open', '--limit', '600', '--json', 'number,title,author,headRefName,createdAt,labels']
res = subprocess.run(cmd, capture_output=True, text=True, encoding='utf-8')
prs = json.loads(res.stdout)

import re

categories = {
    'windows': [r'\bwindows\b', r'\bd3d\b', r'\bd3d11\b', r'\bd3d12\b', r'\bdirect3d\b', r'\bmedia\s*foundation\b', r'\bdxva\b', r'\bwin32\b', r'\bwasapi\b'],
    'gpu_perf': [r'\bgpu\b', r'\bwgpu\b', r'\bperf\b', r'\bzero-copy\b', r'\breadback\b', r'\bread-back\b', r'\bstaging\b', r'\bthroughput\b', r'\bplayback\b', r'\bscrub\b', r'\bfast\s*path\b'],
    'codecs_media': [r'\bdecode\b', r'\bdecoder\b', r'\bencode\b', r'\bencoder\b', r'\bhevc\b', r'\bh\.?264\b', r'\bav1\b', r'\bvp9\b', r'\bprores\b', r'\baac\b', r'\bvfr\b', r'\bmp4\b', r'\b\.mov\b', r'\bmov\b', r'\btimestamp\b', r'\biphone\b', r'\bphone\b', r'\bcolor\b', r'\bhdr\b', r'\blut\b', r'\bmedia\s*pool\b'],
    'stability': [r'\bcrash\b', r'\bpanic\b', r'\bdeadlock\b', r'\bleak\b', r'\boom\b', r'\boverflow\b', r'\bhang\b', r'\btimeout\b', r'\bbound\b', r'\bbounded\b', r'\brace\b', r'\block\b', r'\bfile-lock\b']
}

classified = {cat: [] for cat in categories}
scored = []

for pr in prs:
    title = pr.get('title', '')
    ref = pr.get('headRefName', '')
    text = f"{title} {ref}".lower()
    
    hits = {}
    total_hits = 0
    for cat, patterns in categories.items():
        matched = []
        for pat in patterns:
            if re.search(pat, text, re.IGNORECASE):
                clean_kw = pat.replace(r'\b', '').replace(r'\s*', ' ')
                matched.append(clean_kw)
        if matched:
            hits[cat] = matched
            total_hits += len(matched)
            classified[cat].append((pr, matched))
            
    if total_hits > 0:
        scored.append((total_hits, hits, pr))

scored.sort(key=lambda x: x[0], reverse=True)
print(f"Total open PRs: {len(prs)}")
print(f"Total PRs matching core criteria: {len(scored)}")

with open('.local-development/candidate_prs.json', 'w', encoding='utf-8') as f:
    json.dump([{'number': p['number'], 'title': p['title'], 'author': p['author']['login'], 'branch': p['headRefName'], 'hits': h, 'score': s} for s, h, p in scored], f, indent=2)

with open('.local-development/OPEN_PRS_AUDIT.md', 'w', encoding='utf-8') as f:
    f.write("# FilmCraft Open Pull Requests Audit (Windows & Reliability Candidates)\n\n")
    f.write(f"Analyzed {len(prs)} open pull requests on `storytold/filmcraft` as of October 2026.\n\n")
    
    for cat, items in classified.items():
        f.write(f"## Category: {cat.upper()} ({len(items)} PRs)\n\n")
        f.write("| PR | Title | Author | Branch | Matched Keywords |\n")
        f.write("| :--- | :--- | :--- | :--- | :--- |\n")
        for pr, kw in items:
            title_escaped = pr['title'].replace('|', '/')
            f.write(f"| [#{pr['number']}](https://github.com/storytold/filmcraft/pull/{pr['number']}) | {title_escaped} | `{pr['author']['login']}` | `{pr['headRefName']}` | {', '.join(kw)} |\n")
        f.write("\n")

print("Report successfully generated at .local-development/OPEN_PRS_AUDIT.md")

