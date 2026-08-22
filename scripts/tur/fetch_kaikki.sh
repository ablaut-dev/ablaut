#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Turkish verb extraction (CC BY-SA)
# and convert it for the Turkish golden harness. This is the Wiktionary
# leg of the oracle pair. kaikki is a rolling dump, so it is not
# commit-pinned; see docs/tur/oracles.md for the parsing caveats — the
# person/number tags in the Turkish tables are scrambled, so the
# converter reads person from cell position, not from the tags.
set -e
mkdir -p data/tur
curl -sL "https://kaikki.org/dictionary/Turkish/pos-verb/kaikki.org-dictionary-Turkish-by-pos-verb.jsonl" \
  -o data/tur/kaikki-verbs.jsonl
python3 scripts/tur/kaikki_to_tsv.py data/tur/kaikki-verbs.jsonl > data/tur/kaikki.tsv
wc -l data/tur/kaikki.tsv
