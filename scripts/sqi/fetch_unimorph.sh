#!/bin/sh
# Fetch the UniMorph Albanian inflection table (CC BY-SA), keep verbs.
set -e
mkdir -p data/sqi
curl -sL "https://raw.githubusercontent.com/unimorph/sqi/master/sqi" -o data/sqi/unimorph-sqi.tsv
awk -F'\t' '$3 ~ /^V/' data/sqi/unimorph-sqi.tsv > data/sqi/unimorph.tsv
wc -l data/sqi/unimorph.tsv
