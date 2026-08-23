#!/bin/sh
set -e
mkdir -p data/heb
curl -sL "https://raw.githubusercontent.com/unimorph/heb/master/heb" -o data/heb/unimorph-raw.tsv
python3 scripts/heb/unimorph_to_tsv.py data/heb/unimorph-raw.tsv > data/heb/unimorph.tsv
wc -l data/heb/unimorph.tsv
