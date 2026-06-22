#!/usr/bin/env python3
"""
preview.py — Ruby-free local preview of the QuestHub Jekyll site.

Local Ruby is 2.6 (too old for Jekyll); CI builds on 3.1. This renders the
pages the same way Jekyll would — strips YAML front matter, injects the body
into _layouts/default.html at {{ content }}, resolves the handful of liquid
tags the layout uses, mirrors each page's `permalink` into _preview/, copies
assets/, and serves it. Good enough to eyeball layout + styling before deciding
to deploy. (CI remains the source of truth for the real build.)

Usage:  python3 scripts/preview.py          # serve on :8777, open browser
        python3 scripts/preview.py --port N  # custom port
        python3 scripts/preview.py --no-open # don't auto-open browser
"""
import os, re, sys, glob, shutil, html, argparse, http.server, socketserver, threading, webbrowser, functools

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT  = os.path.join(REPO, "_preview")
LAYOUT = os.path.join(REPO, "_layouts", "default.html")
FRONT = re.compile(r"^---\n(.*?)\n---\n(.*)$", re.S)

def front_val(fm, key):
    m = re.search(rf"^{key}:\s*(.+)$", fm, re.M)
    if not m: return None
    v = m.group(1).strip()
    if v[:1] in "\"'" and v[-1:] == v[:1]:
        v = v[1:-1]
    return v

def render(layout, body, fm, site_title):
    title = front_val(fm, "title") or site_title
    out = layout.replace("{{ content }}", body)
    # title (with | default: site.title)
    out = re.sub(r"\{\{\s*page\.title[^}]*\}\}", html.escape(title), out)
    out = re.sub(r"\{\{\s*page\.description[^}]*\}\}", html.escape(front_val(fm, "description") or ""), out)
    out = re.sub(r"\{\{\s*site\.title\s*\}\}", html.escape(site_title), out)
    # relative_url filter -> just the path
    out = re.sub(r"\{\{\s*'([^']*)'\s*\|\s*relative_url\s*\}\}", r"\1", out)
    # canonical {{ site.url }}{{ page.url }} -> drop
    out = re.sub(r"\{\{\s*site\.\w+\s*\}\}", "", out)
    out = re.sub(r"\{\{\s*page\.url\s*\}\}", "", out)
    # {% seo %} and any other liquid tags -> drop
    out = re.sub(r"\{%.*?%\}", "", out)
    out = re.sub(r"\{\{.*?\}\}", "", out)
    return out

def perma(path, fm):
    p = front_val(fm, "permalink")
    if p:
        return p.strip("/") + "/index.html" if p != "/" else "index.html"
    name = os.path.splitext(os.path.basename(path))[0]
    return "index.html" if name == "index" else f"{name}/index.html"

def build():
    if os.path.isdir(OUT):
        shutil.rmtree(OUT)
    os.makedirs(OUT)
    layout = open(LAYOUT, encoding="utf-8").read()
    cfg = open(os.path.join(REPO, "_config.yml"), encoding="utf-8").read()
    site_title = (re.search(r"^title:\s*(.+)$", cfg, re.M) or [None, "QuestHub"])[1].strip()
    pages = glob.glob(os.path.join(REPO, "*.html")) + glob.glob(os.path.join(REPO, "quests", "*.html"))
    n = 0
    for path in sorted(pages):
        raw = open(path, encoding="utf-8").read()
        m = FRONT.match(raw)
        if not m:
            continue  # only front-matter pages are Jekyll pages
        fm, body = m.group(1), m.group(2)
        rel = perma(path, fm)
        dest = os.path.join(OUT, rel)
        os.makedirs(os.path.dirname(dest), exist_ok=True)
        open(dest, "w", encoding="utf-8").write(render(layout, body, fm, site_title))
        n += 1
    if os.path.isdir(os.path.join(REPO, "assets")):
        shutil.copytree(os.path.join(REPO, "assets"), os.path.join(OUT, "assets"))
    return n

def serve(port, do_open):
    handler = functools.partial(http.server.SimpleHTTPRequestHandler, directory=OUT)
    socketserver.TCPServer.allow_reuse_address = True
    with socketserver.TCPServer(("127.0.0.1", port), handler) as httpd:
        url = f"http://127.0.0.1:{port}/"
        print(f"\n  QuestHub preview → {url}")
        print(f"  Quest pages:       {url}quests/ampl/  (and 21 more)")
        print(f"  Ctrl-C to stop.\n")
        if do_open:
            threading.Timer(0.6, lambda: webbrowser.open(url)).start()
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n  stopped.")

if __name__ == "__main__":
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=8777)
    ap.add_argument("--no-open", action="store_true")
    a = ap.parse_args()
    count = build()
    print(f"  Rendered {count} pages into _preview/")
    serve(a.port, not a.no_open)
