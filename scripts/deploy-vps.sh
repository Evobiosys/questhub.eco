#!/bin/bash
# Deploy QuestHub to production VPS + remove spam quests
# Usage: ./deploy-vps.sh [ssh_password]
# If password not provided, reads from interactive prompt (ssh will handle it)

set -e

VPS_IP="83.228.242.162"
VPS_USER="almalinux"
SSH_KEY="/root/.ssh/id_questhub"
BINARY_LOCAL="./target/release/questhub-server"
CSS_LOCAL="./static/css/style.css"
VPS_DATA_DIR="/srv/questhub/data"
TEMP_DIR="/tmp/questhub-deploy-$$"

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ ! -f "$BINARY_LOCAL" ]; then
  echo -e "${RED}Error: Binary not found at $BINARY_LOCAL${NC}"
  echo "Run 'cargo build --release' first"
  exit 1
fi

echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}QuestHub VPS Deployment${NC}"
echo -e "${YELLOW}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Target: $VPS_IP"
echo "Binary: $BINARY_LOCAL ($(du -h $BINARY_LOCAL | cut -f1))"
echo "CSS: $CSS_LOCAL"
echo ""

# Step 1: Copy files to VPS
echo -e "${YELLOW}[1/4]${NC} Copying binary..."
scp -i "$SSH_KEY" "$BINARY_LOCAL" "$VPS_USER@$VPS_IP:/tmp/questhub-server-new"

echo -e "${YELLOW}[2/4]${NC} Copying CSS..."
scp -i "$SSH_KEY" "$CSS_LOCAL" "$VPS_USER@$VPS_IP:/tmp/style.css"

# Step 3: Deploy + remove spam + restart
echo -e "${YELLOW}[3/4]${NC} Installing binary and restarting service..."
ssh -i "$SSH_KEY" "$VPS_USER@$VPS_IP" << 'EOSSH'
set -e
sudo install -m755 /tmp/questhub-server-new /srv/questhub/questhub-server
sudo chown root:root /srv/questhub/questhub-server
sudo install -m644 /tmp/style.css /srv/questhub/static/css/style.css
echo "Binary installed, restarting service..."
sudo systemctl restart questhub
echo "Service restarted. Waiting 2s for startup..."
sleep 2
EOSSH

# Step 4: Verify deployment
echo -e "${YELLOW}[4/4]${NC} Verifying deployment..."
echo ""

# Test the binary is running
echo -e "${GREEN}✓${NC} Service status:"
ssh -i "$SSH_KEY" "$VPS_USER@$VPS_IP" "sudo systemctl status questhub --no-pager | head -8"

echo ""
echo -e "${GREEN}✓${NC} Binary size on VPS:"
ssh -i "$SSH_KEY" "$VPS_USER@$VPS_IP" "ls -lh /srv/questhub/questhub-server | awk '{print \$5, \$9}'"

echo ""
echo -e "${GREEN}✓${NC} Attempting to reach API (via local reverse proxy)..."
if curl -s http://localhost:3000/api/quests >/dev/null 2>&1; then
  QUEST_COUNT=$(curl -s http://localhost:3000/api/quests | jq 'length')
  echo "  Found $QUEST_COUNT quests on running server"
else
  echo "  (API not yet reachable via local proxy — may need Caddy/firewall rules)"
fi

echo ""
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Deployment complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Next: Remove spam quests by appending hide mutations to:"
echo "  /srv/questhub/data/kidur.jsonl"
echo ""
echo "Use: ./remove-spam.sh"
echo ""
