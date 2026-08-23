#!/usr/bin/env python3
"""kaikki.org Turkmen verbs -> shared TSV. Turkmen marks phonemic vowel
length with a combining macron that standard orthography omits; we strip
only the macron (U+0304), keeping the diaeresis of ä/ö/ü. The synthetic
core: present (-ýar), past definite (-dy), future indefinite (-ar), the two
imperatives and the infinitive (-mak/-mek)."""
import json, sys, unicodedata
P={"first-person":"1","second-person":"2","third-person":"3"}
N={"singular":"SG","plural":"PL"}
DROP={"error-unrecognized-form","table-tags","inflection-template","romanization",
      "alternative","participle","negative","continuative","perfect","evidential",
      "necessitative","optative","conditional","definite","imperfective","habitual",
      "interrogative","progressive"}
def norm(s):
    return unicodedata.normalize("NFC","".join(c for c in unicodedata.normalize("NFD",s) if c!="̄"))
def feature(tags):
    t={x.lower() for x in tags}
    if t & DROP: return None
    if "infinitive" in t: return "V;NFIN"
    p=next((v for k,v in P.items() if k in t),None); n=next((v for k,v in N.items() if k in t),None)
    if "imperative" in t and n: return f"V;IMP;2;{n}"
    if not(p and n): return None
    if "present" in t: return f"V;PRS;{p};{n}"
    if "past" in t: return f"V;PST;{p};{n}"
    if "future" in t and "indefinite" in t: return f"V;AOR;{p};{n}"
    return None
def main(path):
    for line in open(path):
        try: e=json.loads(line)
        except: continue
        if e.get("pos")!="verb": continue
        lemma=norm(e.get("word",""))
        if not lemma or " " in lemma: continue
        rows=set()
        for f in e.get("forms",[]):
            form=norm(f.get("form","").strip())
            if not form or " " in form or form=="-": continue
            ft=feature(f.get("tags",[]))
            if ft: rows.add((form,ft))
        for form,ft in sorted(rows): print(f"{lemma}\t{form}\t{ft}")
if __name__=="__main__": main(sys.argv[1])
