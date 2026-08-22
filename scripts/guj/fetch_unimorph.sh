#!/bin/sh
# Fetch UniMorph guj (Batsuren & Cotterell; English-Wiktionary lineage,
# CC BY-SA 3.0; read at test time only, never redistributed) and convert
# it for the Gujarati golden harness. This is the primary oracle.
#
# Pinned to a commit and checksummed: a silent upstream change would
# shift the gold standard.
set -e
mkdir -p data/guj
curl -sL "https://raw.githubusercontent.com/unimorph/guj/f98348eea353912a61ca5d5af5c68c602a483ebe/guj" \
  -o data/guj/unimorph-guj.txt
echo "fb6437c6544b823c00c85d78e3ba614bec80d1ee8a8982770ff24181f6a503a3  data/guj/unimorph-guj.txt" \
  | shasum -a 256 -c -
python3 scripts/guj/unimorph_to_tsv.py data/guj/unimorph-guj.txt > data/guj/unimorph.tsv
wc -l data/guj/unimorph.tsv
