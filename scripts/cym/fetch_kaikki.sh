#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Welsh verb extraction (CC BY-SA).
set -e
mkdir -p data/cym
curl -sL "https://kaikki.org/dictionary/Welsh/pos-verb/kaikki.org-dictionary-Welsh-by-pos-verb.jsonl" \
  -o data/cym/kaikki-verbs.jsonl
python3 scripts/cym/kaikki_to_tsv.py data/cym/kaikki-verbs.jsonl > data/cym/kaikki.tsv
wc -l data/cym/kaikki.tsv
