#!/bin/sh
# Fetch UniMorph urd (English-Wiktionary lineage, CC BY-SA; read at test
# time only, never redistributed) and convert it for the Urdu golden
# harness. UniMorph urd shares kaikki's Wiktionary lineage, so it is a
# documented spot-check oracle, not the independent gate — see
# docs/urd/oracles.md.
#
# Pinned to a commit and checksummed: a silent upstream change would
# shift the gold standard.
set -e
mkdir -p data/urd
curl -sL "https://raw.githubusercontent.com/unimorph/urd/17b2fb34fac79ba7bc4b90f8e10e6a25bac3d396/urd" \
  -o data/urd/unimorph-urd.txt
echo "3a3129d858c7c02d0ec5c8971f5b60842813dbe2513efaf7effd22842dcff4a3  data/urd/unimorph-urd.txt" \
  | shasum -a 256 -c -
python3 scripts/urd/unimorph_to_tsv.py data/urd/unimorph-urd.txt > data/urd/unimorph.tsv
wc -l data/urd/unimorph.tsv
