#!/bin/sh
# Fetch the UniMorph Welsh inflection table (CC BY-SA), keep the literary
# single-word verb paradigm (V;LIT;...) plus the verbal noun and participle.
# The colloquial forms (V;COL;...) carry a subject pronoun as a second word
# and are periphrastic, so they are dropped.
set -e
mkdir -p data/cym
curl -sL "https://raw.githubusercontent.com/unimorph/cym/master/cym" -o data/cym/unimorph-cym.tsv
awk -F'\t' '$3 ~ /^V/ && $3 !~ /;COL;/' data/cym/unimorph-cym.tsv > data/cym/unimorph.tsv
wc -l data/cym/unimorph.tsv
