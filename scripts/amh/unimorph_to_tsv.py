#!/usr/bin/env python3
"""UniMorph Amharic verbs → shared TSV, features canonicalised to V;+sorted
tokens. Ge'ez (Ethiopic) script; forms kept as is. Gender is dropped on the
plural and 1st person where UniMorph omits it (data-driven: we keep whatever
tokens the row carries, just sorted)."""
import sys, unicodedata
def canon(feat):
    head="V"
    toks=feat.split(";")
    # keep V.MSDR / V.CVB / V.PTCP head markers in the sorted body
    rest=[t for t in toks[1:] if t]
    return "V;"+";".join(sorted(rest)) if toks[0]=="V" else None
def main(path):
    for line in open(path):
        a=line.rstrip("\n").split("\t")
        if len(a)<3 or not a[2].startswith("V"): continue
        lem=unicodedata.normalize("NFC",a[0]).strip()
        form=unicodedata.normalize("NFC",a[1]).strip()
        if not lem or not form or " " in lem or " " in form: continue
        c=canon(a[2])
        if c: print(f"{lem}\t{form}\t{c}")
if __name__=="__main__": main(sys.argv[1])
