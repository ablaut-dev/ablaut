#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Albanian verb extraction (CC BY-SA).
set -e
mkdir -p data/sqi
curl -sL "https://kaikki.org/dictionary/Albanian/pos-verb/kaikki.org-dictionary-Albanian-by-pos-verb.jsonl" \
  -o data/sqi/kaikki-verbs.jsonl
python3 scripts/sqi/kaikki_to_tsv.py data/sqi/kaikki-verbs.jsonl > data/sqi/kaikki.tsv
wc -l data/sqi/kaikki.tsv
