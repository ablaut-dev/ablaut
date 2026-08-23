#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Luxembourgish verb extraction (CC BY-SA).
set -e
mkdir -p data/ltz
curl -sL "https://kaikki.org/dictionary/Luxembourgish/pos-verb/kaikki.org-dictionary-Luxembourgish-by-pos-verb.jsonl" \
  -o data/ltz/kaikki-verbs.jsonl
python3 scripts/ltz/kaikki_to_tsv.py data/ltz/kaikki-verbs.jsonl > data/ltz/kaikki.tsv
wc -l data/ltz/kaikki.tsv
