#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Zulu verb extraction (CC BY-SA) and
# convert it for the Zulu golden harness. This is the Wiktionary leg of
# the oracle pair; it independently confirms the infinitive, imperative,
# subjunctive and remote-past backbone of the productive template.
set -e
mkdir -p data/zul
curl -sL "https://kaikki.org/dictionary/Zulu/pos-verb/kaikki.org-dictionary-Zulu-by-pos-verb.jsonl" \
  -o data/zul/kaikki-verbs.jsonl
python3 scripts/zul/kaikki_to_tsv.py data/zul/kaikki-verbs.jsonl > data/zul/kaikki.tsv
wc -l data/zul/kaikki.tsv
