#!/bin/sh
# Fetch the SGJP (Grammatical Dictionary of Polish) tab dump (2-clause BSD)
# and reduce it to the shared verb TSV.
set -e
mkdir -p data/pol
curl -sL "http://download.sgjp.pl/morfeusz/20250511/sgjp-20250511.tab.gz" -o data/pol/sgjp.tab.gz
gzip -dc data/pol/sgjp.tab.gz | python3 scripts/pol/sgjp_to_tsv.py /dev/stdin > data/pol/sgjp.tsv
wc -l data/pol/sgjp.tsv
