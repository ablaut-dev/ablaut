#!/bin/sh
# Fetch UniMorph mkd (Wiktionary lineage) and convert it for the golden
# harness. The independent leg is the hand-built apertium-mkd FST.
set -e
mkdir -p data/mkd
curl -sL "https://raw.githubusercontent.com/unimorph/mkd/master/mkd" \
  -o data/mkd/unimorph-mkd.tsv
python3 scripts/mkd/unimorph_to_tsv.py data/mkd/unimorph-mkd.tsv > data/mkd/unimorph.tsv
wc -l data/mkd/unimorph.tsv
