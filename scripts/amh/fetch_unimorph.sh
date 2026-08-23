#!/bin/sh
set -e
mkdir -p data/amh
curl -sL "https://raw.githubusercontent.com/unimorph/amh/master/amh" -o data/amh/unimorph-raw.tsv
python3 scripts/amh/unimorph_to_tsv.py data/amh/unimorph-raw.tsv > data/amh/unimorph.tsv
wc -l data/amh/unimorph.tsv
