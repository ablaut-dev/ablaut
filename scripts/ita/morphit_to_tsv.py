#!/usr/bin/env python3
"""Convert Morph-It! (form<TAB>lemma<TAB>VER:mood+tense+person+number)
into the lemma<TAB>form<TAB>features TSV the Italian golden harness
reads.

Usage: python3 scripts/ita/morphit_to_tsv.py data/ita/morphit-utf8.txt \
           > data/ita/morphit.tsv

Clitic-suffixed rows (parlami: trailing +mi/+ci/+ne/…) are the
compositional layer's business and are skipped.
"""
import sys

TENSE = {
    ("ind", "pres"): "V;IND;PRS",
    ("ind", "impf"): "V;IND;PST;IPFV",
    ("ind", "past"): "V;IND;PST;PFV",
    ("ind", "fut"): "V;IND;FUT",
    ("cond", "pres"): "V;COND",
    ("sub", "pres"): "V;SBJV;PRS",
    ("sub", "impf"): "V;SBJV;PST",
    ("impr", "pres"): "V;IMP",
}
NUMBER = {"s": "SG", "p": "PL"}
GENDER = {"m": "MASC", "f": "FEM"}


def features(tag):
    if not tag.startswith("VER:"):
        return None
    parts = tag[4:].split("+")
    if parts[:2] == ["inf", "pres"] and len(parts) == 2:
        return "V;NFIN"
    if parts[:2] == ["ger", "pres"] and len(parts) == 2:
        return "V;GER"
    if parts[0] == "part" and len(parts) == 4:
        n, g = NUMBER.get(parts[2]), GENDER.get(parts[3])
        if n and g:
            when = "PST" if parts[1] == "past" else "PRS"
            return f"V.PTCP;{when};{g};{n}"
        return None
    if len(parts) == 4 and parts[2] in "123" and parts[3] in NUMBER:
        key = TENSE.get((parts[0], parts[1]))
        if key:
            return f"{key};{NUMBER[parts[3]]};{parts[2]}"
    return None


def main(path):
    seen = set()
    for line in open(path):
        cols = line.rstrip("\n").split("\t")
        if len(cols) != 3:
            continue
        form, lemma, tag = cols
        feat = features(tag)
        if feat is None:
            continue
        row = (lemma, form, feat)
        if row not in seen:
            seen.add(row)
            sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
