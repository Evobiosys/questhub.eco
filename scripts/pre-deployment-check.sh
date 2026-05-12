#!/bin/bash
# Pre-deployment verification checklist
# Run this before deploying to VPS to ensure everything is ready

set +e  # Don't exit on errors, just report them

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

passed=0
failed=0

check() {
  local desc="$1"
  local cmd="$2"

  if eval "$cmd" > /dev/null 2>&1; then
    echo -e "${GREEN}✓${NC} $desc"
    ((passed++))
  else
    echo -e "${RED}✗${NC} $desc"
    ((failed++))
  fi
}

warn() {
  local desc="$1"
  echo -e "${YELLOW}⚠${NC} $desc"
}

echo -e "${BLUE}═════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}QuestHub Pre-Deployment Verification${NC}"
echo -e "${BLUE}═════════════════════════════════════════════════════════════${NC}"
echo ""

echo -e "${YELLOW}1. Binary & Build${NC}"
check "Binary exists" "[ -f server/target/release/questhub-server ]"
check "Binary size > 5MB" "[ $(stat -f%z server/target/release/questhub-server 2>/dev/null || stat -c%s server/target/release/questhub-server) -gt 5000000 ]"
check "Binary is executable" "[ -x server/target/release/questhub-server ]"

echo ""
echo -e "${YELLOW}2. Code & Commits${NC}"
check "Git repo clean/committed" "git diff-index --quiet HEAD --"
check "Latest commit is deployment-related" "git log -1 --oneline | grep -qE 'deploy|spam|release'"
check "Remote is up-to-date" "git status | grep -q 'up to date'"

echo ""
echo -e "${YELLOW}3. Deployment Scripts${NC}"
check "deploy-vps.sh exists" "[ -f scripts/deploy-vps.sh ]"
check "deploy-vps.sh is executable" "[ -x scripts/deploy-vps.sh ]"
check "deploy-vps.sh syntax valid" "bash -n scripts/deploy-vps.sh"
check "remove-spam.sh exists" "[ -f scripts/remove-spam.sh ]"
check "remove-spam.sh is executable" "[ -x scripts/remove-spam.sh ]"
check "remove-spam.sh syntax valid" "bash -n scripts/remove-spam.sh"

echo ""
echo -e "${YELLOW}4. Documentation${NC}"
check "DEPLOYMENT-STATUS.md exists" "[ -f DEPLOYMENT-STATUS.md ]"
check "DEPLOYMENT-COMMANDS.md exists" "[ -f DEPLOYMENT-COMMANDS.md ]"
check "Handover doc exists" "[ -f /root/resources/chat-handovers/questhub-vps-deployment-ready.md ]"

echo ""
echo -e "${YELLOW}5. Dependencies & Tooling${NC}"
check "SSH key exists" "[ -f ~/.ssh/id_questhub ]"
check "SSH key is readable" "[ -r ~/.ssh/id_questhub ]"
check "jq is available" "which jq > /dev/null"
check "curl is available" "which curl > /dev/null"
check "scp is available" "which scp > /dev/null"

echo ""
echo -e "${YELLOW}6. Quest Data${NC}"
check "Local kidur.jsonl exists" "[ -f server/data/kidur.jsonl ]"
check "Local kidur.jsonl has entries" "[ $(wc -l < server/data/kidur.jsonl) -gt 20 ]"

echo ""
echo -e "${YELLOW}7. CSS & Static Files${NC}"
check "CSS file exists" "[ -f server/static/css/style.css ]"
check "CSS has internal chip styles" "grep -q 'qh-chip-internal' server/static/css/style.css"
check "Favicon exists" "[ -f server/static/favicon.svg ]"

echo ""
echo -e "${YELLOW}8. VPS Connectivity${NC}"
if [ -f ~/.ssh/id_questhub ]; then
  warn "SSH key auth will be tested (no password required for this check)"
  check "SSH key auth works" "ssh -i ~/.ssh/id_questhub -o BatchMode=yes -o ConnectTimeout=5 almalinux@83.228.242.162 'echo OK' 2>&1 | grep -q OK"
else
  warn "SSH key not found, skipping connectivity test"
fi

echo ""
echo -e "${BLUE}═════════════════════════════════════════════════════════════${NC}"
echo -e "Results: ${GREEN}$passed passed${NC}, ${RED}$failed failed${NC}"
echo -e "${BLUE}═════════════════════════════════════════════════════════════${NC}"
echo ""

if [ $failed -eq 0 ]; then
  echo -e "${GREEN}All checks passed! Ready for deployment.${NC}"
  echo ""
  echo "Next steps:"
  echo "  cd /root/projects/questhub.eco"
  echo "  ./scripts/deploy-vps.sh          # Deploy binary (prompts for SSH password)"
  echo "  ./scripts/remove-spam.sh         # Remove spam quests (prompts for SSH password)"
  echo ""
  exit 0
else
  echo -e "${RED}Some checks failed. Review above.${NC}"
  echo ""
  echo "Common issues:"
  echo "  - Binary not built: run 'cargo build --release' in server/"
  echo "  - Git not clean: run 'git status' and commit/restore changes"
  echo "  - SSH key missing: verify ~/.ssh/id_questhub exists"
  echo ""
  exit 1
fi
