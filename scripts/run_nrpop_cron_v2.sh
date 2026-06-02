#!/bin/bash
export $(grep -v '^#' /Users/nramos/.hermes/.env | xargs) 2>/dev/null
cd /Users/nramos/nr-pop
python3 orchestrate_nrpop_v2.py --quiet >> /Users/nramos/nr-pop/cron.log 2>&1
