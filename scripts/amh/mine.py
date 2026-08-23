#!/usr/bin/env python3
"""Mine Amharic principal parts into data/amh/parts.tsv — the 3sg-masc cell of
each TAM the paradigm is regularly derived from: perfective, imperfective,
perfect, present, imperfective-nonfinite (converb base) and the 2sg-masc
imperative. Single oracle (UniMorph), so the value is taken directly."""
import sys
def load(p):
    d={}
    for line in open(p):
        a=line.rstrip("\n").split("\t")
        if len(a)==3: d[(a[0],a[2])]=a[1]
    return d
CELLS=[("pfv","V;3;MASC;PFV;SG"),("ipfv","V;3;IPFV;MASC;SG"),
       ("prf","V;3;MASC;PRF;SG"),("prs","V;3;MASC;PRS;SG"),
       ("ipfvn","V;3;IPFV;MASC;NFIN;SG"),("imp","V;2;IMP;MASC;SG")]
def main(path):
    u=load(path)
    lemmas=sorted(set(l for l,_ in u))
    print("# lemma\t"+"\t".join(k for k,_ in CELLS))
    for l in lemmas:
        vals=[u.get((l,c),"-") for _,c in CELLS]
        if vals[0]=="-" and vals[1]=="-": continue
        print(l+"\t"+"\t".join(vals))
if __name__=="__main__": main(sys.argv[1])
