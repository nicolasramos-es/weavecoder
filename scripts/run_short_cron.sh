#!/bin/bash
# The Time Gazer - Cron de producción automática (cada 2h)
export PATH="/Users/nramos/.local/bin:/usr/local/bin:/usr/bin:/bin:$PATH"
cd /Users/nramos/.hermes/video-producer
python3 orchestrate_short.py 2>&1