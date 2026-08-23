#!/bin/sh
# Fetch the UniMorph Afrikaans inflection table (CC BY-SA), keep verbs.
set -e
mkdir -p data/afr
curl -sL "https://raw.githubusercontent.com/unimorph/afr/master/afr" -o data/afr/unimorph-afr.tsv
awk -F'\t' '$3 ~ /^V/' data/afr/unimorph-afr.tsv > data/afr/unimorph.tsv
wc -l data/afr/unimorph.tsv
