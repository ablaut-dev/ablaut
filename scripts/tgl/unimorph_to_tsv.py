#!/usr/bin/env python3
"""Convert UniMorph tgl to the shared `lemma ⇥ form ⇥ features` TSV.

UniMorph tgl (unimorph/tgl) keys every form on the bare root lemma and
tags it with aspect × trigger: 344 verb roots × 7 slots. The slots per
root are the root itself (V;NFIN) and, for each of the two triggers
Wiktionary tabulates — actor (AGFOC) and patient/object (PFOC) — the
three aspects: perfective (V;PFV;<trigger>), imperfective
(V;IPFV;<trigger>) and the contemplated aspect, which UniMorph writes as
V;<trigger>;LGSPEC1.

The bundles are already the harness schema, so the only work here is to
drop the non-verb rows and the empty cells.

Usage: python3 scripts/tgl/unimorph_to_tsv.py data/tgl/unimorph-tgl.txt
"""

import sys


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 3:
                continue
            lemma, form, features = (f.strip() for f in fields)
            if not features.startswith("V") or not form or form == "-":
                continue
            rows.add((lemma, form, features))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
