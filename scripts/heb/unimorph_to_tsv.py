#!/usr/bin/env python3
"""UniMorph Hebrew verbs → shared TSV, features canonicalised to V;+sorted
tokens. Forms are unvoweled ktiv male (the modern standard); kept as is."""
import sys, unicodedata
def main(path):
    for line in open(path):
        a=line.rstrip("\n").split("\t")
        if len(a)<3 or not a[2].startswith("V"): continue
        lem=unicodedata.normalize("NFC",a[0]).strip()
        form=unicodedata.normalize("NFC",a[1]).strip()
        if not lem or not form or " " in lem or " " in form: continue
        toks=[t for t in a[2].split(";")[1:] if t]
        print(f"{lem}\t{form}\tV;"+";".join(sorted(toks)))
if __name__=="__main__": main(sys.argv[1])
