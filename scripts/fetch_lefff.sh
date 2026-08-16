#!/bin/sh
# Fetch the Lefff 3.4 extensional lexicon (LGPL-LR, INRIA/Alexina) and convert
# it for the French golden harness. Unlike UniMorph fra, the Lefff is NOT a
# Wiktionary scrape, so it is a genuinely independent oracle vs kaikki.
#
# Canonical source: gitlab.inria.fr/almanach/alexina/lefff (intensional, needs
# the Alexina toolchain); this fetches the widely redistributed compiled .mlex.
set -e
mkdir -p data/lefff
curl -sL "https://raw.githubusercontent.com/ClaudeCoulombe/FrenchLefffLemmatizer/master/french_lefff_lemmatizer/data/lefff-3.4.mlex" \
  -o data/lefff/lefff-3.4.mlex
# Pin the mirror's copy: a silent upstream change would shift the gold standard.
echo "f3da25e58aec161c5ae34d598038dd6304056c2649867ede7e220a74fd34fe12  data/lefff/lefff-3.4.mlex" \
  | shasum -a 256 -c -
python3 scripts/lefff_to_tsv.py data/lefff/lefff-3.4.mlex > data/lefff/fra.tsv
wc -l data/lefff/fra.tsv
