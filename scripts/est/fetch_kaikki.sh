#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Estonian verb extraction (CC BY-SA).
set -e
mkdir -p data/est
curl -sL "https://kaikki.org/dictionary/Estonian/pos-verb/kaikki.org-dictionary-Estonian-by-pos-verb.jsonl" \
  -o data/est/kaikki-verbs.jsonl
python3 scripts/est/kaikki_to_tsv.py data/est/kaikki-verbs.jsonl > data/est/kaikki.tsv
wc -l data/est/kaikki.tsv
