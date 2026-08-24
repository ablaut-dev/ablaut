#!/bin/sh
# Fetch UniMorph zul (Zulu; English-Wiktionary lineage, CC BY-SA 3.0;
# read at test time only, never redistributed) and convert it for the
# Zulu golden harness. This is the primary/CI leg of the oracle pair.
#
# Pinned to a commit and checksummed: a silent upstream change would
# shift the gold standard.
set -e
mkdir -p data/zul
curl -sL "https://raw.githubusercontent.com/unimorph/zul/cc7adc828d0ee63b282a44a105b11689ec5951da/zul" \
  -o data/zul/unimorph-zul.txt
echo "3762e371326fc74a1a7c513a38fd86eef00b1c14ca48148e119910c4b1e18450  data/zul/unimorph-zul.txt" \
  | shasum -a 256 -c -
python3 scripts/zul/unimorph_to_tsv.py data/zul/unimorph-zul.txt > data/zul/unimorph.tsv
wc -l data/zul/unimorph.tsv
