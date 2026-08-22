#!/bin/sh
# Fetch UniMorph swc (Swahili; English-Wiktionary lineage, CC BY-SA 3.0;
# read at test time only, never redistributed) and convert it for the
# Swahili golden harness. This is one leg of the oracle pair.
#
# Pinned to a commit and checksummed: a silent upstream change would
# shift the gold standard.
set -e
mkdir -p data/swa
curl -sL "https://raw.githubusercontent.com/unimorph/swc/02a2bdec0e5cb0dc93b6ef11db4d54a82c34b224/swc" \
  -o data/swa/unimorph-swc.txt
echo "fed7a61f279f5f1db3216cf33aa72cea9cb16fdf175d884508c187e03ef4a924  data/swa/unimorph-swc.txt" \
  | shasum -a 256 -c -
python3 scripts/swa/swc_to_tsv.py data/swa/unimorph-swc.txt > data/swa/swc.tsv
wc -l data/swa/swc.tsv
