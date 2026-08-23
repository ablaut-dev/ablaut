#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Afrikaans verb extraction (CC BY-SA).
set -e
mkdir -p data/afr
curl -sL "https://kaikki.org/dictionary/Afrikaans/pos-verb/kaikki.org-dictionary-Afrikaans-by-pos-verb.jsonl" \
  -o data/afr/kaikki-verbs.jsonl
python3 scripts/afr/kaikki_to_tsv.py data/afr/kaikki-verbs.jsonl > data/afr/kaikki.tsv
wc -l data/afr/kaikki.tsv
