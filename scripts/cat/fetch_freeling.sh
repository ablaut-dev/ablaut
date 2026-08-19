#!/bin/sh
# Fetch the FreeLing Catalan verb entries (LGPL-LR, TALP-UPC) and convert
# them for the Catalan golden harness. FreeLing descends from the Spanish
# Resource Grammar family and is a genuinely independent oracle vs kaikki
# (which is Wiktionary-derived) — the same pairing as Spanish.
#
# Pinned to a commit and checksummed: a silent upstream change would shift
# the gold standard.
set -e
mkdir -p data/cat
curl -sL "https://raw.githubusercontent.com/TALP-UPC/FreeLing/2479f4bb891bb0ea45b9456747016663aac95215/data/ca/dictionary/entries/verbs" \
  -o data/cat/freeling-ca.verbs
echo "70da6ae8f2b116920cb3ec61e2e0b0d82848501a86332f66c7d550d99dd2d996  data/cat/freeling-ca.verbs" \
  | shasum -a 256 -c -
python3 scripts/cat/freeling_to_tsv.py data/cat/freeling-ca.verbs > data/cat/freeling.tsv
wc -l data/cat/freeling.tsv
