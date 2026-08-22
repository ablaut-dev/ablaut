#!/bin/sh
# Generate the ThamizhiMorph Tamil oracle (K. Sarveswaran, Apache-2.0;
# read at test time only, never redistributed). It is a foma finite-state
# transducer, independent of Wiktionary, so its agreement with kaikki is
# evidence rather than an echo. This is the second, FST-derived oracle of
# the Tamil pair — see docs/tam/oracles.md.
#
# Requires foma (brew install foma / apt-get install foma). The verb FSTs
# are recompiled from the lexc + meta-morph sources shipped in the repo's
# foma/ThamizhiMorph-Verbs.zip, not from the checked-in binaries, so the
# oracle is reproducible from source.
#
# Commit-pinned: a silent upstream change would shift the gold standard.
set -e
COMMIT=adbacceda5e8aa902e4b6ed58a3edf5f78cd46fb
mkdir -p data/tam
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

git clone --quiet https://github.com/sarves/thamizhi-morph "$WORK/tm"
git -C "$WORK/tm" checkout --quiet "$COMMIT"

# Unpack the foma verb sources (lexc lexicons + .foma meta-morph rules).
unzip -q -o "$WORK/tm/foma/ThamizhiMorph-Verbs.zip" -d "$WORK/src"

# Compile each class FST from source.
mkdir -p "$WORK/fst"
for f in C3 C4 C11 C12 C62 otherthan-3-4-62-11-12; do
  case "$f" in
    otherthan-3-4-62-11-12) out=verb-c-rest ;;
    *) out=$(printf 'verb-c%s' "$(echo "$f" | tr 'A-Z' 'a-z' | sed 's/^c//')") ;;
  esac
  ( cd "$WORK/src" && foma -q -e "source ThamizhiFST-$f.foma" \
      -e "save stack $WORK/fst/$out.fst" -e "quit" >/dev/null )
done

# Generate only for the lemmas kaikki carries (the overlap is what the
# harness scores); everything else would just be dropped in the intersect.
cut -f1 data/tam/kaikki.tsv | sort -u > "$WORK/lemmas.txt"

python3 scripts/tam/thamizhi_gen.py "$WORK/fst" "$WORK/src" "$WORK/lemmas.txt" \
  > data/tam/thamizhi.tsv
wc -l data/tam/thamizhi.tsv
