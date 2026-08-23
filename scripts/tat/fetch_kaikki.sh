#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Tatar verb extraction (CC BY-SA).
set -e
mkdir -p data/tat
curl -sL "https://kaikki.org/dictionary/Tatar/pos-verb/kaikki.org-dictionary-Tatar-by-pos-verb.jsonl" \
  -o data/tat/kaikki-verbs.jsonl
python3 scripts/tat/kaikki_to_tsv.py data/tat/kaikki-verbs.jsonl > data/tat/kaikki.tsv
wc -l data/tat/kaikki.tsv
