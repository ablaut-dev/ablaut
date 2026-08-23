#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Latin verb extraction to the shared
TSV, under the same UniMorph-lat feature strings so the harness can
intersect the two into agreement gold.

Latin is cited by its 1sg present active indicative (amō, moneō, regō,
capiō, audiō; deponents by their 1sg -or: hortor, loquor). We take that
1sg form — macron-bearing, exactly as UniMorph keys it — as the lemma, so
the two oracles share a key. Length marks (macrons) are kept: both oracles
carry them on every form, so stripping would only destroy information.

Scope: the present-system active-indicative core — present, imperfect and
(simple) future indicative, the present imperative and the present active
infinitive. Deponents inflect with passive morphology but Wiktextract (and
UniMorph) label those forms ACT, so they intersect without special casing.
Subjunctives, the perfect system, the passive of non-deponents, participles,
supines and gerunds are a later pass.

Usage: kaikki_to_tsv.py data/lat/kaikki-verbs.jsonl > data/lat/kaikki.tsv
"""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
DROP = {"error-unrecognized-form", "table-tags", "inflection-template",
        "romanization", "alternative", "multiword-construction", "canonical"}


def feature(tags):
    t = {x.lower() for x in tags}
    if t & DROP:
        return None
    p = next((v for k, v in PERSON.items() if k in t), None)
    n = next((v for k, v in NUMBER.items() if k in t), None)

    # Present active infinitive (deponents label it neither active nor
    # passive; non-deponents carry a bare and an explicit-active copy).
    if "infinitive" in t and "present" in t:
        if "passive" in t or "perfect" in t or "future" in t:
            return None
        return "V;NFIN;ACT;PRS"

    # Present imperative (2sg / 2pl). Skip the future imperative and the
    # passive imperative of non-deponents.
    if "imperative" in t:
        if "future" in t or "passive" in t or "present" not in t or n is None:
            return None
        return f"V;IMP;ACT;PRS;2;{n}"

    # Finite indicative. Require an explicit 'active' tag: that admits the
    # deponents (labelled active) while excluding the passive column of
    # ordinary verbs. Perfect-system cells all carry 'perfect'/'pluperfect'.
    if "indicative" not in t or "active" not in t or not (p and n):
        return None
    if "perfect" in t or "pluperfect" in t:
        return None
    if "present" in t:
        return f"V;IND;ACT;PRS;{p};{n}"
    if "imperfect" in t:
        return f"V;IND;ACT;PST;IPFV;{p};{n}"
    if "future" in t:
        return f"V;IND;ACT;FUT;{p};{n}"
    return None


def lemma_of(entry):
    """The 1sg present active indicative form (macron-bearing), matching
    the UniMorph lemma key. None if the entry has no such cell."""
    for f in entry.get("forms", []):
        t = {x.lower() for x in f.get("tags", [])}
        if {"active", "indicative", "present", "first-person",
                "singular"} <= t and "perfect" not in t:
            form = f.get("form", "").strip()
            if form and form != "-" and " " not in form:
                return form
    return None


def main(path):
    for line in open(path):
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        if e.get("pos") != "verb":
            continue
        lemma = lemma_of(e)
        if not lemma:
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
