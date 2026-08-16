#!/bin/sh
# Fetch the kaikki.org (Wiktextract) German verb extraction (CC BY-SA) and
# convert it for the golden harness:
#   cargo run --release --bin golden data/deu/kaikki.tsv
set -e
mkdir -p data/deu
curl -sL "https://kaikki.org/dictionary/German/pos-verb/kaikki.org-dictionary-German-by-pos-verb.jsonl" \
  -o data/deu/kaikki-verbs.jsonl
python3 scripts/deu/kaikki_to_tsv.py data/deu/kaikki-verbs.jsonl > data/deu/kaikki.tsv
wc -l data/deu/kaikki.tsv
