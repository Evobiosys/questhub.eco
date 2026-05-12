# QuestHub VPS Deployment - Manual Commands

If the deployment scripts don't work or you prefer manual control, use these commands directly.

## Prerequisites
- SSH key at `/root/.ssh/id_questhub`
- SSH password for `almalinux` user (VPS requires both key + password)
- VPS IP: `83.228.242.162`

## Full Deployment Sequence

### 1. Verify Binary
```bash
ls -lh /root/projects/questhub.eco/target/release/questhub-server
# Should show ~6.0M file, built recently
```

### 2. Copy Binary to VPS
```bash
scp -i /root/.ssh/id_questhub \
  /root/projects/questhub.eco/target/release/questhub-server \
  almalinux@83.228.242.162:/tmp/questhub-server-new

# Will prompt for SSH password
```

### 3. Copy CSS to VPS
```bash
scp -i /root/.ssh/id_questhub \
  /root/projects/questhub.eco/static/css/style.css \
  almalinux@83.228.242.162:/tmp/style.css

# Will prompt for SSH password again (or cached)
```

### 4. Install Binary and Restart Service
```bash
ssh -i /root/.ssh/id_questhub almalinux@83.228.242.162 << 'EOF'
# Install new binary
sudo install -m755 /tmp/questhub-server-new /srv/questhub/questhub-server
sudo chown root:root /srv/questhub/questhub-server

# Install CSS
sudo install -m644 /tmp/style.css /srv/questhub/static/css/style.css

# Restart service
sudo systemctl restart questhub

# Verify
sudo systemctl status questhub --no-pager

echo "Waiting for service startup..."
sleep 2

# Test API response
curl -s http://localhost:3000/api/quests | jq 'length'
EOF

# Will prompt for SSH password
```

### 5. Verify Binary is Running
```bash
curl -s http://83.228.242.162:3000/api/quests | jq '.[0]' | head -20
# Should return quest data (if accessible from your network)
```

## Spam Removal (After Binary Deploy)

### 1. Fetch Current Log
```bash
scp -i /root/.ssh/id_questhub \
  almalinux@83.228.242.162:/srv/questhub/data/kidur.jsonl \
  /tmp/kidur-current.jsonl
```

### 2. Identify Spam Quest IDs
```bash
for title in "Goldau" "Vaugondry" "Niederhelfenschwil" "Bydgoszcz"; do
  id=$(grep -o '"id":"[^"]*".*"content":"'"$title"'"' /tmp/kidur-current.jsonl | \
       head -1 | grep -o '"id":"[^"]*"' | cut -d'"' -f4)
  echo "$title → $id"
done
```

### 3. Create Hide Mutations
```bash
cat > /tmp/spam-mutations.jsonl << 'EOF'
{"seq":39,"ts":"2026-05-12T14:30:00Z","op":"hide_node","node_id":"<ID1>"}
{"seq":40,"ts":"2026-05-12T14:30:00Z","op":"hide_node","node_id":"<ID2>"}
{"seq":41,"ts":"2026-05-12T14:30:00Z","op":"hide_node","node_id":"<ID3>"}
{"seq":42,"ts":"2026-05-12T14:30:00Z","op":"hide_node","node_id":"<ID4>"}
EOF

# Replace <ID1>, <ID2>, etc. with actual UUIDs from step 2
```

### 4. Backup and Apply
```bash
ssh -i /root/.ssh/id_questhub almalinux@83.228.242.162 << 'EOF'
# Backup current log
sudo cp /srv/questhub/data/kidur.jsonl /srv/questhub/data/kidur.jsonl.backup-$(date +%Y%m%d-%H%M%S)

# Create temporary copy locally for mutation append
cp /srv/questhub/data/kidur.jsonl /tmp/kidur-mutations.jsonl

# Append mutations (read from stdin)
cat >> /tmp/kidur-mutations.jsonl

# Replace original with mutated version
sudo install -m644 /tmp/kidur-mutations.jsonl /srv/questhub/data/kidur.jsonl

# Restart service to reload log
sudo systemctl restart questhub

echo "Done. Spam quests hidden."
EOF
```

Then pipe in the mutations file:
```bash
cat /tmp/spam-mutations.jsonl | ssh -i /root/.ssh/id_questhub almalinux@83.228.242.162 \
  "sudo tee -a /srv/questhub/data/kidur.jsonl > /dev/null"
```

## Rollback (If Needed)

Restore previous binary and data:
```bash
ssh -i /root/.ssh/id_questhub almalinx@83.228.242.162 << 'EOF'
# Restore from backup
sudo cp /srv/questhub/data/kidur.jsonl.backup-* /srv/questhub/data/kidur.jsonl

# Or restore binary from previous commit on GitHub and rebuild
cd /srv/questhub
sudo git pull origin <previous-commit-hash>
sudo cargo build --release
sudo install -m755 target/release/questhub-server /srv/questhub/questhub-server

# Restart
sudo systemctl restart questhub
EOF
```

## Troubleshooting

### SSH Connection Refused
```bash
# Check if key-based auth works (no password)
ssh -i /root/.ssh/id_questhub -v almalinux@83.228.242.162 "echo OK"

# If fails with "Permission denied (publickey,password)": password required
# Try with explicit password prompt:
ssh -i /root/.ssh/id_questhub -v almalinux@83.228.242.162 "echo OK"
# When prompted: enter password
```

### Service Won't Start
```bash
ssh -i /root/.ssh/id_questhub almalinux@83.228.242.162 << 'EOF'
sudo systemctl status questhub
sudo journalctl -u questhub -n 50 --no-pager
EOF
```

### API Not Responding
```bash
ssh -i /root/.ssh/id_questhub almalinux@83.228.242.162 << 'EOF'
# Check if service is running
sudo systemctl is-active questhub

# Check if process exists
ps aux | grep questhub

# Check for port listener
sudo netstat -tlnp | grep 3000

# Check logs
sudo journalctl -u questhub -f
EOF
```

## Verification Checklist

After deployment:
- [ ] Binary is deployed (check file timestamp: `ls -l /srv/questhub/questhub-server`)
- [ ] Service is running (`sudo systemctl is-active questhub`)
- [ ] API responds (`curl http://localhost:3000/api/quests`)
- [ ] 52 quests are in the database (count with `curl http://localhost:3000/api/quests | jq 'length'`)
- [ ] interHarness quest has internal chip visible
- [ ] Email on /about is obfuscated until clicked
- [ ] Spam quests don't appear in quest list
- [ ] New submissions work (test via web form)
