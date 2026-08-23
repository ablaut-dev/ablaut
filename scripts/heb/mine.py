#!/usr/bin/env python3
"""Mine Hebrew principal parts into data/heb/parts.tsv: past-3sgm, future-3sgm,
present-masc-sg and the infinitive — enough to regularly derive the paradigm.
Prefer the value both oracles agree on, else UniMorph, else kaikki."""
import sys
def load(p):
    d={}
    for line in open(p):
        a=line.rstrip("\n").split("\t")
        if len(a)==3: d[(a[0],a[2])]=a[1]
    return d
def main(u_path,k_path):
    u=load(u_path); k=load(k_path)
    PAST="V;3;MASC;PST;SG"; FUT="V;3;FUT;MASC;SG"; PRS="V;MASC;PRS;SG"; NFIN="V;NFIN"
    def pick(l,c):
        a=u.get((l,c)); b=k.get((l,c))
        if a and b and a==b: return a
        return a or b or "-"
    lemmas=sorted(set(l for l,_ in u)|set(l for l,_ in k))
    print("# lemma\tpast\tfuture\tpresent\tinfinitive")
    for l in lemmas:
        past=pick(l,PAST); fut=pick(l,FUT)
        if past=="-" and fut=="-": continue
        print(f"{l}\t{past}\t{fut}\t{pick(l,PRS)}\t{pick(l,NFIN)}")
if __name__=="__main__": main(sys.argv[1],sys.argv[2])
