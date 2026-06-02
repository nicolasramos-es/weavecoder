#!/bin/bash
export $(grep -v '^#' /Users/nramos/.hermes/.env | xargs) 2>/dev/null
cd /Users/nramos/nr-latino
python3 orchestrate_nrlatino_v2.py --quiet >> /Users/nramos/nr-latino/cron.log 2>&1
