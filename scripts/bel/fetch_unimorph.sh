#!/bin/sh
# Fetch the UniMorph Belarusian inflection table (CC BY-SA), keep verbs,
# strip stress marks and normalise apostrophes so it intersects kaikki.
set -e
mkdir -p data/bel
curl -sL "https://raw.githubusercontent.com/unimorph/bel/master/bel" -o data/bel/unimorph-bel.tsv
python3 scripts/bel/unimorph_to_tsv.py data/bel/unimorph-bel.tsv > data/bel/unimorph.tsv
wc -l data/bel/unimorph.tsv
