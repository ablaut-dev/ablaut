#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Uzbek verb extraction (CC BY-SA).
set -e
mkdir -p data/uzb
curl -sL "https://kaikki.org/dictionary/Uzbek/pos-verb/kaikki.org-dictionary-Uzbek-by-pos-verb.jsonl" \
  -o data/uzb/kaikki-verbs.jsonl
python3 scripts/uzb/kaikki_to_tsv.py data/uzb/kaikki-verbs.jsonl > data/uzb/kaikki.tsv
wc -l data/uzb/kaikki.tsv
