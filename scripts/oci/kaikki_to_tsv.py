#!/usr/bin/env python3
"""Convert the kaikki.org Occitan verb extraction to the shared TSV, under
the same UniMorph-oci feature strings so the harness can intersect the two
into agreement gold.

Occitan is cited by its infinitive. The scored synthetic paradigm is the
one UniMorph attests: the present, imperfect (IPFV) and preterite (PFV)
indicatives, the future and conditional, the present and imperfect
subjunctives (person/number), the three imperatives (2sg/1pl/2pl), the
infinitive, the past participle, the gerund (V.CVB;PRS) and — where
attested — the present participle.

kaikki carries several dialect tables per verb (Languedocien standard
first, then Provençal/Gascon variants) plus IPA transcriptions. The IPA
rows are dropped by an orthographic whitelist; `error-unrecognized-form`,
`table-tags`, `inflection-template`, `romanization`, `alternative`,
`multiword-construction` and the `conditional-ii` paradigm (absent from
UniMorph) are dropped by tag. Every remaining dialect spelling is kept:
the harness only needs UniMorph's form to lie in kaikki's variant set.
"""
import json
import sys

P = {"first-person": "1", "second-person": "2", "third-person": "3"}
N = {"singular": "SG", "plural": "PL"}
DROP = {"error-unrecognized-form", "table-tags", "inflection-template",
        "romanization", "alternative", "multiword-construction",
        "conditional-ii", "obsolete", "archaic", "literary"}

# Occitan orthography: letters + the accented vowels/cedilla + interpunct,
# apostrophe and hyphen. Anything else (IPA ʀ ˈ ŋ ɛ …, spaces) is not a
# spelled form and is skipped.
OK = set("abcdefghijklmnopqrstuvwxyzàáâçèéêíïîòóôúûü'·-")


def is_ortho(form):
    return bool(form) and all(c in OK for c in form)


def feature(tags):
    t = {x.lower() for x in tags}
    if t & DROP:
        return None
    if "infinitive" in t:
        return "V;NFIN"
    if "gerund" in t:
        return "V.CVB;PRS"
    if "participle" in t:
        # Only the bare (masc sg) participle; gendered/plural forms are
        # out of the UniMorph paradigm.
        if t & {"feminine", "masculine", "plural", "neuter"}:
            return None
        if "past" in t:
            return "V.PTCP;PST"
        if "present" in t:
            return "V.PTCP;PRS"
        return None
    person = next((v for k, v in P.items() if k in t), None)
    number = next((v for k, v in N.items() if k in t), None)
    if "imperative" in t:
        if "first-person" in t and number == "PL":
            return "V;1;PL;IMP"
        if number == "PL":
            return "V;2;PL;IMP"
        return "V;2;SG;IMP"  # bare second-person defaults to singular
    if "subjunctive" in t:
        # The bare "present;subjunctive" / "imperfect;subjunctive" rows are
        # the 1sg (Languedocien parle / parlèssi).
        person = person or "1"
        number = number or "SG"
        if "imperfect" in t:
            return f"V;{person};{number};SBJV;PST;IPFV"
        if "present" in t:
            return f"V;{person};{number};SBJV;PRS"
        return None
    if not (person and number):
        return None
    if "conditional" in t:
        return f"V;{person};{number};COND"
    if "indicative" in t:
        if "present" in t:
            return f"V;IND;PRS;{person};{number}"
        if "imperfect" in t:
            return f"V;IND;PST;{person};{number};IPFV"
        if "preterite" in t:
            return f"V;IND;PST;{person};{number};PFV"
        if "future" in t:
            return f"V;{person};{number};IND;FUT"
    return None


def main(path):
    for line in open(path):
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        if e.get("pos") != "verb":
            continue
        lemma = e.get("word", "")
        if not is_ortho(lemma):
            continue
        rows = set()
        for f in e.get("forms", []):
            form = f.get("form", "").strip()
            if not is_ortho(form):
                continue
            feat = feature(f.get("tags", []))
            if feat:
                rows.add((form, feat))
        for form, feat in sorted(rows):
            print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
