#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Kazakh verb extraction (CC BY-SA).
set -e
mkdir -p data/kaz
curl -sL "https://kaikki.org/dictionary/Kazakh/pos-verb/kaikki.org-dictionary-Kazakh-by-pos-verb.jsonl" \
  -o data/kaz/kaikki-verbs.jsonl
python3 scripts/kaz/kaikki_to_tsv.py data/kaz/kaikki-verbs.jsonl > data/kaz/kaikki.tsv
wc -l data/kaz/kaikki.tsv
