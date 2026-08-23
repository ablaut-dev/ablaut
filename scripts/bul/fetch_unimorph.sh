#!/bin/sh
# Fetch the UniMorph Bulgarian inflection table (CC BY-SA), keep verbs.
set -e
mkdir -p data/bul
curl -sL "https://raw.githubusercontent.com/unimorph/bul/master/bul" -o data/bul/unimorph-bul.tsv
awk -F'\t' '$3 ~ /^V/' data/bul/unimorph-bul.tsv > data/bul/unimorph.tsv
wc -l data/bul/unimorph.tsv
