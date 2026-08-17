#!/usr/bin/env python3
"""Generate the Vabamorf oracle TSV: synthesize every slot for the
kaikki lemma list. Vabamorf (Filosoft, LGPL) is a generator, not a
lexicon — an independent lineage driven through estnltk.

Usage: .venv-est/bin/python scripts/est/vabamorf_gen.py \
           data/est/kaikki-verbs.jsonl > data/est/vabamorf.tsv
"""
import json
import sys

from estnltk.vabamorf.morf import synthesize

# vabamorf form code → shared feature string
CODES = [
    ("ma", "V;NFIN;MA"), ("da", "V;NFIN;DA"),
    ("n", "V;IND;PRS;SG;1"), ("d", "V;IND;PRS;SG;2"),
    ("b", "V;IND;PRS;SG;3"), ("me", "V;IND;PRS;PL;1"),
    ("te", "V;IND;PRS;PL;2"), ("vad", "V;IND;PRS;PL;3"),
    ("takse", "V;IND;PRS;IMPRS"),
    ("sin", "V;IND;PST;SG;1"), ("sid", "V;IND;PST;SG;2"),
    ("s", "V;IND;PST;SG;3"), ("sime", "V;IND;PST;PL;1"),
    ("site", "V;IND;PST;PL;2"), ("ti", "V;IND;PST;IMPRS"),
    ("ksin", "V;COND;PRS;SG;1"), ("ksid", "V;COND;PRS;SG;2"),
    ("ks", "V;COND;PRS;SG;3"), ("ksime", "V;COND;PRS;PL;1"),
    ("ksite", "V;COND;PRS;PL;2"), ("taks", "V;COND;PRS;IMPRS"),
    ("nuksin", "V;COND;PRF;SG;1"), ("nuksid", "V;COND;PRF;SG;2"),
    ("nuks", "V;COND;PRF;SG;3"), ("nuksime", "V;COND;PRF;PL;1"),
    ("nuksite", "V;COND;PRF;PL;2"),
    ("o", "V;IMP;SG;2"), ("gu", "V;IMP;SG;3"),
    ("gem", "V;IMP;PL;1"), ("ge", "V;IMP;PL;2"),
    ("tagu", "V;IMP;IMPRS"),
    ("vat", "V;QUOT"),
    ("v", "V.PTCP;PRS;ACT"), ("tav", "V.PTCP;PRS;PASS"),
    ("nud", "V.PTCP;PST;ACT"), ("tud", "V.PTCP;PST;PASS"),
    ("mas", "V;NFIN;MA;INE"), ("mast", "V;NFIN;MA;ELA"),
    ("maks", "V;NFIN;MA;TRANSL"), ("mata", "V;NFIN;MA;ABE"),
    ("tama", "V;NFIN;MA;PASS"),
]


def main(path):
    lemmas = set()
    for line in open(path):
        e = json.loads(line)
        w = e.get("word", "")
        if w and " " not in w and w.endswith("ma"):
            lemmas.add(w)
    for lemma in sorted(lemmas):
        for code, feat in CODES:
            for form in synthesize(lemma, code):
                if form and " " not in form:
                    print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
