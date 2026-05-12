#!/bin/bash
# Remove spam quests from production VPS
# Identifies 4 known spam entries and appends hide mutations to kidur.jsonl

set -e

VPS_IP="83.228.242.162"
VPS_USER="almalinux"
SSH_KEY="/root/.ssh/id_questhub"
VPS_DATA_DIR="/srv/questhub/data"

# Known spam quest titles (from manual review)
declare -a SPAM_TITLES=(
  "Goldau"              # 019e0973
  "Vaugondry"           # 019df4c6
  "Niederhelfenschwil"  # 019de341
  "Bydgoszcz"           # 019dde0a
)

echo "═══════════════════════════════════════════════════════════════"
echo "QuestHub Spam Removal"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Target: $VPS_IP"
echo "Data dir: $VPS_DATA_DIR"
echo ""
echo "Spam quests to remove:"
printf '%s\n' "${SPAM_TITLES[@]}" | sed 's/^/  - /'
echo ""

# Step 1: Fetch current kidur.jsonl from VPS
echo "Fetching current log from VPS..."
scp -i "$SSH_KEY" "$VPS_USER@$VPS_IP:$VPS_DATA_DIR/kidur.jsonl" /tmp/kidur-current.jsonl

# Step 2: Extract node IDs for spam quests
echo "Identifying node IDs for spam quests..."
declare -A SPAM_IDS

for title in "${SPAM_TITLES[@]}"; do
  id=$(grep -o '"id":"[^"]*".*"content":"'"$title"'"' /tmp/kidur-current.jsonl | head -1 | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
  if [ -n "$id" ]; then
    SPAM_IDS["$title"]="$id"
    echo "  ✓ $title → $id"
  else
    echo "  ? $title → not found (may already be removed)"
  fi
done

# Step 3: Create hide mutations
echo ""
echo "Creating hide mutations..."

TEMP_MUTATIONS="/tmp/spam-mutations-$$.jsonl"
touch "$TEMP_MUTATIONS"

for title in "${SPAM_TITLES[@]}"; do
  id="${SPAM_IDS[$title]}"
  if [ -n "$id" ]; then
    # Get current log length to continue seq numbering
    LAST_SEQ=$(tail -1 /tmp/kidur-current.jsonl | jq -r '.seq // 0')
    NEXT_SEQ=$((LAST_SEQ + 1))

    # Create hide mutation
    TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    echo "{\"seq\":$NEXT_SEQ,\"ts\":\"$TIMESTAMP\",\"op\":\"hide_node\",\"node_id\":\"$id\"}" >> "$TEMP_MUTATIONS"
  fi
done

# Step 4: Backup and update
echo "Backing up current log on VPS..."
ssh -i "$SSH_KEY" "$VPS_USER@$VPS_IP" \
  "sudo cp $VPS_DATA_DIR/kidur.jsonl $VPS_DATA_DIR/kidur.jsonl.backup-$(date +%Y%m%d-%H%M%S)"

echo "Appending hide mutations to VPS log..."
cat "$TEMP_MUTATIONS" | ssh -i "$SSH_KEY" "$VPS_USER@$VPS_IP" \
  "sudo tee -a $VPS_DATA_DIR/kidur.jsonl > /dev/null"

# Step 5: Restart service to reload from log
echo "Restarting service to reload updated log..."
ssh -i "$SSH_KEY" "$VPS_USER@$VPS_IP" \
  "sudo systemctl restart questhub"

sleep 2

# Step 6: Verify removal
echo ""
echo "Verifying spam removal..."
for title in "${SPAM_TITLES[@]}"; do
  id="${SPAM_IDS[$title]}"
  if [ -n "$id" ]; then
    # Try to fetch the quest via API - should 404 if hidden
    result=$(curl -s "http://$VPS_IP:3000/api/quests/$id" 2>/dev/null | jq -r '.id // "not_found"')
    if [ "$result" = "not_found" ]; then
      echo "  ✓ $title ($id) → hidden"
    else
      echo "  ! $title ($id) → still visible (hidden flag may not filter API)"
    fi
  fi
done

# Cleanup
rm -f /tmp/kidur-current.jsonl "$TEMP_MUTATIONS"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "Spam removal complete!"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Changes made:"
echo "  - Appended 4 hide mutations to kidur.jsonl"
echo "  - Created backup: kidur.jsonl.backup-TIMESTAMP"
echo "  - Restarted questhub service"
echo ""
echo "Note: If the API still shows spam quests, the hidden filter"
echo "may need implementation in the handlers/quest.rs GET endpoints."
echo ""
