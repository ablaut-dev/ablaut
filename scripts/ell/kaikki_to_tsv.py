#!/usr/bin/env python3
"""Convert the kaikki.org (Modern) Greek verb extraction to the shared TSV,
under the same UniMorph-ell feature strings so the harness can intersect
the two into agreement gold.

Greek is cited by its 1sg imperfective present. The synthetic (one-word)
paradigm is the imperfective present and past, the perfective past
(aorist), the imperative and the active present participle (gerund); the
perfect, future and subjunctive are analytic (έχω/θα/να + form) and are
dropped here as they are multi-word. Active voice only, to match the
UniMorph-ell verb tables.
"""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
DROP = {"romanization", "canonical", "table-tags", "inflection-template",
        "alternative", "passive", "dependent", "obsolete", "archaic",
        "informal", "rare", "dialectal", "learned"}


def feature(tags):
    t = {x.lower() for x in tags}
    if t & DROP:
        return None
    person = next((v for k, v in PERSON.items() if k in t), None)
    number = next((v for k, v in NUMBER.items() if k in t), None)

    if "participle" in t and "active" in t and "present" in t:
        return "V;PTCP"
    if "imperative" in t and number:
        return f"V;2;{number};IMP"
    if not (person and number):
        return None
    if "present" in t and "imperfective" in t:
        return f"V;{person};{number};IPFV;PRS"
    if "imperfect" in t:  # imperfective past
        return f"V;{person};{number};IPFV;PST"
    if "past" in t and "perfective" in t:  # aorist
        return f"V;{person};{number};PFV;PST"
    return None


def main(path):
    for line in open(path):
        e = json.loads(line)
        if e.get("pos") != "verb":
            continue
        lemma = e.get("word", "")
        if not lemma or " " in lemma:
            continue
        rows = set()
        for f in e.get("forms", []):
            form = f.get("form", "").lstrip("{").strip()
            if not form or form == "-" or " " in form:
                continue
            feat = feature(f.get("tags", []))
            if feat:
                rows.add((form, feat))
        for form, feat in sorted(rows):
            print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
