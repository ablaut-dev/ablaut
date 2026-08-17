#!/bin/sh
# Fetch SALDO's morphology (Språkbanken, CC BY; LMF XML, ~254 MB) and
# convert it for the Swedish golden harness.
set -e
mkdir -p data/swe
curl -sL "https://svn.spraakbanken.gu.se/sb-arkiv/pub/lmf/saldom/saldom.xml" \
  -o data/swe/saldom.xml
python3 scripts/swe/saldo_to_tsv.py data/swe/saldom.xml > data/swe/saldo.tsv
wc -l data/swe/saldo.tsv
