#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Spanish verb extraction (CC BY-SA)
# and convert it for the Spanish golden harness. Large: ~700 MB.
set -e
mkdir -p data/spa
curl -sL "https://kaikki.org/dictionary/Spanish/pos-verb/kaikki.org-dictionary-Spanish-by-pos-verb.jsonl" \
  -o data/spa/kaikki-verbs.jsonl
python3 scripts/spa/kaikki_to_tsv.py data/spa/kaikki-verbs.jsonl > data/spa/kaikki.tsv
wc -l data/spa/kaikki.tsv
