#!/bin/bash
export $(grep -v '^#' /Users/nramos/.hermes/.env | xargs) 2>/dev/null
cd /Users/nramos/nr-rock
python3 orchestrate_nrrock_v2.py --quiet >> /Users/nramos/nr-rock/cron.log 2>&1
