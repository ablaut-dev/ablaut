#!/bin/sh
# Fetch UniMorph ben (Batsuren & Cotterell; Wikipedia/Wiktionary lineage,
# CC BY-SA 3.0; read at test time only, never redistributed) and convert
# it for the Bengali golden harness. This is the primary (and, given the
# oracle situation, sole scored) oracle — see docs/ben/oracles.md.
#
# Pinned to a commit and checksummed: a silent upstream change would
# shift the gold standard.
set -e
mkdir -p data/ben
curl -sL "https://raw.githubusercontent.com/unimorph/ben/55a44fa60e9b7a3a5ff7acbed66b07571ceefa79/ben" \
  -o data/ben/unimorph-ben.txt
echo "b0f1e2d005bce183dfcc2a570de1ce3d4d9a17feafa6c222b3f72b3e6ff894b2  data/ben/unimorph-ben.txt" \
  | shasum -a 256 -c -
python3 scripts/ben/unimorph_to_tsv.py data/ben/unimorph-ben.txt > data/ben/unimorph.tsv
wc -l data/ben/unimorph.tsv
