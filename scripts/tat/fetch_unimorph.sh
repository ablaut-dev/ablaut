#!/bin/sh
# Fetch the UniMorph Tatar inflection table (CC BY-SA), keep verbs.
# NOTE: UniMorph tat is Latin-script (Common Turkic orthography) while the
# kaikki.org extraction is Cyrillic Kazan Tatar, so the two do NOT overlap
# and cannot form an agreement gold. The engine is therefore scored on the
# single Cyrillic oracle (kaikki) — Beta tier. This fetch is kept only for
# provenance / future romanised work.
set -e
mkdir -p data/tat
curl -sL "https://raw.githubusercontent.com/unimorph/tat/master/tat" -o data/tat/unimorph-tat.tsv
awk -F'\t' '$3 ~ /^V/' data/tat/unimorph-tat.tsv > data/tat/unimorph.tsv
wc -l data/tat/unimorph.tsv
