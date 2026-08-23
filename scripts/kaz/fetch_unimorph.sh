#!/bin/sh
# Fetch the UniMorph Kazakh table (CC BY-SA) and keep the verb rows
# (features beginning with V). NOTE: as of this writing the UniMorph kaz
# file contains only nominal paradigms (N;...) — there are no verb rows —
# so it cannot serve as a second oracle. The Kazakh engine is therefore a
# single-oracle (kaikki.org only) Beta, exactly like Azerbaijani. This
# script is kept so the second oracle can be wired in the day UniMorph
# ships Kazakh verbs.
set -e
mkdir -p data/kaz
curl -sL "https://raw.githubusercontent.com/unimorph/kaz/master/kaz" -o data/kaz/unimorph-kaz.tsv
awk -F'\t' '$3 ~ /^V/' data/kaz/unimorph-kaz.tsv > data/kaz/unimorph.tsv
wc -l data/kaz/unimorph.tsv
