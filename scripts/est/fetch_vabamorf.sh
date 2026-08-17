#!/bin/sh
# Generate the Vabamorf oracle (Filosoft lineage, LGPL) by driving
# estnltk's synthesizer over the kaikki lemma list. Generator-as-
# oracle: an independent lineage without a downloadable lexicon.
set -e
mkdir -p data/est
[ -f data/est/kaikki-verbs.jsonl ] || ./scripts/est/fetch_kaikki.sh
pip3 install --quiet estnltk 2>/dev/null || pip3 install --quiet --break-system-packages estnltk
python3 scripts/est/vabamorf_gen.py data/est/kaikki-verbs.jsonl > data/est/vabamorf.tsv
wc -l data/est/vabamorf.tsv
