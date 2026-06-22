#!/usr/bin/env python3
"""
check-quest-pages.py — Ruby-free sanity check for QuestHub quest detail pages.

Validates each quests/*.html file the way Jekyll would consume it, without
needing a full Jekyll/Ruby build (local Ruby is 2.6, too old for modern Jekyll;
CI builds on Ruby 3.1). For each page it:
  1. Confirms YAML front matter exists with layout/permalink/title/description.
  2. Strips front matter, injects the body into _layouts/default.html at
     {{ content }}, and parses the whole document with html.parser to catch
     malformed/unclosed tags.
  3. Reports per-file PASS/FAIL.

Usage: python3 scripts/check-quest-pages.py
Exit code 0 = all pass, 1 = any failure.
"""
import os, re, sys, glob
from html.parser import HTMLParser

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LAYOUT = os.path.join(REPO, "_layouts", "default.html")
VOID = {"area","base","br","col","embed","hr","img","input","link","meta",
        "param","source","track","wbr"}

class WellFormed(HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.stack = []
        self.errors = []
    def handle_starttag(self, tag, attrs):
        if tag not in VOID:
            self.stack.append(tag)
    def handle_startendtag(self, tag, attrs):
        pass
    def handle_endtag(self, tag):
        if tag in VOID:
            return
        if tag in self.stack:
            # pop to the matching tag (tolerate optional-close tags like <p>, <li>)
            while self.stack and self.stack[-1] != tag:
                self.stack.pop()
            if self.stack:
                self.stack.pop()
        else:
            self.errors.append(f"stray </{tag}>")

FRONT = re.compile(r"^---\n(.*?)\n---\n(.*)$", re.S)

def check(path, layout_html):
    raw = open(path, encoding="utf-8").read()
    m = FRONT.match(raw)
    errs = []
    if not m:
        return ["missing/invalid front matter"]
    fm, body = m.group(1), m.group(2)
    for key in ("layout:", "permalink:", "title:", "description:"):
        if key not in fm:
            errs.append(f"front matter missing {key}")
    if "permalink: /quests/" not in fm:
        errs.append("permalink not under /quests/")
    rendered = layout_html.replace("{{ content }}", body)
    p = WellFormed()
    p.feed(rendered)
    p.close()
    if p.errors:
        errs.extend(p.errors)
    if p.stack:
        errs.append("unclosed tags: " + ", ".join(p.stack[-5:]))
    return errs

def main():
    layout_html = open(LAYOUT, encoding="utf-8").read()
    # neutralize liquid in layout so html.parser sees plain markup
    layout_html = re.sub(r"\{%.*?%\}", "", layout_html)
    layout_html = re.sub(r"\{\{(?!\s*content\s*\}\}).*?\}\}", "x", layout_html)
    pages = sorted(glob.glob(os.path.join(REPO, "quests", "*.html")))
    if not pages:
        print("no quest pages found")
        return 1
    bad = 0
    for path in pages:
        errs = check(path, layout_html)
        rel = os.path.relpath(path, REPO)
        if errs:
            bad += 1
            print(f"FAIL  {rel}")
            for e in errs:
                print(f"        - {e}")
        else:
            print(f"PASS  {rel}")
    print(f"\n{len(pages)-bad}/{len(pages)} pages OK")
    return 1 if bad else 0

if __name__ == "__main__":
    sys.exit(main())
