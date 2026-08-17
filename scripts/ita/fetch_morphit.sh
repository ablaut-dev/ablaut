#!/bin/sh
# Fetch Morph-It! (Zanchetta & Baroni, UniBo; CC BY-SA 2.0 / LGPL dual)
# and convert it for the Italian golden harness. Corpus-derived with
# manual curation — not a Wiktionary derivative, so a genuinely
# independent oracle vs kaikki.
set -e
mkdir -p data/ita
curl -sL "https://docs.sslmit.unibo.it/lib/exe/fetch.php?media=resources:morph-it.tgz" \
  -o data/ita/morph-it.tgz
echo "b137343dc095e038ebfaef7c743185e85ec35f5322a1b8629b600e6e898c7024  data/ita/morph-it.tgz" \
  | shasum -a 256 -c -
tar xzf data/ita/morph-it.tgz -C data/ita
iconv -f ISO-8859-1 -t UTF-8 data/ita/current_version/morph-it_048.txt \
  > data/ita/morphit-utf8.txt
python3 scripts/ita/morphit_to_tsv.py data/ita/morphit-utf8.txt > data/ita/morphit.tsv
wc -l data/ita/morphit.tsv
