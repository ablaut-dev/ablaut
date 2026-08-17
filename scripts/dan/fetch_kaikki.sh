#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Danish verb extraction (CC BY-SA).
set -e
mkdir -p data/dan
curl -sL "https://kaikki.org/dictionary/Danish/pos-verb/kaikki.org-dictionary-Danish-by-pos-verb.jsonl" \
  -o data/dan/kaikki-verbs.jsonl
python3 scripts/dan/kaikki_to_tsv.py data/dan/kaikki-verbs.jsonl > data/dan/kaikki.tsv
wc -l data/dan/kaikki.tsv
