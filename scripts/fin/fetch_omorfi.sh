#!/bin/sh
# Generate the Omorfi oracle (Apache/GPL, the open Finnish morphology)
# by driving omorfi.generate.hfst over the kaikki lemma list.
set -e
mkdir -p data/fin
[ -f data/fin/kaikki-verbs.jsonl ] || ./scripts/fin/fetch_kaikki.sh
[ -f data/fin/omorfi.generate.hfst ] || {
  curl -sL "https://github.com/flammie/omorfi/releases/download/v0.9.12/omorfi-hfst-models-0.9.12.tar.xz" \
    -o data/fin/omorfi-models.tar.xz
  cd data/fin && tar xJf omorfi-models.tar.xz omorfi.generate.hfst && cd ../..
}
pip3 install --quiet hfst 2>/dev/null || pip3 install --quiet --break-system-packages hfst
python3 scripts/fin/omorfi_gen.py data/fin/kaikki-verbs.jsonl > data/fin/omorfi.tsv
wc -l data/fin/omorfi.tsv
