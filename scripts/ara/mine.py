#!/usr/bin/env python3
"""Mine per-lemma principal parts for the Arabic engine into data/ara/parts.tsv.

Arabic verbs are cited by the unvoweled skeleton, which does not fix the
vocalisation, so the engine cannot vowel a paradigm from the lemma alone.
We store four fully-voweled principal parts per lemma — the 3sg-masc cells
the whole paradigm is regularly derived from:
    PA  perfect   active  (كَتَبَ)
    IA  imperfect active  indicative (يَكْتُبُ)
    PP  perfect   passive (كُتِبَ)      — '-' if the oracle lacks it
    IP  imperfect passive indicative (يُكْتَبُ) — '-' if absent
Value preferred is the one both oracles agree on, else UniMorph, else kaikki.
"""
import sys

def load(p):
    d = {}
    for line in open(p):
        a = line.rstrip("\n").split("\t")
        if len(a) == 3:
            d[(a[0], a[2])] = a[1]
    return d

def key(*toks):
    return "V;" + ";".join(sorted(toks))

def main(uni_path, kai_path):
    u = load(uni_path); k = load(kai_path)
    PA = key("3","SG","MASC","PST","PRF","IND","ACT")
    IA = key("3","SG","MASC","IPFV","IND","ACT")
    PP = key("3","SG","MASC","PST","PRF","IND","PASS")
    IP = key("3","SG","MASC","IPFV","IND","PASS")
    def pick(l, c):
        a = u.get((l, c)); b = k.get((l, c))
        if a and b and a == b: return a
        return a or b or "-"
    lemmas = sorted(set(l for l, _ in u) | set(l for l, _ in k))
    print("# lemma\tPA\tIA\tPP\tIP")
    for l in lemmas:
        pa = pick(l, PA); ia = pick(l, IA)
        if pa == "-" or ia == "-":
            continue
        print(f"{l}\t{pa}\t{ia}\t{pick(l,PP)}\t{pick(l,IP)}")

if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
