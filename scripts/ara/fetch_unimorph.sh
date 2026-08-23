#!/bin/sh
# Fetch the UniMorph Arabic verb paradigms (CC BY-SA / CC0 per UniMorph).
set -e
mkdir -p data/ara
curl -sL "https://raw.githubusercontent.com/unimorph/ara/master/ara" -o data/ara/unimorph-raw.tsv
python3 scripts/ara/unimorph_to_tsv.py data/ara/unimorph-raw.tsv > data/ara/unimorph.tsv
wc -l data/ara/unimorph.tsv
