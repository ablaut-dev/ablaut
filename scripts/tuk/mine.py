#!/usr/bin/env python3
"""Turkmen productive Oghuz vowel-harmony rules + mined exceptions.
Usage: mine.py data/tuk/kaikki.tsv > data/tuk/parts.tsv"""
import sys
from collections import defaultdict
BACK="ayou"; FRONT="äeiöü"; VOW=BACK+FRONT
CELLS=["V;PRS;1;SG","V;PRS;2;SG","V;PRS;3;SG","V;PRS;1;PL","V;PRS;2;PL","V;PRS;3;PL",
    "V;PST;1;SG","V;PST;2;SG","V;PST;3;SG","V;PST;1;PL","V;PST;2;PL","V;PST;3;PL",
    "V;AOR;1;SG","V;AOR;2;SG","V;AOR;3;SG","V;AOR;1;PL","V;AOR;2;PL","V;AOR;3;PL",
    "V;IMP;2;SG","V;IMP;2;PL","V;NFIN"]
def produce(cit,feat):
    if not (cit.endswith("mak") or cit.endswith("mek")): return None
    if feat=="V;NFIN": return cit
    stem=cit[:-3]
    v=next((c for c in reversed(stem) if c in VOW),"ä")
    back=v in BACK
    A="a" if back else "ä"; I="y" if back else "i"
    cop={"1;SG":I+"n","2;SG":"s"+I+"n","3;SG":"","1;PL":I+"s","2;PL":"s"+I+"ň"+I+"z","3;PL":"lar" if back else "ler"}
    past={"1;SG":"m","2;SG":"ň","3;SG":"","1;PL":"k","2;PL":"ň"+I+"z","3;PL":"lar" if back else "ler"}
    if feat=="V;IMP;2;SG": return stem
    if feat=="V;IMP;2;PL": return stem+I+"ň"
    p=feat.split(";"); pn=p[2]+";"+p[3]; tense=p[1]
    if tense=="PRS": return stem+"ý"+A+"r"+cop[pn]
    if tense=="PST": return stem+"d"+I+past[pn]
    if tense=="AOR": return stem+A+"r"+cop[pn]
    return None
def main(path):
    gold=defaultdict(dict)
    for l in open(path):
        a=l.rstrip("\n").split("\t")
        if len(a)>=3: gold[a[0]].setdefault(a[2],set()).add(a[1])
    total=hit=0; parts={}
    for cit,cells in gold.items():
        row={}
        for feat,forms in cells.items():
            if feat not in CELLS: continue
            pred=produce(cit,feat)
            if pred is None: continue
            total+=1
            if pred in forms: hit+=1
            else: row[feat]=sorted(forms)[0]
        if row: parts[cit]=row
    sys.stderr.write(f"rule acc: {hit}/{total}={100*hit/max(total,1):.1f}% mined:{len(parts)}\n")
    print("# lemma\t"+"\t".join(CELLS))
    for cit in sorted(parts):
        print(cit+"\t"+"\t".join(parts[cit].get(c,"-") for c in CELLS))
if __name__=="__main__": main(sys.argv[1])
