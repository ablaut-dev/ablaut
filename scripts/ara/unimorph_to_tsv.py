#!/usr/bin/env python3
"""Normalise UniMorph Arabic verb rows to the shared lemma⇥form⇥feat TSV.

Arabic lemmas are the unvoweled consonantal skeleton; forms are fully
voweled. Features are canonicalised to `V;` + the remaining tags sorted
alphabetically, so the UniMorph and kaikki adapters emit byte-identical
feature keys for the same cell (the golden harness intersects on them).
UniMorph's jussive tag is LGSPEC1 → normalised to JUS.
"""
import sys, unicodedata

def norm(s):
    return unicodedata.normalize("NFC", s).strip()

def canon(feat):
    toks = feat.split(";")
    if toks[0] != "V":
        return None
    rest = [t for t in toks[1:] if t]
    rest = ["JUS" if t == "LGSPEC1" else t for t in rest]
    # participle rows carry V.PTCP; keep it in the sorted body
    return "V;" + ";".join(sorted(rest))

def main(path):
    for line in open(path):
        a = line.rstrip("\n").split("\t")
        if len(a) < 3:
            continue
        lemma, form, feat = norm(a[0]), norm(a[1]), a[2].strip()
        if not feat.startswith("V") or not lemma or not form:
            continue
        if " " in lemma or " " in form:
            continue
        c = canon(feat)
        if c:
            print(f"{lemma}\t{form}\t{c}")

if __name__ == "__main__":
    main(sys.argv[1])
