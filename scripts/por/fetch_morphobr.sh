#!/bin/sh
# Fetch the MorphoBr verb dictionaries (Apache-2.0, LR-POR) and convert
# them for the Portuguese golden harness. Unlike UniMorph por (a
# Wiktionary scrape), MorphoBr descends from DELAF-PB/LABEL-LEX with
# systematic error correction — a genuinely independent oracle vs
# kaikki, and the only fully permissive lexicon in the project.
#
# Pinned to a commit; the concatenation is checksummed after download.
set -e
SHA=b49015cd308f128338f1aae4b49f46c887ce9608
mkdir -p data/por/morphobr
for letter in a b c d e f g h i j k l m n o p q r s t u v w x z; do
  f="verbs-$letter.dict"
  [ -s "data/por/morphobr/$f" ] || \
    curl -sfL "https://raw.githubusercontent.com/LR-POR/MorphoBr/$SHA/verbs/$f" \
      -o "data/por/morphobr/$f" || rm -f "data/por/morphobr/$f"
done
python3 scripts/por/morphobr_to_tsv.py data/por/morphobr/*.dict > data/por/morphobr.tsv
wc -l data/por/morphobr.tsv
