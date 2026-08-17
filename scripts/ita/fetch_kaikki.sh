#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Italian verb extraction (CC BY-SA)
# and convert it for the Italian golden harness.
set -e
mkdir -p data/ita
curl -sL "https://kaikki.org/dictionary/Italian/pos-verb/kaikki.org-dictionary-Italian-by-pos-verb.jsonl" \
  -o data/ita/kaikki-verbs.jsonl
python3 scripts/ita/kaikki_to_tsv.py data/ita/kaikki-verbs.jsonl > data/ita/kaikki.tsv
wc -l data/ita/kaikki.tsv
