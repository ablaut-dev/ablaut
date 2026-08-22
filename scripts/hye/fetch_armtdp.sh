#!/bin/sh
# Fetch the UD Armenian-ArmTDP treebank (CC BY-SA 4.0; read at test time
# only, never redistributed) and convert its verb tokens to the shared
# TSV. This is a *spot check*, not one of the two oracles: a treebank
# attests forms rather than filling paradigms. It exists to corroborate
# the core irregulars of scripts/hye/manual.tsv, which UniMorph hye
# omits — see docs/hye/oracles.md.
#
# Pinned to a commit: a silent upstream change would move the numbers.
set -e
mkdir -p data/hye/armtdp
COMMIT=90fe19c2cc3942c4ab53c08e8aa09770c1177ab3
for split in train dev test; do
  curl -sL "https://raw.githubusercontent.com/UniversalDependencies/UD_Armenian-ArmTDP/$COMMIT/hy_armtdp-ud-$split.conllu" \
    -o "data/hye/armtdp/hy_armtdp-ud-$split.conllu"
done
python3 scripts/hye/armtdp_to_tsv.py data/hye/armtdp/*.conllu > data/hye/armtdp.tsv
wc -l data/hye/armtdp.tsv
