#!/bin/bash
# run_nrpop_gen_upload.sh - Genera shorts NR Pop y sube a YouTube
export $(grep -v '^#' /Users/nramos/.hermes/.env | xargs) 2>/dev/null
cd /Users/nramos/nr-pop
python3 orchestrate_nrpop_v2.py --quiet >> /Users/nramos/nr-pop/cron.log 2>&1
echo "[$(date)] 🎵 NR Pop generado" >> /tmp/nrmusic_upload.log
# Subir después de generar
bash /Users/nramos/.hermes/scripts/run_nrmusic_upload.sh pop
