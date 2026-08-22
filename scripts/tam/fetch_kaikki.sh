#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Tamil verb extraction (CC BY-SA) and
# convert it for the Tamil golden harness. This is the Wiktionary leg of
# the oracle pair; the second, independent leg is the ThamizhiMorph FST
# (scripts/tam/fetch_thamizhi.sh). See docs/tam/oracles.md.
set -e
mkdir -p data/tam
curl -sL "https://kaikki.org/dictionary/Tamil/pos-verb/kaikki.org-dictionary-Tamil-by-pos-verb.jsonl" \
  -o data/tam/kaikki-verbs.jsonl
python3 scripts/tam/kaikki_to_tsv.py data/tam/kaikki-verbs.jsonl > data/tam/kaikki.tsv
wc -l data/tam/kaikki.tsv
