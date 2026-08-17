#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Irish verb extraction (CC BY-SA).
set -e
mkdir -p data/gle
curl -sL "https://kaikki.org/dictionary/Irish/pos-verb/kaikki.org-dictionary-Irish-by-pos-verb.jsonl" \
  -o data/gle/kaikki-verbs.jsonl
python3 scripts/gle/kaikki_to_tsv.py data/gle/kaikki-verbs.jsonl > data/gle/kaikki.tsv
wc -l data/gle/kaikki.tsv
