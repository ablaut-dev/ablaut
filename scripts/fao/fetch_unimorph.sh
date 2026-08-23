#!/bin/sh
# Fetch the UniMorph Faroese inflection table (CC BY-SA), keep verbs.
set -e
mkdir -p data/fao
curl -sL "https://raw.githubusercontent.com/unimorph/fao/master/fao" -o data/fao/unimorph-fao.tsv
awk -F'\t' '$3 ~ /^V/' data/fao/unimorph-fao.tsv > data/fao/unimorph.tsv
wc -l data/fao/unimorph.tsv
