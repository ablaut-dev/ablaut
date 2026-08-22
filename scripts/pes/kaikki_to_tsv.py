#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Persian verb extraction into the
common golden-harness TSV: `lemma <TAB> form <TAB> features`.

Features are a small UniMorph-style bundle shared with the apertium
oracle (see scripts/pes/apertium_to_tsv.py) so the harness can intersect
them. kaikki carries several registers per lemma (literary, colloquial
Tehrani, Dari); every variant is emitted — the harness unions them and
the agreement with apertium selects the literary form.

All forms are normalized with the Perso-Arabic rules of norm.py (the
twin of src/perso_arabic.rs), matching what the engine emits.

Usage: python3 scripts/pes/kaikki_to_tsv.py data/pes/kaikki-verbs.jsonl > data/pes/kaikki.tsv
"""
import json
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from norm import normalize  # noqa: E402


def person(t):
    for k, v in (("first-person", "1"), ("second-person", "2"), ("third-person", "3")):
        if k in t:
            return v
    return None


def number(t):
    if "singular" in t:
        return "SG"
    if "plural" in t:
        return "PL"
    return None


def features(tags):
    """Map a kaikki tag set to the shared feature bundle, or None to skip."""
    t = set(tags)
    # non-finite
    if tags == ["infinitive"]:
        return "V;NFIN"
    if t == {"participle", "past"}:
        return "V;PTCP;PST"
    if t == {"participle", "present"}:
        return "V;PTCP;PRS"
    if "imperative" in t:
        n = number(t)
        return f"V;IMP;2;{n}" if n and person(t) == "2" else None
    p, n = person(t), number(t)
    if not p or not n:
        return None
    pn = f"{p};{n}"
    if "subjunctive" in t:
        # present subjunctive (بکنم) vs perfect subjunctive (کرده باشم)
        return f"V;SBJV;PRF;{pn}" if "past" in t else f"V;SBJV;{pn}"
    if "aorist" in t:
        return f"V;AOR;{pn}"
    if "progressive" in t:
        base = "PRS" if "present" in t else "PST"
        return f"V;IND;{base};PROG;{pn}"
    if "perfect" in t and "present" in t:
        return f"V;IND;PRF;{pn}"
    if "pluperfect" in t:
        return f"V;IND;PLUP;{pn}"
    if "future" in t:
        return f"V;FUT;{pn}"
    if "present" in t and "indicative" in t:
        return f"V;IND;PRS;{pn}"  # the می-present (and, unmarked, imperfect)
    if "past" in t and "indicative" in t:
        return f"V;IND;PST;{pn}"
    return None


def main():
    path = sys.argv[1]
    seen = set()
    for line in open(path, encoding="utf-8"):
        d = json.loads(line)
        lemma = normalize(d.get("word", ""))
        if not lemma:
            continue
        for f in d.get("forms", []):
            feat = features(f.get("tags", []))
            if not feat:
                continue
            form = normalize(f.get("form", ""))
            if not form or "no-table-tags" in form:
                continue
            row = (lemma, form, feat)
            if row not in seen:
                seen.add(row)
                print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main()
