#!/usr/bin/env python3
"""kaikki.org Uzbek verbs -> shared TSV. Uzbek has no vowel harmony; the
kaikki tables give the third-person present/past/future core. Citation is
the -moq infinitive."""
import json, sys
P={"first-person":"1","second-person":"2","third-person":"3"}
N={"singular":"SG","plural":"PL"}
DROP={"error-unrecognized-form","table-tags","inflection-template","romanization",
      "alternative","participle","negative","continuative","perfective","obligative",
      "intentive","conditional","imperfective","habitual","formal","interrogative"}
def feature(tags):
    t={x.lower() for x in tags}
    if t & DROP: return None
    if "infinitive" in t: return "V;NFIN"
    p=next((v for k,v in P.items() if k in t),None); n=next((v for k,v in N.items() if k in t),None)
    if "imperative" in t and p and n: return f"V;IMP;{p};{n}"
    if not(p and n): return None
    if "present" in t and "indicative" in t: return f"V;PRS;{p};{n}"
    if "past" in t and "definite" in t: return f"V;PST;{p};{n}"
    if "future" in t and "definite" in t: return f"V;FUT;{p};{n}"
    if "future" in t and "indefinite" in t: return f"V;AOR;{p};{n}"
    return None
def main(path):
    for line in open(path):
        try: e=json.loads(line)
        except: continue
        if e.get("pos")!="verb": continue
        lemma=e.get("word","")
        if not lemma or " " in lemma: continue
        rows=set()
        for f in e.get("forms",[]):
            form=f.get("form","").strip().rstrip("!")
            if not form or " " in form or form=="-": continue
            ft=feature(f.get("tags",[]))
            if ft: rows.add((form,ft))
        for form,ft in sorted(rows): print(f"{lemma}\t{form}\t{ft}")
if __name__=="__main__": main(sys.argv[1])
