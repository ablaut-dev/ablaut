#!/bin/sh
# Fetch the kaikki.org (Wiktextract) German verb extraction (CC BY-SA) and
# convert it for the golden harness:
#   cargo run --release --bin golden data/kaikki/deu.tsv
set -e
mkdir -p data/kaikki
curl -sL "https://kaikki.org/dictionary/German/pos-verb/kaikki.org-dictionary-German-by-pos-verb.jsonl" \
  -o data/kaikki/verbs-deu.jsonl
python3 scripts/kaikki_deu_to_tsv.py data/kaikki/verbs-deu.jsonl > data/kaikki/deu.tsv
wc -l data/kaikki/deu.tsv
