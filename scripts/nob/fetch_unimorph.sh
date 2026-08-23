#!/bin/sh
# Fetch UniMorph nob and convert it for the Norwegian golden harness.
# This is the Wiktionary-lineage leg of the oracle pair (the independent
# leg is the hand-built apertium-nob FST). Commit-pinned + checksummed so
# a silent upstream change can't move the gold standard. Read at test time
# only, never redistributed.
set -e
mkdir -p data/nob
curl -sL "https://raw.githubusercontent.com/unimorph/nob/master/nob" \
  -o data/nob/unimorph-nob.tsv
python3 scripts/nob/unimorph_to_tsv.py data/nob/unimorph-nob.tsv > data/nob/unimorph.tsv
wc -l data/nob/unimorph.tsv
