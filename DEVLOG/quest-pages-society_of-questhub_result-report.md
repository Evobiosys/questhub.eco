# Result Report — Quest Detail Pages: Society &amp; Politics

Category: **Society & Politics** · Eyebrow: `Society &amp; Politics · Quest`
Built by the Society & Politics agent, following `quest-detail-pages_of-questhub_handover.md`
and the `quests/ampl.html` gold-standard canary (scoped `<style>` block copied verbatim).

## Pages created (title → permalink)

| # | Title | Permalink | Dot / Stage | Seed style |
|---|-------|-----------|-------------|------------|
| 1 | Bridging Worldviews in Politics | `/quests/bridging-worldviews/` | green / Growing | `--qh-green-light`, `stage-growing` chip |
| 2 | Political Options Mapped, Autoupdated | `/quests/political-options-mapped/` | gold / Sprouting | `--qh-gold-light`, default chip |
| 3 | Geopolitical Game-Theoretic Machine | `/quests/geopolitical-game-theory/` | dormant / Dormant | `--qh-sand` + border, default chip |
| 4 | Regenerate Ukraine | `/quests/regenerate-ukraine/` | green / Growing | `--qh-green-light`, `stage-growing` chip |
| 5 | Enabling Refugees as Valued Residents | `/quests/refugees-valued-residents/` | gold / Sprouting | `--qh-gold-light`, default chip |
| 6 | Mirror EU Politicians to Mastodon/Bluesky | `/quests/mirror-eu-politicians/` | dormant / Dormant | `--qh-sand` + border, default chip |

Files written under `quests/`:
- `bridging-worldviews.html`
- `political-options-mapped.html`
- `geopolitical-game-theory.html`
- `regenerate-ukraine.html`
- `refugees-valued-residents.html`
- `mirror-eu-politicians.html`

## Conformance
- Front matter matches the canary: `layout: default`, `permalink: /quests/<slug>/`,
  `title: "<Title> | QuestHub"`, single-sentence `description:`.
- Each page: `.qh-quest-hero` (eyebrow + seed dot, `<h1>` verbatim title, `.lede`, `.qh-meta`
  chips) and `.qh-quest-body` with the four sections — The seed / Why it matters /
  What building it out looks like (a `<ul>` of concrete steps) / Status — plus the
  `&larr; Back to the garden` CTA.
- Seed-stage mapping applied per the handover table (green→Growing, gold→Sprouting,
  dormant→Dormant). No `index.html` edits. Only `--qh-*` CSS vars used; no hardcoded hex.
- HTML entities used (`&amp; &mdash; &rsquo;`). No Tana tags added (correctly excluded for site HTML).
- Content grounded in each card description and expanded without inventing facts, partners,
  dates, or numbers. Politically sensitive topics kept measured, non-partisan, and concrete,
  framed through the EvoBioSys synergy / omni-win lens. No editorializing against any group.

## Checker result
`python3 scripts/check-quest-pages.py` run from repo root: **all 6 pages PASS**
(full run reported 21/21 pages OK — every other agent's pages present at run time also passed).

## Flags
- None. All assigned pages validate.
