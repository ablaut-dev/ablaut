#!/bin/sh
# Fetch the UniMorph Latin inflection table (CC BY-SA), keep the scored
# present-system active-indicative verb cells.
set -e
mkdir -p data/lat
curl -sL "https://raw.githubusercontent.com/unimorph/lat/master/lat" -o data/lat/unimorph-lat.tsv
python3 scripts/lat/unimorph_to_tsv.py data/lat/unimorph-lat.tsv > data/lat/unimorph.tsv
wc -l data/lat/unimorph.tsv
