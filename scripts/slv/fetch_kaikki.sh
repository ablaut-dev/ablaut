#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Slovene verb extraction (CC BY-SA).
# en.wiktionary names the language "Slovene".
set -e
mkdir -p data/slv
curl -sL "https://kaikki.org/dictionary/Slovene/pos-verb/kaikki.org-dictionary-Slovene-by-pos-verb.jsonl" \
  -o data/slv/kaikki-verbs.jsonl
python3 scripts/slv/kaikki_to_tsv.py data/slv/kaikki-verbs.jsonl > data/slv/kaikki.tsv
wc -l data/slv/kaikki.tsv
