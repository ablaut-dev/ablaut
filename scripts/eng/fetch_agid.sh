#!/bin/sh
# Fetch AGID (Automatically Generated Inflection Database) from its
# source repository (SCOWL/wordlist project; liberal wordlist license).
set -e
mkdir -p data/eng
curl -sL "https://raw.githubusercontent.com/en-wl/wordlist/master/agid/infl.txt" \
  -o data/eng/agid-infl.txt
python3 scripts/eng/agid_to_tsv.py data/eng/agid-infl.txt > data/eng/agid.tsv
wc -l data/eng/agid.tsv
