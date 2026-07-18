#!/bin/bash
# Auto-sync hermes-config to GitHub
# Called by cron every 30 minutes and by git hooks

REPO_DIR="$HOME/hermes-config"
cd "$REPO_DIR" || exit 1

# Sync config files from live locations
cp "$HOME/.hermes/config.yaml" config/config.yaml

# Sync profile configs
mkdir -p config/profiles
for p in "$HOME/.hermes/profiles/"*/; do
    name=$(basename "$p")
    if [ -f "$p/config.yaml" ]; then
        cp "$p/config.yaml" "config/profiles/$name.yaml"
    fi
done

# Sync cron jobs
cp "$HOME/.hermes/cron/jobs.json" cron/ 2>/dev/null || true

# Sync skills list
ls "$HOME/.hermes/skills/" > config/skills_list.txt 2>/dev/null || true

# Commit and push if there are changes
git add -A
if ! git diff --cached --quiet; then
    git commit -m "auto-sync $(date +'%Y-%m-%d %H:%M')"
    git push origin main 2>/dev/null
fi
