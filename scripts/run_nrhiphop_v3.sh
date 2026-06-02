#!/bin/bash
# run_nrhiphop_gen_upload.sh - Genera shorts NR Hip-Hop y sube a YouTube
export $(grep -v '^#' /Users/nramos/.hermes/.env | xargs) 2>/dev/null
cd /Users/nramos/nr-hiphop
python3 orchestrate_nrhiphop_v2.py --quiet >> /Users/nramos/nr-hiphop/cron.log 2>&1
echo "[$(date)] 🎤 NR Hip-Hop generado" >> /tmp/nrmusic_upload.log
bash /Users/nramos/.hermes/scripts/run_nrmusic_upload.sh hiphop
