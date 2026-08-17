#!/usr/bin/env python3
"""Convert BuNaMo (the Irish National Morphology Database, Foras na
Gaeilge, ODbL) verb XMLs to the shared TSV. Independent forms only;
forms are stored unmutated, which is the shared convention (the
kaikki converter strips lenition)."""
import glob
import sys
import xml.etree.ElementTree as ET

TENSE = {"Past": "PST", "PastCont": "PSTHAB", "PresCont": "PRS",
         "Fut": "FUT", "Cond": "COND"}
PERSON = {"Base": "BASE", "Sg1": "1SG", "Sg2": "2SG", "Pl1": "1PL",
          "Pl2": "2PL", "Pl3": "3PL", "Auto": "AUTO"}


def main(root):
    for path in sorted(glob.glob(f"{root}/verb/*.xml")):
        t = ET.parse(path).getroot()
        lemma = t.get("default", "")
        if not lemma or " " in lemma or not lemma[0].islower():
            continue
        rows = set()
        for vn in t.findall("verbalNoun"):
            rows.add((vn.get("default"), "V;VN"))
        for va in t.findall("verbalAdjective"):
            rows.add((va.get("default"), "V.PTCP"))
        for tf in t.findall("tenseForm"):
            if tf.get("dependency") != "Indep":
                continue
            tense = TENSE.get(tf.get("tense"))
            person = PERSON.get(tf.get("person"))
            if tense and person:
                rows.add((tf.get("default"), f"V;{tense};{person}"))
        for mf in t.findall("moodForm"):
            mood = {"Imper": "IMP", "Subj": "SBJV"}.get(mf.get("mood"))
            person = PERSON.get(mf.get("person"))
            if mood and person:
                rows.add((mf.get("default"), f"V;{mood};{person}"))
        for form, feat in sorted(rows):
            if form and " " not in form:
                print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
