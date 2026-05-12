# QuestHub VPS Deployment Status

**Last Updated:** 2026-05-12 14:30 UTC

## Current State

### ✓ Completed
- **Binary built**: `target/release/questhub-server` (6.0M, built 2026-05-12 14:23 UTC)
- **Code committed**: Branch `master`, commit `8acf5ed` — "feat: spam defense, internal chip, email reveal, 52 new quests"
- **52 quests submitted** to live API via `scripts/submit-tana-quests.py`
  - 22 tech quests
  - 11 tools quests
  - 9 society/culture quests
  - 5 knowledge quests
  - 1 interHarness quest (with `parent_project="EvoBioSys"` → displays as internal chip)
- **Email reveal implemented**: JS-click pattern on `.email-reveal` class prevents scraper harvesting
- **Internal chip CSS**: Styled to show "internal" with EvoBioSys link on hover/focus
- **4-layer spam defense** deployed:
  1. Honeypot field (checks if website field is filled)
  2. Rate limiting per IP (10/hour, in-memory tracker)
  3. Time-trap (2s minimum delay via client-side PoW)
  4. SHA-256 PoW captcha (18-bit difficulty, self-hosted)

### ○ Pending — Blocked on SSH Access

#### 1. Deploy binary to VPS
- **Command**: `./scripts/deploy-vps.sh`
- **What it does**:
  - Copies binary via SCP to VPS
  - Copies CSS updates via SCP
  - Restarts questhub systemd service
  - Verifies service is running
- **Requires**: SSH password (VPS has `publickey,password` auth enabled)
- **Expected outcome**: Live site receives new binary with parent_project field support, internal chip rendering, and updated CSS

#### 2. Remove 4 spam quests
- **Known spam entries** (from manual review):
  - "Goldau" (ID: 019e0973)
  - "Vaugondry" (ID: 019df4c6)
  - "Niederhelfenschwil" (ID: 019de341)
  - "Bydgoszcz" (ID: 019dde0a)
- **Command**: `./scripts/remove-spam.sh`
- **What it does**:
  - Fetches current kidur.jsonl from VPS
  - Identifies node IDs for spam quests
  - Creates hide mutations
  - Backs up current log (`kidur.jsonl.backup-TIMESTAMP`)
  - Appends mutations to VPS log
  - Restarts service
- **Requires**: SSH access to VPS
- **Note**: Spam quests are already on live API (submitted before removal was planned). Once binary redeploys, they'll be hidden from the in-memory index.

## Next Steps

### Step 1: Provide SSH Password and Deploy
```bash
cd /root/projects/questhub.eco
./scripts/deploy-vps.sh
# SSH will prompt for password. Provide the almalinux user's password for VPS.
```

### Step 2: Remove Spam Quests
```bash
./scripts/remove-spam.sh
# This script will fetch the log, identify spam by title, and append hide mutations
```

### Step 3: Verify Deployment
- Check live site: https://questhub.eco/
- Try submitting a test quest
- Verify internal chip appears for interHarness quest
- Confirm email is obfuscated until clicked
- Confirm spam quests don't appear in quest list

## VPS Details
- **Host**: 83.228.242.162 (Infomaniak VPS Lite)
- **User**: almalinx (SSH key: ~/.ssh/id_questhub)
- **Service**: questhub (systemd)
- **Binary path**: /srv/questhub/questhub-server
- **Data**: /srv/questhub/data/kidur.jsonl
- **Static**: /srv/questhub/static/

## Files Modified (Pending Deployment)
- `src/handlers/quest.rs` — Added parent_project field, rate limit check, captcha verification
- `src/spam.rs` — New file with SpamGuard + PoW captcha
- `src/handlers/captcha.rs` — New endpoint for issuing challenges
- `src/main.rs` — Integrated SpamGuard, enabled ConnectInfo<SocketAddr>
- `src/routes.rs` — Added /api/captcha/challenge route
- `static/css/style.css` — Added .qh-chip-internal and .qh-chip-tooltip styles
- `templates/index.html` — Added internal chip rendering, email reveal JS, PoW solver
- `templates/about.html` — Added email reveal
- `templates/peak.html` — Added email reveal

## Notes
- Local kidur.jsonl has 38 entries (seed data). Live VPS log will have additional entries from the 52 submitted quests.
- The parent_project field is now part of QuestResponse JSON serialization, enabling the internal chip visible check.
- If deployment is delayed, quests remain on live API but without the internal chip rendering until binary updates.
