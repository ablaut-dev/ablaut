#!/usr/bin/env python3
"""Convert UniMorph mkd to the shared `lemma <TAB> form <TAB> features`
TSV. Macedonian has no infinitive; the lemma is the 3sg present. Stress
marks (combining acute U+0301) are stripped — Macedonian orthography does
not write them and apertium does not emit them, so this aligns the two
oracles. Single-word forms only; the `-` null cells are dropped.

Usage: python3 scripts/mkd/unimorph_to_tsv.py data/mkd/unimorph-mkd.tsv
"""
import sys
import unicodedata


def strip(s):
    return "".join(c for c in unicodedata.normalize("NFD", s) if c != "́")


def main(path):
    rows = set()
    for line in open(path, encoding="utf-8"):
        f = line.rstrip("\n").split("\t")
        if len(f) != 3:
            continue
        lemma, form, feats = (x.strip() for x in f)
        if not feats.startswith("V"):
            continue
        form = strip(form)
        if not form or form == "-" or " " in form:
            continue
        rows.add((strip(lemma), form, feats))
    out = sys.stdout
    for lemma, form, feats in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feats}\n")


if __name__ == "__main__":
    main(sys.argv[1])
