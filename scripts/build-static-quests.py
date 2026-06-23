#!/usr/bin/env python3
"""
build-static-quests.py — render quests/*.html into self-contained static pages
for serving via Caddy at questhub.eco/q/<slug>/ (no Jekyll/Ruby, no app rebuild).

Each output is a complete HTML document that links the shared stylesheet at
/q/assets/style.css (deployed alongside), keeps the page's own scoped <style>,
and wraps a trimmed nav/footer. The "Back to the garden" link points at the live
questhub.eco home (the Kidur app). Output tree mirrors the Caddy route:

    dist-quests/<slug>/index.html      ->  /q/<slug>/
    dist-quests/assets/style.css       ->  /q/assets/style.css

Usage: python3 scripts/build-static-quests.py
"""
import os, re, glob, shutil, html

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT  = os.path.join(REPO, "dist-quests")
FRONT = re.compile(r"^---\n(.*?)\n---\n(.*)$", re.S)

def fv(fm, key):
    m = re.search(rf"^{key}:\s*(.+)$", fm, re.M)
    if not m: return ""
    v = m.group(1).strip()
    if len(v) >= 2 and v[0] in "\"'" and v[-1] == v[0]:
        v = v[1:-1]
    return v

NAV = """  <nav class="qh-nav" id="nav">
    <a href="https://questhub.eco/" class="qh-nav-logo">Quest<span>Hub</span></a>
    <div class="qh-nav-right">
      <a href="https://questhub.eco/" class="qh-nav-link">The Garden</a>
      <a href="https://evobiosys.org" class="qh-nav-parent" target="_blank" rel="noopener">Part of EvoBioSys &#x2197;</a>
    </div>
  </nav>"""

FOOTER = """  <footer class="qh-footer">
    <div class="qh-footer-inner">
      <div class="qh-footer-brand">
        <span class="qh-footer-logo">QuestHub</span>
        <span class="qh-footer-tagline">The Garden of Quests</span>
      </div>
      <div class="qh-footer-links">
        <a href="https://questhub.eco/">The Garden</a>
        <a href="https://evobiosys.org" target="_blank" rel="noopener">EvoBioSys</a>
      </div>
      <p class="qh-footer-copy">&copy; 2026 EvoBioSys &middot; Part of the EvoBioSys ecosystem</p>
    </div>
  </footer>
  <script>
    const nav = document.getElementById('nav');
    if (nav) window.addEventListener('scroll', () => nav.classList.toggle('scrolled', window.scrollY > 20));
  </script>"""

PAGE = """<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>{title}</title>
  <meta name="description" content="{desc}">
  <link rel="canonical" href="https://questhub.eco/q/{slug}/">
  <meta property="og:title" content="{title}">
  <meta property="og:description" content="{desc}">
  <meta property="og:type" content="article">
  <link rel="stylesheet" href="/q/assets/style.css">
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" rel="stylesheet">
</head>
<body>
{nav}
{body}
{footer}
</body>
</html>
"""

def main():
    if os.path.isdir(OUT):
        shutil.rmtree(OUT)
    os.makedirs(os.path.join(OUT, "assets"))
    shutil.copyfile(os.path.join(REPO, "assets", "css", "style.css"),
                    os.path.join(OUT, "assets", "style.css"))
    pages = sorted(glob.glob(os.path.join(REPO, "quests", "*.html")))
    built = []
    for path in pages:
        raw = open(path, encoding="utf-8").read()
        m = FRONT.match(raw)
        if not m:
            continue
        fm, body = m.group(1), m.group(2)
        perma = fv(fm, "permalink")          # /quests/<slug>/
        slug = perma.strip("/").split("/")[-1]
        title = fv(fm, "title")
        desc = html.escape(fv(fm, "description"), quote=True)
        doc = PAGE.format(title=html.escape(title), desc=desc, slug=slug,
                          nav=NAV, body=body.strip(), footer=FOOTER)
        dest_dir = os.path.join(OUT, slug)
        os.makedirs(dest_dir, exist_ok=True)
        open(os.path.join(dest_dir, "index.html"), "w", encoding="utf-8").write(doc)
        built.append((slug, title))
    print(f"Built {len(built)} static quest pages + assets/style.css into dist-quests/")
    for slug, title in built:
        print(f"  /q/{slug}/  —  {title}")

if __name__ == "__main__":
    main()
