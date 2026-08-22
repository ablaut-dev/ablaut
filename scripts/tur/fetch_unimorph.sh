#!/bin/sh
# Fetch UniMorph tur and convert it for the Turkish golden harness. This
# is the native-verified leg of the oracle pair: UniMorph Turkish is
# generated with TRmorph (Çağrı Çöltekin's finite-state Turkish
# morphology) and is independent of Wiktionary/kaikki.
#
# Pinned to a commit and checksummed: a silent upstream change would
# shift the gold standard. Read at test time only, never redistributed.
set -e
mkdir -p data/tur
curl -sL "https://raw.githubusercontent.com/unimorph/tur/6c179ace7d2f3d7f3484020e5304c1544d07bb6b/tur" \
  -o data/tur/unimorph-tur.tsv
echo "4ae44f4b039c344f7078d8073b968b0d67133d1e9d7e653ce8aba5ed24387957  data/tur/unimorph-tur.tsv" \
  | shasum -a 256 -c -
python3 scripts/tur/unimorph_to_tsv.py data/tur/unimorph-tur.tsv > data/tur/unimorph.tsv
wc -l data/tur/unimorph.tsv
