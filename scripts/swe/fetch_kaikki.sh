#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Swedish verb extraction (CC BY-SA)
# and convert it for the Swedish golden harness.
set -e
mkdir -p data/swe
curl -sL "https://kaikki.org/dictionary/Swedish/pos-verb/kaikki.org-dictionary-Swedish-by-pos-verb.jsonl" \
  -o data/swe/kaikki-verbs.jsonl
python3 scripts/swe/kaikki_to_tsv.py data/swe/kaikki-verbs.jsonl > data/swe/kaikki.tsv
wc -l data/swe/kaikki.tsv
