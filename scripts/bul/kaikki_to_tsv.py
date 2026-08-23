#!/usr/bin/env python3
"""Convert the kaikki.org Bulgarian verb extraction to the shared TSV,
under the same UniMorph-bul feature strings so the harness can intersect
the two into agreement gold.

kaikki marks stress with a combining acute (U+0301) that UniMorph does not
carry, so every form is stress-stripped first. Bulgarian is cited by its
1sg present; the paradigm is present, aorist and imperfect (person/number),
the imperative, the active past l-participle (aorist-based = plain,
imperfect-based = renarrative/NFH) with the full definite-article system
(indefinite, definite, and NOM/ACC on the masculine definite), the present
active participle, the passive participle, the verbal noun and the converb.
"""
import json
import sys
import unicodedata

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
GENDER = {"masculine": "MASC", "feminine": "FEM", "neuter": "NEUT"}
DROP = {"romanization", "canonical", "table-tags", "inflection-template",
        "dubitative", "renarrative", "conclusive", "obsolete", "archaic",
        "rare", "dialectal"}


def destress(s):
    return "".join(c for c in unicodedata.normalize("NFD", s)
                   if unicodedata.category(c) != "Mn")


def definiteness(t):
    """Return the ;INDF / ;DEF / ;DEF;NOM / ;DEF;ACC suffix."""
    if "definite" in t:
        if "subjective" in t:
            return ";DEF;NOM"
        if "objective" in t:
            return ";DEF;ACC"
        return ";DEF"
    return ";INDF"


def feature(tags):
    t = {x.lower() for x in tags}
    if t & DROP:
        return None
    person = next((v for k, v in PERSON.items() if k in t), None)
    number = next((v for k, v in NUMBER.items() if k in t), None)
    gender = next((v for k, v in GENDER.items() if k in t), None)

    if "adverbial" in t or "gerund" in t or "converb" in t:
        return "V.CVB"
    if "noun-from-verb" in t:  # verbal noun (neuter)
        num = "PL" if "plural" in t else "NEUT;SG"
        return f"V.MSDR;{num}{definiteness(t)}"
    if "imperative" in t and number:
        return f"V;IMP;2;{number}"
    if "participle" in t:
        gnum = gender + ";SG" if gender else ("PL" if number == "PL" else None)
        if gnum is None:
            return None
        if "passive" in t:
            return f"V.PTCP;PASS;PST;{gnum}{definiteness(t)}"
        if "present" in t:
            return f"V.PTCP;ACT;PRS;{gnum}{definiteness(t)}"
        if "aorist" in t:
            return f"V.PTCP;ACT;PST;{gnum}{definiteness(t)}"
        if "imperfect" in t:  # renarrative l-participle; UniMorph: NFH, indefinite only
            if "definite" in t:
                return None
            return f"V.PTCP;ACT;PST;NFH;{gnum};INDF"
        return None
    # Finite indicative cells.
    if person and number:
        if "present" in t:
            return f"V;IND;PRS;{person};{number}"
        if "aorist" in t:
            return f"V;IND;PST;{person};{number}"
        if "imperfect" in t:
            return f"V;IND;PROG;PST;{person};{number}"
    return None


def main(path):
    for line in open(path):
        e = json.loads(line)
        if e.get("pos") != "verb":
            continue
        lemma = destress(e.get("word", ""))
        if not lemma or " " in lemma:
            continue
        rows = set()
        for f in e.get("forms", []):
            form = destress(f.get("form", ""))
            if not form or form == "-" or " " in form:
                continue
            feat = feature(f.get("tags", []))
            if feat:
                rows.add((form, feat))
        for form, feat in sorted(rows):
            print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
