#!/usr/bin/env python3
"""Uzbek productive suffix rules (no vowel harmony) + mined exceptions.
Validates rule accuracy against kaikki, emits data/uzb/parts.tsv.
Usage: mine.py data/uzb/kaikki.tsv > data/uzb/parts.tsv"""
import sys
from collections import defaultdict
VOW="aeiouoʻ"
CELLS=["V;PRS;3;SG","V;PRS;3;PL","V;PST;3;SG","V;PST;3;PL","V;FUT;3;SG","V;FUT;3;PL",
    "V;AOR;3;SG","V;AOR;3;PL","V;IMP;3;SG","V;IMP;3;PL","V;NFIN"]
def produce(cit,feat):
    if not cit.endswith("moq"): return None
    if feat=="V;NFIN": return cit
    stem=cit[:-3]
    v = stem[-1] in VOW if stem else False
    pres = stem+("ydi" if v else "adi")
    past = stem+"di"
    fut = stem+("yajak" if v else "ajak")
    aor = stem+("ydi" if v else "adi")  # placeholder; aorist mostly mined
    imp = stem+"sin"
    base={"V;PRS;3;SG":pres,"V;PRS;3;PL":pres+"lar","V;PST;3;SG":past,"V;PST;3;PL":past+"lar",
        "V;FUT;3;SG":fut,"V;FUT;3;PL":fut+"lar","V;AOR;3;SG":stem+"ar","V;AOR;3;PL":stem+"arlar",
        "V;IMP;3;SG":imp,"V;IMP;3;PL":imp+"lar"}
    return base.get(feat)
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
