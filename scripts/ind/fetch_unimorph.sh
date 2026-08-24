#!/bin/sh
# Fetch the UniMorph Indonesian inflection table (CC BY-SA) and build the
# harness gold. UniMorph `ind` carries ~15k verb (V;...) rows and is the sole
# reliable oracle — kaikki's Indonesian verb dump is thin (~77 rich entries),
# too sparse to form a two-oracle agreement loop, so `ind` is scored directly
# (Beta tier).
set -e
mkdir -p data/ind
curl -sL "https://raw.githubusercontent.com/unimorph/ind/master/ind" -o data/ind/unimorph_raw.tsv
python3 scripts/ind/unimorph_to_tsv.py data/ind/unimorph_raw.tsv data/ind/unimorph.tsv
# Beta: no aligning second oracle.
touch data/ind/_no_second_oracle.tsv
wc -l data/ind/unimorph.tsv
