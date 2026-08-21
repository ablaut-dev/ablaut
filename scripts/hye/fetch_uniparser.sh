#!/bin/sh
# Generate the uniparser Eastern Armenian oracle (Timofey Arkhangelskiy's
# formalized grammar of literary Eastern Armenian, MIT; read at test time
# only, never redistributed). Independent of Wiktionary, so its agreement
# with UniMorph hye is evidence rather than an echo.
#
# Version-pinned: a silent upstream change would shift the gold standard.
set -e
mkdir -p data/hye
pip3 install --quiet uniparser-eastern-armenian==2.1.2 \
  || pip3 install --quiet --break-system-packages uniparser-eastern-armenian==2.1.2
python3 scripts/hye/uniparser_gen.py > data/hye/uniparser.tsv
wc -l data/hye/uniparser.tsv
