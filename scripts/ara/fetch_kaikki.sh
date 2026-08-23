#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Arabic verb extraction (CC BY-SA).
set -e
mkdir -p data/ara
curl -sL "https://kaikki.org/dictionary/Arabic/pos-verb/kaikki.org-dictionary-Arabic-by-pos-verb.jsonl" \
  -o data/ara/kaikki-verbs.jsonl
python3 scripts/ara/kaikki_to_tsv.py data/ara/kaikki-verbs.jsonl > data/ara/kaikki.tsv
wc -l data/ara/kaikki.tsv
