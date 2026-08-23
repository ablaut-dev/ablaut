#!/usr/bin/env python3
"""kaikki.org Hebrew verbs → shared TSV. Hebrew Wiktionary forms carry niqqud
which UniMorph does not, so points (U+0591–U+05C7 combining marks) are stripped;
the residual disagreement is ktiv male vs chaser, left for the harness to
exclude. Tags map to UniMorph's cell structure: present has no person; 1st
person and the past 3pl carry no gender."""
import json, sys, unicodedata
PERSON={"first-person":"1","second-person":"2","third-person":"3"}
NUM={"singular":"SG","plural":"PL"}
GEN={"masculine":"MASC","feminine":"FEM"}
def strip_niqqud(s):
    return "".join(c for c in unicodedata.normalize("NFC",s)
                   if not (0x591<=ord(c)<=0x5C7 and unicodedata.category(c)=="Mn"))
def canon(tags):
    t=set(tags)
    if t & {"participle","infinitive","noun-from-verb","negative","error-unrecognized-form"}:
        return None
    p=next((PERSON[k] for k in PERSON if k in t),None)
    num=next((NUM[k] for k in NUM if k in t),None)
    g=next((GEN[k] for k in GEN if k in t),None)
    if "imperative" in t: tense="IMP"
    elif "future" in t: tense="FUT"
    elif "past" in t: tense="PST"
    elif "present" in t: tense="PRS"
    else: return None
    if num is None: return None
    if tense=="PRS": p=None
    elif p is None: return None
    if p=="1": g=None
    if tense=="PST" and p=="3" and num=="PL": g=None
    body=[x for x in (p,num,g,tense) if x]
    return "V;"+";".join(sorted(body))
def main(path):
    for line in open(path):
        try: e=json.loads(line)
        except: continue
        lem=strip_niqqud((e.get("word") or "").strip())
        if not lem or " " in lem: continue
        for f in e.get("forms",[]):
            if "romanization" in f.get("tags",[]): continue
            form=strip_niqqud((f.get("form") or "").strip())
            if not form or " " in form: continue
            c=canon(f.get("tags",[]))
            if c: print(f"{lem}\t{form}\t{c}")
if __name__=="__main__": main(sys.argv[1])
