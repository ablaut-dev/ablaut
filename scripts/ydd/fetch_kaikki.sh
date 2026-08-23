#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Yiddish verb extraction (CC BY-SA).
set -e
mkdir -p data/ydd
curl -sL "https://kaikki.org/dictionary/Yiddish/pos-verb/kaikki.org-dictionary-Yiddish-by-pos-verb.jsonl" \
  -o data/ydd/kaikki-verbs.jsonl
python3 scripts/ydd/kaikki_to_tsv.py data/ydd/kaikki-verbs.jsonl > data/ydd/kaikki.tsv
wc -l data/ydd/kaikki.tsv
