#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Tamil verb extraction to the
shared golden TSV `lemma <tab> form <tab> features`.

Tamil is cited by its verb root (செய், படி, வா). The synthetic core
scored here is the finite tense/PNG grid (past, present, future ×
1sg/1pl/2sg/2pl/3sgm/3sgf/3sgh/3sgn/3ple/3pln), the two imperatives, the
infinitive, the adverbial (verbal) participle, the three adjectival
(relative) participles and the conditional. Negatives are periphrastic
in the literary register and Wiktextract tags them inconsistently
(செய்யவில்லை fills a dozen cells), so they are out of scope; the
gerunds and pronominalised participles Wiktextract lists are noun
inflection and are dropped too.

Feature bundles follow a small UniMorph-style schema shared verbatim
with the ThamizhiMorph oracle (scripts/tam/thamizhi_gen.py) so the two
sources intersect in the harness.
"""
import json
import sys


def png(tags):
    """Person-number-gender token from a Wiktextract tag set, or None."""
    if "first-person" in tags:
        return "1SG" if "singular" in tags else "1PL" if "plural" in tags else None
    if "second-person" in tags:
        return "2SG" if "singular" in tags else "2PL" if "plural" in tags else None
    if "third-person" in tags:
        if "singular" in tags:
            if "masculine" in tags:
                return "3SGM"
            if "feminine" in tags:
                return "3SGF"
            if "honorific" in tags:
                return "3SGH"
            if "neuter" in tags:
                return "3SGN"
        if "plural" in tags:
            if "epicene" in tags:
                return "3PLE"
            if "neuter" in tags:
                return "3PLN"
    return None


TENSE = {"present": "PRS", "past": "PST", "future": "FUT"}


def slot(tags):
    """Map a Wiktextract tag set to a feature bundle, or None to skip."""
    if "negative" in tags:
        return None
    if "infinitive" in tags:
        return "V;INF"
    if "participle" in tags and "adverbial" in tags:
        return None if ("future" in tags or "past" in tags) else "V;CVB"
    if "participle" in tags and "adjectival" in tags:
        if "past" in tags:
            return "V;PTCP;PST"
        if "present" in tags:
            return "V;PTCP;PRS"
        if "future" in tags:
            return "V;PTCP;FUT"
        return None
    if "gerund" in tags or "potential" in tags or "cohortative" in tags:
        return None
    if "conditional" in tags:
        return None if ("informal" in tags or "future" in tags) else "V;COND"
    if "imperative" in tags:
        if "present" in tags or "past" in tags or "perfect" in tags:
            return None
        return "V;IMP;SG" if "singular" in tags else "V;IMP;PL" if "plural" in tags else None
    if "progressive" in tags or "effective" in tags or "perfect" in tags:
        return None
    t = next((v for k, v in TENSE.items() if k in tags), None)
    p = png(tags)
    if t and p:
        return f"V;{t};{p}"
    return None


def main(path):
    for line in open(path, encoding="utf-8"):
        e = json.loads(line)
        lemma = e.get("word", "")
        if not lemma or " " in lemma:
            continue
        rows = set()
        for f in e.get("forms", []):
            if f.get("source") != "conjugation":
                continue
            tags = set(f.get("tags", []))
            form = f.get("form", "")
            if not form or " " in form or form == "-" or "(" in form:
                continue
            feat = slot(tags)
            if feat:
                rows.add((form, feat))
        if not rows:
            continue
        rows.add((lemma, "V;NFIN"))
        for form, feat in sorted(rows):
            print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
