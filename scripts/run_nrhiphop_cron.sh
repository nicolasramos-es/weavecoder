#!/bin/bash
export $(grep -v '^#' /Users/nramos/.hermes/.env | xargs) 2>/dev/null
cd /Users/nramos/nr-hiphop
python3 orchestrate_nrhiphop_v3.py --quiet >> /Users/nramos/nr-hiphop/cron.log 2>&1
