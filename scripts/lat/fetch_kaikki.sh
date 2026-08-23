#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Latin verb extraction (CC BY-SA).
set -e
mkdir -p data/lat
curl -sL "https://kaikki.org/dictionary/Latin/pos-verb/kaikki.org-dictionary-Latin-by-pos-verb.jsonl" \
  -o data/lat/kaikki-verbs.jsonl
python3 scripts/lat/kaikki_to_tsv.py data/lat/kaikki-verbs.jsonl > data/lat/kaikki.tsv
wc -l data/lat/kaikki.tsv
