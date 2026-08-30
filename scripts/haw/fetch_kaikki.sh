#!/bin/sh
# Fetch the kaikki.org (Wiktextract) Hawaiian verb extraction (CC BY-SA) and
# convert it into the Hawaiian golden oracle. Hawaiian's only bound verbal
# morphology is derivational (the hoʻo- causative, full reduplication, the
# -ʻia passive); TAM is periphrastic and out of scope. The oracle is the
# lemma-linked "Derived terms" and passive `forms` Wiktionary records — the
# single independent source the engine is gated against (Beta tier).
set -e
mkdir -p data/haw
curl -sL "https://kaikki.org/dictionary/Hawaiian/pos-verb/kaikki.org-dictionary-Hawaiian-by-pos-verb.jsonl" \
  -o data/haw/kaikki-verbs.jsonl
python3 scripts/haw/kaikki_to_tsv.py data/haw/kaikki-verbs.jsonl > data/haw/kaikki.tsv
wc -l data/haw/kaikki.tsv
