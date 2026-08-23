#!/usr/bin/env python3
"""Adapt the kaikki.org (Wiktextract) Arabic verb extraction to the shared
lemma⇥form⇥feat TSV, using the SAME canonical feature vocabulary as the
UniMorph adapter so the golden harness can intersect them.

kaikki tags a finite verb form with a bag of words; we map that bag to the
canonical tokens and sort them. Cross-oracle normalisations that matter:
  * 1st person carries no gender in the Arabic verb — drop MASC/FEM there
    (kaikki tags it, UniMorph does not).
  * perfect = past+perfective+indicative → PST;PRF;IND
  * imperfect indicative = non-past+imperfective+indicative → IPFV;IND
  * subjunctive/jussive/imperative → SBJV / JUS / IMP (mood only)
  * participles decline like nouns → dropped in both adapters.
"""
import json, sys, unicodedata

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "dual": "DU", "plural": "PL"}
GENDER = {"masculine": "MASC", "feminine": "FEM"}

def canon(tagset):
    t = set(tagset)
    if "participle" in t or "noun-from-verb" in t:
        return None
    person = next((PERSON[k] for k in PERSON if k in t), None)
    number = next((NUMBER[k] for k in NUMBER if k in t), None)
    gender = next((GENDER[k] for k in GENDER if k in t), None)
    voice = "PASS" if "passive" in t else "ACT"
    # tense / mood
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
    for line in open(path):
        try:
            e = json.loads(line)
        except Exception:
            continue
        lemma = unicodedata.normalize("NFC", (e.get("word") or "").strip())
        if not lemma or " " in lemma:
            continue
        for f in e.get("forms", []):
            form = unicodedata.normalize("NFC", (f.get("form") or "").strip())
            tags = f.get("tags", [])
            if not form or " " in form or "romanization" in tags:
                continue
            c = canon(tags)
            if c:
                print(f"{lemma}\t{form}\t{c}")

if __name__ == "__main__":
    main(sys.argv[1])
