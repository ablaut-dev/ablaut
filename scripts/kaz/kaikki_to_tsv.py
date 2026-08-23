#!/usr/bin/env python3
"""kaikki.org Kazakh verbs -> shared TSV, the synthetic (one-word) core
only: the aorist / present-future (stem + а/е/й + full endings), the
definite past / preterite (stem + ды/ті + possessive endings) and the
presumptive future (stem + ар/ер/р + full endings), each across eight
person/number cells (the 2nd person splits informal / formal).

Only single-word forms are kept. The perfect (-ған), renarrative (-ыпты),
habitual-past (-атын), intentive (-мақ), conditional (-са) and optative
(periphrastic, "… келеді") tenses are dropped: several are analytic
multi-word forms, and kaikki conflates a few of them under the same
person/number tags, so this pass stays conservative. Kazakh Cyrillic has
no stress/length diacritics to strip.
"""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
# Tenses that are analytic, conflated, or left to a later pass.
DROP = {"error-unrecognized-form", "table-tags", "inflection-template",
        "romanization", "alternative", "abbreviation", "impersonal",
        "participle", "perfect", "pluperfect", "renarrative", "habitual",
        "intentive", "optative", "conditional", "perfective",
        "imperfective", "transitional-past", "future", "past", "nominative",
        "genitive", "dative", "accusative", "locative", "ablative",
        "instrumental", "personal"}
TENSE = {"aorist": "AOR", "preterite": "PST", "presumptive": "FUT"}


def feature(tags):
    t = {x.lower() for x in tags}
    if t & DROP:
        return None
    p = next((v for k, v in PERSON.items() if k in t), None)
    n = next((v for k, v in NUMBER.items() if k in t), None)
    if not (p and n):
        return None
    pol = ""
    if p == "2":
        if "formal" in t:
            pol = ";FORM"
        elif "informal" in t:
            pol = ";INFM"
        else:
            return None
    tense = next((v for k, v in TENSE.items() if k in t), None)
    if not tense:
        return None
    return f"V;{tense};{p};{n}{pol}"


def main(path):
    for line in open(path):
        try:
            e = json.loads(line)
        except Exception:
            continue
        if e.get("pos") != "verb":
            continue
        lemma = e.get("word", "")
        if not lemma or " " in lemma:
            continue
        rows = set()
        for f in e.get("forms", []):
            form = f.get("form", "").strip()
            if not form or " " in form or form == "-":
                continue
            ft = feature(f.get("tags", []))
            if ft:
                rows.add((form, ft))
        for form, ft in sorted(rows):
            print(f"{lemma}\t{form}\t{ft}")


if __name__ == "__main__":
    main(sys.argv[1])
