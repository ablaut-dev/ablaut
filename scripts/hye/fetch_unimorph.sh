#!/bin/sh
# Fetch UniMorph hye (English-Wiktionary lineage, CC BY-SA 3.0; read at
# test time only, never redistributed) and convert it for the Armenian
# golden harness. This is the Wiktionary leg of the oracle pair.
#
# Pinned to a commit and checksummed: a silent upstream change would
# shift the gold standard.
set -e
mkdir -p data/hye
curl -sL "https://raw.githubusercontent.com/unimorph/hye/8d404710b0451dae1576d3aaf77c91e638eff66b/hye" \
  -o data/hye/unimorph-hye.txt
echo "850ca7b5fa06398795edc35daf0e202adab8765946b8920ab3307c3fb22ce091  data/hye/unimorph-hye.txt" \
  | shasum -a 256 -c -
python3 scripts/hye/unimorph_to_tsv.py data/hye/unimorph-hye.txt > data/hye/unimorph.tsv
wc -l data/hye/unimorph.tsv
