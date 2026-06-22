# Handover — QuestHub Quest Detail Pages

## Goal
QuestHub's homepage (`index.html`) lists ~43 quests as static cards. We are giving each
quest its own **detail page**, following the proven SoTranscribe pattern (a card became a
full static page at `/sotranscribe/`). Your job: build the detail pages for **your assigned
category** as standalone HTML files under `quests/`.

You are ONE of several parallel agents. Each agent owns a disjoint set of quests and writes
only its own new files. **Do not edit `index.html`** — the orchestrator converts the homepage
cards into links in a single pass after all agents finish (avoids write conflicts).

## How the site works (do not re-investigate)
- Jekyll static site. A page = an HTML file with YAML front matter; `_layouts/default.html`
  wraps it (nav + footer + global CSS already included). You write only the page body + a
  scoped `<style>` block.
- Global palette lives in `assets/css/style.css` as CSS custom properties. **Reuse them** —
  never hardcode hex. Available vars include:
  `--qh-gold #b8860b`, `--qh-gold-light`, `--qh-green #4a7c59`, `--qh-green-light`,
  `--qh-green-pale`, `--qh-soil #3d2b1f`, `--qh-soil-muted`, `--qh-parchment`, `--qh-cream`,
  `--qh-sand`, `--qh-border`. Button class `.btn.btn-primary` exists globally.
- Local Ruby is 2.6 (too old to run Jekyll); CI builds on Ruby 3.1. **Do NOT install Ruby,
  gems, or anything else. Do NOT deploy / push.** Validate with the Python checker (below).

## GOLD-STANDARD REFERENCE — read it first
`quests/ampl.html` is the canary. Copy its structure exactly:
- Front matter: `layout: default`, `permalink: /quests/<slug>/`, `title: "<Quest> | QuestHub"`,
  `description:` (one sentence, ~150 chars, plain text, no quotes-inside-quotes issues).
- Outer `<div class="qh-quest" id="top">` containing a scoped `<style>` block (copy AMPL's
  CSS verbatim — it's already correct and self-contained), then:
  - `.qh-quest-hero`: eyebrow (`<category> · Quest` with a seed dot), `<h1>`, `.lede`
    (one expanded sentence), `.qh-meta` chips (Stage, Parent if any, "EvoBioSys network").
  - `.qh-quest-body`: four `<section>`s — **The seed**, **Why it matters**,
    **What building it out looks like** (a `<ul>` of concrete steps), **Status**.
  - Closing `.qh-quest-cta` with `<a href="/" class="btn btn-primary">&larr; Back to the garden</a>`.

### Seed-stage mapping (match the homepage card's dot color)
| Card dot | Stage label | eyebrow seed style | meta chip class |
|----------|-------------|--------------------|-----------------|
| green    | Growing     | `background: var(--qh-green-light)` | `stage-growing` (green pill) |
| gold     | Sprouting   | `background: var(--qh-gold-light)`  | (default pill) |
| dormant  | Dormant     | `background: var(--qh-sand); border: 1px solid var(--qh-border)` | (default pill) |

The AMPL canary shows the green/Growing variant. For gold set the eyebrow `.seed` background
to `var(--qh-gold-light)` and drop the `stage-growing` class from the chip. For dormant use the
sand background with border, label "Dormant".

## Content rules
- **Ground every claim in the card's description** (given per-quest below). Expand it into
  substantive, specific prose — but do NOT invent facts, partners, dates, or numbers that
  aren't implied. When in doubt, write about the *problem* and *what building it out looks
  like* rather than fabricating status.
- Voice: same as the homepage — calm, concrete, EvoBioSys "sovereign / regenerative / local-
  first" framing. No marketing fluff, no emoji in body text.
- Each page ~120–200 lines. Self-contained.
- Use `&amp; &mdash; &rsquo;` HTML entities like the existing pages do.
- Do NOT add Tana tags (e.g. `#ai-formulated`) — that rule is for Tana node writes, not site HTML.

## Validate before you finish
Run: `python3 scripts/check-quest-pages.py`
It checks front matter + HTML well-formedness for every page in `quests/`. Your pages must
PASS. (Other agents' pages may not exist yet when you run — only your own need to pass.)

## Output: result report
Write `DEVLOG/quest-pages-<category-slug>_of-questhub_result-report.md` listing each page you
created (title → permalink), the checker result for your files, and anything you flagged.

### Handback prompt (paste this back to the orchestrator when done)
`<category> quest pages done. Result at DEVLOG/quest-pages-<category-slug>_of-questhub_result-report.md. Resume: orchestrator converts index.html cards to links + runs full checker.`

---

## ASSIGNMENT (filled per-agent below)
The orchestrator fills in your category + quest list when dispatching you. Each quest entry
gives: **Title** (use verbatim in h1) · dot color · parent (if any) · card description (your
factual basis) · suggested slug.
