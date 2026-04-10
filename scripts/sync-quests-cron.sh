#!/usr/bin/env bash
# sync-quests-cron.sh
# =====================================================================
# VPS cron wrapper: fetch Tana #quest nodes → update _data/quests.yml
# → commit + push to GitHub if anything changed.
#
# Setup on VPS:
#   1. Clone repo:  git clone git@github.com:Evobiosys/questhub.eco.git /srv/questhub.eco
#   2. Create secrets file at /root/.secrets/questhub (chmod 600):
#        export TANA_API_TOKEN="your-tana-token"
#        export TANA_WORKSPACE_ID="34saXxFYBC"   # optional, this is the default
#   3. Ensure git remote uses SSH deploy key (no password prompt):
#        git remote set-url origin git@github.com:Evobiosys/questhub.eco.git
#   4. Add cron (every 30 min):
#        crontab -e
#        */30 * * * * /srv/questhub.eco/scripts/sync-quests-cron.sh >> /var/log/questhub-sync.log 2>&1
# =====================================================================

set -euo pipefail

REPO_DIR="/srv/questhub.eco"
SECRETS_FILE="/root/.secrets/questhub"
LOG_TS="[$(date -Iseconds)]"

# ── Load secrets ──────────────────────────────────────────────────────
if [[ -f "$SECRETS_FILE" ]]; then
  # shellcheck source=/dev/null
  source "$SECRETS_FILE"
fi

if [[ -z "${TANA_API_TOKEN:-}" ]]; then
  echo "$LOG_TS ERROR: TANA_API_TOKEN not set (checked $SECRETS_FILE)" >&2
  exit 1
fi

export TANA_API_TOKEN
export TANA_WORKSPACE_ID="${TANA_WORKSPACE_ID:-34saXxFYBC}"

# ── Enter repo ────────────────────────────────────────────────────────
if [[ ! -d "$REPO_DIR/.git" ]]; then
  echo "$LOG_TS ERROR: $REPO_DIR is not a git repo" >&2
  exit 1
fi
cd "$REPO_DIR"

echo "$LOG_TS Starting quest sync (workspace: $TANA_WORKSPACE_ID)"

# Pull latest to avoid push conflicts
git pull --ff-only origin main 2>&1 | sed "s/^/$LOG_TS git: /"

# ── Fetch quests from Tana ────────────────────────────────────────────
python3 scripts/fetch-tana-quests.py --output _data/quests.yml 2>&1 | sed "s/^/$LOG_TS /"

# ── Commit + push if changed ──────────────────────────────────────────
if ! git diff --quiet _data/quests.yml; then
  git add _data/quests.yml
  git commit -m "sync: quests from Tana [$(date -u +%Y-%m-%dT%H:%MZ)]"
  git push origin main
  echo "$LOG_TS Pushed updated _data/quests.yml"
else
  echo "$LOG_TS No changes in quests.yml — skipping commit"
fi
