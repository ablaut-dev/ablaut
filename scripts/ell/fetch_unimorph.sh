#!/bin/sh
# Fetch the UniMorph Greek inflection table (CC BY-SA); keep verbs and
# split comma-listed variants into separate single-word rows.
set -e
mkdir -p data/ell
curl -sL "https://raw.githubusercontent.com/unimorph/ell/master/ell" -o data/ell/unimorph-ell.tsv
awk -F'\t' '$3 ~ /^V/{n=split($2,a,", "); for(i=1;i<=n;i++){f=a[i]; gsub(/^ +| +$/,"",f); if(f!="" && f!~/ /) print $1"\t"f"\t"$3}}' data/ell/unimorph-ell.tsv > data/ell/unimorph.tsv
wc -l data/ell/unimorph.tsv
