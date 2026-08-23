#!/bin/sh
# Fetch the UniMorph Polish inflection table (CC BY-SA), keep verbs.
set -e
mkdir -p data/pol
curl -sL "https://raw.githubusercontent.com/unimorph/pol/master/pol" -o data/pol/unimorph-pol.tsv
awk -F'\t' '$3 ~ /^V/' data/pol/unimorph-pol.tsv > data/pol/unimorph.tsv
wc -l data/pol/unimorph.tsv
