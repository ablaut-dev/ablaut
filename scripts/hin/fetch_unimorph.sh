#!/bin/sh
# Fetch UniMorph hin (Batsuren & Cotterell; English-Wiktionary lineage,
# CC BY-SA 3.0; read at test time only, never redistributed) and convert
# it for the Hindi golden harness. This is one leg of the oracle pair.
#
# Pinned to a commit and checksummed: a silent upstream change would
# shift the gold standard.
set -e
mkdir -p data/hin
curl -sL "https://raw.githubusercontent.com/unimorph/hin/ae6231f736fa5a08205c768ca23ff6c19e80d25e/hin" \
  -o data/hin/unimorph-hin.txt
echo "3c9228053925a69fa5c20c5bafc880268c294afd8d267504258ee405597fb18a  data/hin/unimorph-hin.txt" \
  | shasum -a 256 -c -
python3 scripts/hin/unimorph_to_tsv.py data/hin/unimorph-hin.txt > data/hin/unimorph.tsv
wc -l data/hin/unimorph.tsv
