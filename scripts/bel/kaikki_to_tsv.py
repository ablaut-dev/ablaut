#!/usr/bin/env python3
"""Convert the kaikki.org Belarusian verb extraction to the shared TSV,
under the same UniMorph-bel feature strings so the harness can intersect
the two into agreement gold.

Belarusian is cited by its infinitive (in -ць / -ці / -чы). The synthetic
one-word paradigm we score is: the present (imperfective verbs), the
synthetic future (perfective verbs — kaikki tags these `future,perfective`,
UniMorph tags them V;FUT), the past (l-participle with gender/number
agreement) and the imperative. The analytic imperfective future
(бу́ду … + infinitive) is multi-word and dropped by the space filter.

kaikki marks stress with a combining acute (U+0301) that Belarusian
orthography does not write, so every form is stress-stripped first, and
the several apostrophe glyphs are normalised to a plain U+0027.
"""
import json
import sys
import unicodedata

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
GENDER = {"masculine": "MASC", "feminine": "FEM", "neuter": "NEUT"}
DROP = {"romanization", "canonical", "table-tags", "inflection-template",
        "class", "alternative", "obsolete", "archaic", "rare", "dialectal",
        "taraškievica", "active", "passive", "adverbial", "nominative",
        "accusative", "genitive", "dative", "instrumental", "locative",
        "animate", "inanimate"}


def norm(s):
    # Strip only the acute/grave stress marks, NOT every combining mark:
    # Belarusian ў decomposes to у + combining breve (U+0306), so a blanket
    # Mn strip would destroy it. Re-compose (NFC) afterwards.
    s = unicodedata.normalize("NFD", s)
    s = "".join(c for c in s if c not in ("́", "̀"))
    s = unicodedata.normalize("NFC", s)
    return s.replace("’", "'").replace("ʼ", "'").replace("`", "'")


def feature(tags):
    t = {x.lower() for x in tags}
    if t & DROP:
        return None
    if "infinitive" in t:
        return None  # citation, not a scored cell
    person = next((v for k, v in PERSON.items() if k in t), None)
    number = next((v for k, v in NUMBER.items() if k in t), None)
    gender = next((v for k, v in GENDER.items() if k in t), None)

    if "imperative" in t and number:
        return f"V;IMP;2;{number}"
    # Past: l-participle, gender in the singular, bare number in the plural.
    if "past" in t and "participle" not in t:
        if number == "PL":
            return "V;PST;PL"
        if gender:
            return f"V;PST;SG;{gender}"
        return None
    # Synthetic non-past. present+person+number = imperfective present;
    # future+person+number = perfective synthetic future (one word).
    if person and number:
        if "present" in t:
            return f"V;PRS;{person};{number}"
        if "future" in t:
            return f"V;FUT;{person};{number}"
    return None


def main(path):
    for line in open(path):
        e = json.loads(line)
        if e.get("pos") != "verb":
            continue
        lemma = norm(e.get("word", ""))
        if not lemma or " " in lemma:
            continue
        rows = set()
        for f in e.get("forms", []):
            form = norm(f.get("form", ""))
            if not form or form == "-" or " " in form:
                continue
            feat = feature(f.get("tags", []))
            if feat:
                rows.add((form, feat))
        for form, feat in sorted(rows):
            print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
