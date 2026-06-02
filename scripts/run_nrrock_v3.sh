#!/bin/bash
# run_nrrock_gen_upload.sh - Genera shorts NR Rock y sube a YouTube
export $(grep -v '^#' /Users/nramos/.hermes/.env | xargs) 2>/dev/null
cd /Users/nramos/nr-rock
python3 orchestrate_nrrock_v2.py --quiet >> /Users/nramos/nr-rock/cron.log 2>&1
echo "[$(date)] 🎸 NR Rock generado" >> /tmp/nrmusic_upload.log
bash /Users/nramos/.hermes/scripts/run_nrmusic_upload.sh rock
