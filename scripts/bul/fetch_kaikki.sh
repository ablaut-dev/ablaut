#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Bulgarian verb extraction (CC BY-SA).
set -e
mkdir -p data/bul
curl -sL "https://kaikki.org/dictionary/Bulgarian/pos-verb/kaikki.org-dictionary-Bulgarian-by-pos-verb.jsonl" \
  -o data/bul/kaikki-verbs.jsonl
python3 scripts/bul/kaikki_to_tsv.py data/bul/kaikki-verbs.jsonl > data/bul/kaikki.tsv
wc -l data/bul/kaikki.tsv
