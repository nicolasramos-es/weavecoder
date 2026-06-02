#!/bin/bash
# Daily backup script for hermes-config
# Runs at 06:00 UTC every day

REPO_DIR="$HOME/hermes-config"
cd "$REPO_DIR" || exit 1

# Pull latest to avoid conflicts
git pull origin main --rebase 2>/dev/null

# Update config, skills, memory
cp "$HOME/.hermes/config.yaml" config/
cp "$HOME/.hermes/MEMORY.md" memory/ 2>/dev/null || true
cp "$HOME/.hermes/USER.md" memory/ 2>/dev/null || true
cp -r "$HOME/.hermes/scripts/" . 2>/dev/null || true
cp "$HOME/.hermes/cron/jobs.json" cron/ 2>/dev/null || true

# Update conversation summaries
mkdir -p gemini-conversations/resumenes chatgpt-conversations/resumenes
cp "$HOME/.hermes/gemini-conversations/resumenes/"*.md gemini-conversations/resumenes/ 2>/dev/null || true
cp "$HOME/.hermes/chatgpt-conversations/resumenes/STATS.md" chatgpt-conversations/ 2>/dev/null || true

# Commit and push
git add -A
if ! git diff --cached --quiet; then
    git commit -m "Daily backup $(date +'%Y-%m-%d %H:%M')"
    git push origin main
    echo "Backup completed: $(date)"
else
    echo "No changes to backup"
fi