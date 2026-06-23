# Quest Pages — Deploy Result Report

## What shipped
22 rich quest detail pages (top-4 categories: Governance, Technology, Society, Finance),
live at **`https://questhub.eco/q/<slug>/`**.

## Architecture reality (discovered mid-task)
- questhub.eco is served by the **Kidur Rust app** (`questhub-server`, systemd `questhub.service`,
  reverse-proxied by Caddy from `localhost:3000`), NOT GitHub Pages. The Jekyll repo deploys to
  GitHub Pages, which the domain does not point at.
- The live app already renders `/quest/<id>` detail pages from an oplog
  (`/srv/questhub/data/kidur.jsonl`, 95 quests), but content is thin (one-line, HTML-escaped,
  single `<p>` — see `server/src/handlers/pages.rs` `html_escape` + `quest_detail.html`).
- Rebuilding the app to render rich content is **blocked**: the `kidur-*` crate sources are not
  on this machine and the server has no Rust toolchain. See [[project-questhub-deploy]] memory.

## Earliest-milestone path taken: Caddy static fallback (no rebuild)
1. `scripts/build-static-quests.py` renders `quests/*.html` into self-contained static pages
   (`dist-quests/<slug>/index.html`) reusing the verified `assets/css/style.css`.
2. Deployed to `/srv/questhub-static/quests/` (additive; nothing else touched).
3. Added Caddy route `handle_path /q/* { root * /srv/questhub-static/quests; file_server }`
   before the catch-all reverse-proxy. `caddy validate` passed; `systemctl reload caddy`.

## Rollback points
- Live Caddyfile pre-edit backup: `/home/almalinux/Caddyfile.bak.qstatic.20260622_222037`
  (sha256 `9d6000cc…`). Rollback = restore it + `systemctl reload caddy` + `rm -rf
  /srv/questhub-static/quests`. App binary + `kidur.jsonl` were never touched.
- Repo: tag `quest-pages-v1-known-good` (pre-layout-change state).

## Verification
- `https://questhub.eco/q/ampl/`, `/q/signal-orbit/`, `/q/refugees-valued-residents/`,
  `/q/assets/style.css` → all HTTP 200 (public).
- App home unchanged (still "An Inventory of Human Aspiration", served by the Kidur app).
- Browser-rendered: correct palette, seed-stage colors, 4 narrative sections, nav/footer.

## Open / next
- Full app integration (homepage cards linking to rich pages; rich rendering inside the Kidur
  app at `/quest/<id>`) needs the `kidur-*` source to rebuild the binary — deferred.
- The other ~21 of 95 live quests (bottom categories) have no rich page yet.
