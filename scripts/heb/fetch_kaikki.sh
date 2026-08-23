#!/bin/sh
set -e
mkdir -p data/heb
curl -sL "https://kaikki.org/dictionary/Hebrew/pos-verb/kaikki.org-dictionary-Hebrew-by-pos-verb.jsonl" -o data/heb/kaikki-verbs.jsonl
python3 scripts/heb/kaikki_to_tsv.py data/heb/kaikki-verbs.jsonl > data/heb/kaikki.tsv
wc -l data/heb/kaikki.tsv
