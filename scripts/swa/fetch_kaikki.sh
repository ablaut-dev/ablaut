#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Swahili verb extraction (CC BY-SA)
# and convert it for the Swahili golden harness. ~54 MB. This is the
# Wiktionary leg of the oracle pair; it carries the full noun-class
# concord matrix and the negative paradigm swc omits.
set -e
mkdir -p data/swa
curl -sL "https://kaikki.org/dictionary/Swahili/pos-verb/kaikki.org-dictionary-Swahili-by-pos-verb.jsonl" \
  -o data/swa/kaikki-verbs.jsonl
python3 scripts/swa/kaikki_to_tsv.py data/swa/kaikki-verbs.jsonl > data/swa/kaikki.tsv
wc -l data/swa/kaikki.tsv
