#!/usr/bin/env python3
"""Convert the kaikki.org Albanian verb extraction to the shared TSV,
under the same UniMorph-sqi feature strings so the harness can intersect
the two into agreement gold.

Albanian is cited by its 1sg present indicative. The synthetic (one-word)
paradigm kept here is the present, imperfect and aorist indicative, the
admirative present and imperfect, the imperative and the participle. The
subjunctive, optative, perfect and future are analytic (të / do / ka +
form) and are dropped as multi-word.
"""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
DROP = {"romanization", "canonical", "table-tags", "inflection-template",
        "error-unrecognized-form", "alternative", "obsolete", "archaic",
        "rare", "dialectal", "optative", "subjunctive", "perfect",
        "pluperfect", "future"}


def feature(tags):
    t = {x.lower() for x in tags}
    if t & DROP:
        return None
    if "participle" in t:
        return "V;V.PTCP"
    person = next((v for k, v in PERSON.items() if k in t), None)
    number = next((v for k, v in NUMBER.items() if k in t), None)
    if "imperative" in t and number:
        return f"V;2;{number};IMP"
    if not (person and number):
        return None
    if "admirative" in t:
        return f"V;{person};{number};ADM;{'IPFV' if 'imperfect' in t else 'PRS'}"
    if "imperfect" in t:
        return f"V;{person};{number};IND;IPFV"
    if "aorist" in t:
        return f"V;{person};{number};IND;PST"
    if "present" in t and "indicative" in t:
        return f"V;{person};{number};IND;PRS"
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
            form = f.get("form", "").strip()
            if not form or form == "-" or " " in form:
                continue
            feat = feature(f.get("tags", []))
            if feat:
                rows.add((form, feat))
        for form, feat in sorted(rows):
            print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
