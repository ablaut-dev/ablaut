#!/usr/bin/env python3
"""Convert the kaikki.org Welsh verb extraction to the shared TSV, under the
same UniMorph-cym feature strings so the harness can intersect the two into
agreement gold.

Welsh is cited by its verbal noun (berfenw), e.g. "canu". The scored scope is
the SYNTHETIC (single-word) literary paradigm: the present(-future),
imperfect, preterite and pluperfect indicatives, the present subjunctive and
the imperative — each in six personal forms (1/2/3 x SG/PL) plus the
impersonal/autonomous form (UniMorph person "4") — together with the verbal
noun (V;V.MSDR) and the verbal adjective/participle (V;V.PTCP).

The colloquial paradigm carries the subject pronoun as a separate word
("cana i", "canith o"); those and the initial-mutation forms ('soft',
'nasal', 'aspirate') are periphrastic / non-lemmatic and are dropped, so only
single-word forms survive. The two morphological registers converge on the
same feature strings, matching UniMorph's V;LIT column.
"""
import json
import sys

P = {"first-person": "1", "second-person": "2", "third-person": "3"}
N = {"singular": "SG", "plural": "PL"}
DROP = {"error-unrecognized-form", "table-tags", "inflection-template",
        "romanization", "alternative", "colloquial",
        "soft", "nasal", "aspirate", "h-prothesis", "mixed-mutation"}


def feature(tags):
    t = {x.lower() for x in tags}
    if t & DROP:
        return None
    if "noun-from-verb" in t:
        return "V;V.MSDR"
    if "participle" in t:
        return "V;V.PTCP"
    # Person / number (impersonal = autonomous = UniMorph person 4).
    if "impersonal" in t:
        pers, num = "4", None
    else:
        pers = next((v for k, v in P.items() if k in t), None)
        num = next((v for k, v in N.items() if k in t), None)
        if not (pers and num):
            return None
    # Mood / tense (checked most specific first).
    if "imperative" in t:
        mood = "IMP"
    elif "subjunctive" in t:
        mood = "SBJV"
    elif "pluperfect" in t:
        mood = "IND;PST;PFV"
    elif "preterite" in t:
        mood = "IND;PST"
    elif "imperfect" in t:
        mood = "IND;IPFV"
    elif "present" in t:
        mood = "IND;PRS"
    else:
        return None
    if pers == "4":
        return f"V;LIT;4;{mood}"
    return f"V;LIT;{pers};{num};{mood}"


def main(path):
    for line in open(path):
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        if e.get("pos") != "verb":
            continue
        lemma = e.get("word", "").strip()
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
