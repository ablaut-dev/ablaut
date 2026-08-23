#!/usr/bin/env python3
"""Convert UniMorph nob to the shared `lemma <TAB> form <TAB> features` TSV.

UniMorph Norwegian Bokmål enumerates the finite paradigm with no
person/number agreement: present (V;IND;PRS), preterite (V;IND;PST), the
s-form (V;IND;PASS), imperative (V;IMP) and the two participles
(V.PTCP;PRS, V.PTCP;PST). The lemma is the infinitive; there is no
separate V;NFIN row. Kept as-is except: single-word forms only, and the
32 verbal-noun (V.MSDR) rows and one stray combined tag are dropped —
neither is a conjugation slot.

Usage: python3 scripts/nob/unimorph_to_tsv.py data/nob/unimorph-nob.tsv
"""
import sys

DROP = {"V.MSDR", "V;V.PTCP;PRS"}


def main(path):
    rows = set()
    for line in open(path, encoding="utf-8"):
        f = line.rstrip("\n").split("\t")
        if len(f) != 3:
            continue
        lemma, form, feats = (x.strip() for x in f)
        if not feats.startswith("V") or feats in DROP:
            continue
        # "-" is UniMorph's null marker for a defective slot (intransitives
        # with no s-passive: gro, havne); a space marks a multi-word or
        # alternative-note cell. Neither is a real target form.
        if not form or form == "-" or " " in form:
            continue
        rows.add((lemma, form, feats))
    out = sys.stdout
    for lemma, form, feats in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feats}\n")


if __name__ == "__main__":
    main(sys.argv[1])
