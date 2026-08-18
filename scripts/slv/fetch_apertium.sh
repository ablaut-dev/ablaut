#!/bin/sh
# Fetch the apertium-slv monodix (GPL-2.0; read at test time only,
# never redistributed) and expand its verb paradigms to the shared TSV.
set -e
mkdir -p data/slv
curl -sL "https://raw.githubusercontent.com/apertium/apertium-slv/master/apertium-slv.slv.dix" \
  -o data/slv/apertium-slv.slv.dix
python3 scripts/slv/apertium_to_tsv.py data/slv/apertium-slv.slv.dix > data/slv/apertium.tsv
wc -l data/slv/apertium.tsv
