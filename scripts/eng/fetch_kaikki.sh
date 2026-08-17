#!/bin/sh
# Fetch the kaikki.org (Wiktextract) English verb extraction (CC BY-SA).
set -e
mkdir -p data/eng
curl -sL "https://kaikki.org/dictionary/English/pos-verb/kaikki.org-dictionary-English-by-pos-verb.jsonl" \
  -o data/eng/kaikki-verbs.jsonl
python3 scripts/eng/kaikki_to_tsv.py data/eng/kaikki-verbs.jsonl > data/eng/kaikki.tsv
wc -l data/eng/kaikki.tsv
