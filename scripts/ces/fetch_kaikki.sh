#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Czech verb extraction (CC BY-SA).
set -e
mkdir -p data/ces
curl -sL "https://kaikki.org/dictionary/Czech/pos-verb/kaikki.org-dictionary-Czech-by-pos-verb.jsonl" \
  -o data/ces/kaikki-verbs.jsonl
python3 scripts/ces/kaikki_to_tsv.py data/ces/kaikki-verbs.jsonl > data/ces/kaikki.tsv
wc -l data/ces/kaikki.tsv
