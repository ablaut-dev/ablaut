#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Esperanto verb extraction (CC BY-SA)
# and convert it for the Esperanto golden harness. This is the single
# oracle (Beta tier): Esperanto is perfectly regular, so one clean source
# fully pins the paradigm.
set -e
mkdir -p data/epo
curl -sL "https://kaikki.org/dictionary/Esperanto/pos-verb/kaikki.org-dictionary-Esperanto-by-pos-verb.jsonl" \
  -o data/epo/kaikki-verbs.jsonl
python3 scripts/epo/kaikki_to_tsv.py data/epo/kaikki-verbs.jsonl > data/epo/kaikki.tsv
wc -l data/epo/kaikki.tsv
