#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Faroese verb extraction (CC BY-SA).
set -e
mkdir -p data/fao
curl -sL "https://kaikki.org/dictionary/Faroese/pos-verb/kaikki.org-dictionary-Faroese-by-pos-verb.jsonl" \
  -o data/fao/kaikki-verbs.jsonl
python3 scripts/fao/kaikki_to_tsv.py data/fao/kaikki-verbs.jsonl > data/fao/kaikki.tsv
wc -l data/fao/kaikki.tsv
