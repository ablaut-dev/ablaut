#!/bin/sh
# Fetch the FreeLing Spanish verb entries (LGPL-LR, TALP-UPC) and convert
# them for the Spanish golden harness. Unlike UniMorph spa (a truncated
# Wiktionary scrape), FreeLing descends from the Spanish Resource Grammar
# and is a genuinely independent oracle vs kaikki.
#
# Pinned to a commit and checksummed: a silent upstream change would
# shift the gold standard.
set -e
mkdir -p data/spa
curl -sL "https://raw.githubusercontent.com/TALP-UPC/FreeLing/2479f4bb891bb0ea45b9456747016663aac95215/data/es/dictionary/entries/MM.verb" \
  -o data/spa/freeling-verbs.src
echo "72a0dafe3df20e2d23a6f5b823240aaeb01f4715b9cf373970991da086e18033  data/spa/freeling-verbs.src" \
  | shasum -a 256 -c -
python3 scripts/spa/freeling_to_tsv.py data/spa/freeling-verbs.src > data/spa/freeling.tsv
wc -l data/spa/freeling.tsv
