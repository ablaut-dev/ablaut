#!/bin/sh
# Fetch UniMorph tel (English-Wiktionary lineage, CC BY-SA 3.0; read at
# test time only, never redistributed) and convert it for the Telugu
# golden harness. This is the primary oracle.
#
# Pinned to a commit and checksummed: a silent upstream change would
# shift the gold standard.
set -e
mkdir -p data/tel
curl -sL "https://raw.githubusercontent.com/unimorph/tel/551f60f5f43448e4503580e8dd6d338c7b5a7c6e/tel" \
  -o data/tel/unimorph-tel.txt
echo "369009d424e9d9ab81d85d20c71e36a8ebfb1323cc2b29b8bff2238f84cec78d  data/tel/unimorph-tel.txt" \
  | shasum -a 256 -c -
python3 scripts/tel/unimorph_to_tsv.py data/tel/unimorph-tel.txt > data/tel/unimorph.tsv
wc -l data/tel/unimorph.tsv
