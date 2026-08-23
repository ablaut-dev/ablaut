#!/usr/bin/env python3
"""kaikki.org Luxembourgish verbs -> shared TSV, the synthetic (one-word)
core only: the present indicative (six person/number cells), the two
imperatives (2sg / 2pl) and the past participle, plus the infinitive
(citation). Luxembourgish has no synthetic preterite for most verbs — the
past is periphrastic (hunn/sinn + past participle) — and the conditional,
subjunctive, future and perfect are all multi-word, so they are dropped.

Separable-prefix verbs (zeréckzéien -> `zitts zeréck`) surface their finite
forms as two words; those are skipped (spaces), leaving the participle and
infinitive, which stay single tokens.

Usage: kaikki_to_tsv.py data/ltz/kaikki-verbs.jsonl > data/ltz/kaikki.tsv"""
import json
import sys

# Non-form / metadata tag-sets to reject outright.
JUNK = {
    "error-unrecognized-form", "table-tags", "inflection-template",
    "romanization", "alternative", "auxiliary", "no-table-tags",
}
# Multi-word / analytic tenses we do not score.
EXOTIC = {
    "conditional", "subjunctive", "preterite", "perfect", "future",
    "pluperfect", "progressive",
}


def feature(tags):
    t = {x.lower() for x in tags}
    if t & JUNK:
        return None
    if t & EXOTIC:
        return None
    # Past participle (the productive `ge-…-t`); bare "participle" repeats.
    if "participle" in t:
        return "V.PTCP;PST" if "past" in t else None
    if "infinitive" in t:
        return "V;NFIN"
    # Imperatives: only the true 2sg / 2pl are single-word; the 1/3-person
    # "imperatives" kaikki lists are hortatives (empty or periphrastic).
    if "imperative" in t:
        if "second-person" in t and "singular" in t:
            return "V;IMP;2;SG"
        if "second-person" in t and "plural" in t:
            return "V;IMP;2;PL"
        return None
    # Present indicative, six person/number cells.
    if "indicative" in t and "present" in t:
        p = ("1" if "first-person" in t else
             "2" if "second-person" in t else
             "3" if "third-person" in t else None)
        n = ("SG" if "singular" in t else
             "PL" if "plural" in t else None)
        if p and n:
            return f"V;IND;PRS;{n};{p}"
    return None


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
            if not form or " " in form or form == "-":
                continue
            ft = feature(f.get("tags", []))
            if ft:
                rows.add((form, ft))
        for form, ft in sorted(rows):
            print(f"{lemma}\t{form}\t{ft}")


if __name__ == "__main__":
    main(sys.argv[1])
