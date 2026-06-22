# Result Report — Finance &amp; Economics Quest Pages

Category: **Finance & Economics** · Eyebrow: `Finance &amp; Economics · Quest`

## Pages created

| Title | Permalink | Dot / Stage | Notes |
|-------|-----------|-------------|-------|
| Personal Regenerative Endowment | `/quests/personal-regenerative-endowment/` | green / Growing | `stage-growing` chip; green-light seed |
| Funding Flow | `/quests/funding-flow/` | gold / Sprouting | gold-light seed; default chip |
| flexGmbH | `/quests/flexgmbh/` | dormant / Dormant | sand seed + border; default chip |
| Fractal Seed Investment Fund | `/quests/fractal-seed-fund/` | gold / Sprouting | gold-light seed; default chip; Parent chip "under SoFin" |

Files written under `quests/`:
- `quests/personal-regenerative-endowment.html`
- `quests/funding-flow.html`
- `quests/flexgmbh.html`
- `quests/fractal-seed-fund.html`

## Approach

- Copied the `quests/ampl.html` gold-standard structure and scoped `<style>` block verbatim.
- Applied the seed-stage mapping per dot color:
  - green → `stage-growing` chip + `.seed { background: var(--qh-green-light) }`
  - gold → default chip + `.seed { background: var(--qh-gold-light) }`
  - dormant → default chip + `.seed { background: var(--qh-sand); border: 1px solid var(--qh-border) }`
- Each page has the four sections (The seed / Why it matters / What building it out looks like / Status), grounded in the card description with no invented facts, figures, partners, or financial claims.
- Reused `--qh-*` CSS vars only; no hardcoded hex.
- `index.html` was NOT edited (left to orchestrator). Nothing installed, pushed, or deployed.

## Checker result

`python3 scripts/check-quest-pages.py` from repo root: **all 15 pages PASS** (15/15 OK), including the 4 Finance & Economics pages above.

```
PASS  quests/flexgmbh.html
PASS  quests/fractal-seed-fund.html
PASS  quests/funding-flow.html
PASS  quests/personal-regenerative-endowment.html
```

## Flags

None. All four files pass front-matter and HTML well-formedness checks.
