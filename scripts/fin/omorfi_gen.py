#!/usr/bin/env python3
"""Generate the Omorfi oracle TSV: drive omorfi.generate.hfst over the
kaikki lemma list. Omorfi (Apache/GPL) is the open Finnish morphology
— the second generator-as-oracle after Estonian's Vabamorf.

Usage: .venv-est/bin/python scripts/fin/omorfi_gen.py \
           data/fin/kaikki-verbs.jsonl > data/fin/omorfi.tsv
"""
import json
import sys

import hfst

PERS = [("SG1", "SG;1"), ("SG2", "SG;2"), ("SG3", "SG;3"),
        ("PL1", "PL;1"), ("PL2", "PL;2"), ("PL3", "PL;3")]

CODES = [("[VOICE=ACT][INF=A][NUM=SG][CASE=LAT]", "V;NFIN")]
for mood, tense, feat in [("INDV", "[TENSE=PRESENT]", "IND;PRS"),
                          ("INDV", "[TENSE=PAST]", "IND;PST"),
                          ("COND", "", "COND"), ("POTN", "", "POT")]:
    for pers, pf in PERS:
        CODES.append((f"[VOICE=ACT][MOOD={mood}]{tense}[PERS={pers}]",
                      f"V;{feat};{pf}"))
    CODES.append((f"[VOICE=PSS][MOOD={mood}]{tense}[PERS=PE4]",
                  f"V;{feat};PASS"))
for pers, pf in [("SG2", "SG;2"), ("SG3", "SG;3"), ("PL1", "PL;1"),
                 ("PL2", "PL;2"), ("PL3", "PL;3")]:
    CODES.append((f"[VOICE=ACT][MOOD=IMPV][PERS={pers}]", f"V;IMP;{pf}"))
CODES.append(("[VOICE=PSS][MOOD=IMPV][PERS=PE4]", "V;IMP;PASS"))
for voice, v in [("ACT", "ACT"), ("PSS", "PASS")]:
    for pcp, t in [("VA", "PRS"), ("NUT", "PST")]:
        CODES.append((f"[VOICE={voice}][PCP={pcp}][CMP=POS][NUM=SG][CASE=NOM]",
                      f"V.PTCP;{t};{v}"))


def main(path):
    lemmas = set()
    for line in open(path):
        e = json.loads(line)
        w = e.get("word", "")
        if w and " " not in w and w[0].islower():
            lemmas.add(w)
    t = hfst.HfstInputStream("data/fin/omorfi.generate.hfst").read()
    for lemma in sorted(lemmas):
        head = f"[WORD_ID={lemma}][UPOS=VERB]"
        for code, feat in CODES:
            for form, _w in t.lookup(head + code):
                if form and " " not in form and "@" not in form:
                    print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
