#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Romanian verb extraction (CC BY-SA)
# and convert it for the Romanian golden harness.
set -e
mkdir -p data/ron
curl -sL "https://kaikki.org/dictionary/Romanian/pos-verb/kaikki.org-dictionary-Romanian-by-pos-verb.jsonl" \
  -o data/ron/kaikki-verbs.jsonl
python3 scripts/ron/kaikki_to_tsv.py data/ron/kaikki-verbs.jsonl > data/ron/kaikki.tsv
wc -l data/ron/kaikki.tsv
