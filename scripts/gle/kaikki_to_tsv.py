#!/usr/bin/env python3
"""Convert the kaikki.org Irish verb extraction to the shared TSV.

Analytic rows (glanann tú) are multi-word and skipped; the synthetic
forms remain. Initial mutations are normalized away — the shared
convention is the unmutated citation form (ghlan → glan, d'ól → ól) —
since kaikki prints past-tense lenition and BuNaMo does not.
"""
import json
import sys

SKIP = {"error-unrecognized-form", "table-tags", "inflection-template",
        "archaic", "obsolete", "dialectal", "relative", "dependent"}
PER = {"first-person": "1", "second-person": "2", "third-person": "3"}


def unmutate(form, lemma):
    if form.startswith("d'"):
        form = form[2:]
        if form and form[0] == "f" and len(form) > 1 and form[1] == "h":
            form = form[0] + form[2:]
    if (len(form) > 2 and form[1] == "h" and lemma and form[0] == lemma[0]
            and (len(lemma) < 2 or lemma[1] != "h")):
        form = form[0] + form[2:]
    return form


def slot(tags):
    n = "SG" if "singular" in tags else "PL" if "plural" in tags else None
    p = next((v for k, v in PER.items() if k in tags), None)
    pn = f"{p}{n}" if n and p else None
    if "autonomous" in tags:
        if "imperative" in tags:
            return "V;IMP;AUTO"
        if "subjunctive" in tags:
            return "V;SBJV;AUTO"
        if "habitual" in tags and "past" in tags:
            return "V;PSTHAB;AUTO"
        if "past" in tags:
            return "V;PST;AUTO"
        if "future" in tags:
            return "V;FUT;AUTO"
        if "conditional" in tags:
            return "V;COND;AUTO"
        if "present" in tags:
            return "V;PRS;AUTO"
        return None
    if "imperative" in tags:
        return f"V;IMP;{pn}" if pn else None
    if "subjunctive" in tags:
        return f"V;SBJV;{pn}" if pn else None
    if "conditional" in tags:
        return f"V;COND;{pn}" if pn else None
    if "future" in tags:
        return f"V;FUT;{pn}" if pn else None
    if "habitual" in tags and "past" in tags:
        return f"V;PSTHAB;{pn}" if pn else None
    if "past" in tags:
        return f"V;PST;{pn}" if pn else None
    if "present" in tags and "habitual" not in tags:
        return f"V;PRS;{pn}" if pn else None
    return None


def main(path):
    for line in open(path):
        e = json.loads(line)
        lemma = e.get("word", "")
        if not lemma or " " in lemma or not lemma[0].islower():
            continue
        rows = set()
        for f in e.get("forms", []):
            tags = set(f.get("tags", []))
            if tags & SKIP or f.get("source") != "conjugation":
                continue
            form = f.get("form", "")
            if not form or " " in form or form == "-":
                continue
            feat = slot(tags)
            if feat:
                rows.add((unmutate(form, lemma), feat))
            elif "verbal-noun" in tags or "noun-from-verb" in tags:
                rows.add((form, "V;VN"))
            elif "participle" in tags:
                rows.add((form, "V.PTCP"))
        if len(rows) >= 8:
            for form, feat in sorted(rows):
                print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
