#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Galician verb extraction (CC BY-SA).
set -e
mkdir -p data/glg
curl -sL "https://kaikki.org/dictionary/Galician/pos-verb/kaikki.org-dictionary-Galician-by-pos-verb.jsonl" \
  -o data/glg/kaikki-verbs.jsonl
python3 scripts/glg/kaikki_to_tsv.py data/glg/kaikki-verbs.jsonl > data/glg/kaikki.tsv
wc -l data/glg/kaikki.tsv
