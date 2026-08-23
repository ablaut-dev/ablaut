#!/usr/bin/env python3
"""Adapt the kaikki.org (Wiktextract) Arabic verb extraction to the shared
lemma⁩⁦form⁩⁦feat TSV, using the SAME canonical feature vocabulary as the
UniMorph adapter so the golden harness can intersect them.

The unvoweled skeleton is a homograph across the derived forms (كتب is both
Form I كَتَبَ "write" and Form II كَتَّبَ "make write"). Wiktionary lists each as
a separate entry tagged form-i / form-ii / … ; when several readings share an
unvoweled lemma we keep the one from the LOWEST form class per cell, so the
default conjugation is the basic verb rather than an arbitrary derived one.

Cross-oracle normalisations: 1st person carries no gender; perfect =
past+perfective+indicative; imperfect indicative = non-past+imperfective+
indicative; subjunctive/jussive/imperative are mood-only; participles dropped.
"""
import json, sys, unicodedata

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "dual": "DU", "plural": "PL"}
GENDER = {"masculine": "MASC", "feminine": "FEM"}
ROMAN = {"i":1,"ii":2,"iii":3,"iv":4,"v":5,"vi":6,"vii":7,"viii":8,"ix":9,"x":10,
         "xi":11,"xii":12,"xiii":13,"xiv":14,"xv":15}

def form_rank(tags):
    for t in tags:
        if t.startswith("form-"):
            return ROMAN.get(t[5:], 99)
    return 99

def canon(tagset):
    t = set(tagset)
    if "participle" in t or "noun-from-verb" in t:
        return None
    person = next((PERSON[k] for k in PERSON if k in t), None)
    number = next((NUMBER[k] for k in NUMBER if k in t), None)
    gender = next((GENDER[k] for k in GENDER if k in t), None)
    voice = "PASS" if "passive" in t else "ACT"
    if "imperative" in t:
        mood = ["IMP"]
    elif "jussive" in t:
        mood = ["JUS"]
    elif "subjunctive" in t:
        mood = ["SBJV"]
    elif "non-past" in t or "imperfective" in t:
        mood = ["IPFV", "IND"]
    elif "past" in t or "perfective" in t:
        mood = ["PST", "PRF", "IND"]
    else:
        return None
    if person is None or number is None:
        return None
    if person == "1":
        gender = None
    body = [person, number] + ([gender] if gender else []) + mood + [voice]
    return "V;" + ";".join(sorted(body))

def main(path):
    # (lemma, feat) -> (best_rank, form)
    best = {}
    for line in open(path):
        try:
            e = json.loads(line)
        except Exception:
            continue
        lemma = unicodedata.normalize("NFC", (e.get("word") or "").strip())
        if not lemma or " " in lemma:
            continue
        rank = form_rank([x for f in e.get("forms", []) if "canonical" in f.get("tags", []) for x in f["tags"]])
        for f in e.get("forms", []):
            form = unicodedata.normalize("NFC", (f.get("form") or "").strip())
            tags = f.get("tags", [])
            if not form or " " in form or "romanization" in tags:
                continue
            c = canon(tags)
            if not c:
                continue
            key = (lemma, c)
            cur = best.get(key)
            if cur is None or rank < cur[0]:
                best[key] = (rank, form)
    for (lemma, c), (_, form) in best.items():
        print(f"{lemma}\t{form}\t{c}")

if __name__ == "__main__":
    main(sys.argv[1])
