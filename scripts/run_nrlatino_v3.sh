#!/bin/bash
# run_nrlatino_gen_upload.sh - Genera shorts NR Latino y sube a YouTube
export $(grep -v '^#' /Users/nramos/.hermes/.env | xargs) 2>/dev/null
cd /Users/nramos/nr-latino
python3 orchestrate_nrlatino_v2.py --quiet >> /Users/nramos/nr-latino/cron.log 2>&1
echo "[$(date)] 💃 NR Latino generado" >> /tmp/nrmusic_upload.log
bash /Users/nramos/.hermes/scripts/run_nrmusic_upload.sh latino
