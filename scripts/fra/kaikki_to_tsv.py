#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) French verb JSONL into the
lemma<TAB>form<TAB>features TSV that the French golden harness reads.

Usage: python3 scripts/fra/kaikki_to_tsv.py data/fra/kaikki-verbs.jsonl > data/fra/kaikki.tsv

Only synthetic conjugation-table forms are kept; compound tenses
(multiword-construction) are the compositional layer's business, as in German.
The feature schema matches scripts/fra/lefff_to_tsv.py so cross_oracle.py can
compare the two directly.
"""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
SKIP = {"table-tags", "inflection-template", "class", "multiword-construction"}

# Pronominal-verb tables carry reflexive clitics ("s'absentant", "nous
# absenterions", "absente-toi"); the Lefff stores bare forms. Strip them so
# the two oracles describe the same object (same policy as German reflexives).
PROCLITICS = ("me ", "te ", "se ", "nous ", "vous ", "m'", "t'", "s'", "m’", "t’", "s’")
ENCLITICS = ("-toi", "-nous", "-vous")


def strip_clitics(form):
    for c in PROCLITICS:
        if form.startswith(c):
            return form[len(c):]
    for c in ENCLITICS:
        if form.endswith(c):
            return form[: -len(c)]
    return form


def features(tags):
    """Map a kaikki tag set to a feature string, or None."""
    if tags & SKIP:
        return None
    p = next((PERSON[t] for t in tags if t in PERSON), None)
    n = next((NUMBER[t] for t in tags if t in NUMBER), None)
    if "infinitive" in tags:
        return "V;NFIN"
    if "participle" in tags:
        if "present" in tags:
            return "V.PTCP;PRS"
        # The conjugation table lists only the masculine singular.
        return "V.PTCP;PST;MASC;SG"
    if "imperative" in tags:
        return f"V;IMP;{n};{p}" if n and p else None
    if not (p and n):
        return None
    if "subjunctive" in tags:
        tense = "PST" if "imperfect" in tags else "PRS"
        return f"V;SBJV;{tense};{n};{p}"
    if "conditional" in tags:
        return f"V;COND;{n};{p}"
    if "indicative" in tags:
        if "present" in tags:
            return f"V;IND;PRS;{n};{p}"
        if "imperfect" in tags:
            return f"V;IND;PST;IPFV;{n};{p}"
        if "historic" in tags:
            return f"V;IND;PST;PFV;{n};{p}"
        if "future" in tags:
            return f"V;IND;FUT;{n};{p}"
    return None


def main(path):
    seen = set()
    for line in open(path):
        entry = json.loads(line)
        lemma = entry.get("word", "")
        forms = [f for f in entry.get("forms", []) if f.get("source") == "conjugation"]
        if not lemma or not forms:
            continue
        for f in forms:
            form = f.get("form", "").strip()
            if not form or form == "-" or form.startswith("—"):
                continue
            feat = features(set(f.get("tags", [])))
            if feat is None:
                continue
            form = strip_clitics(form)
            row = (lemma, form, feat)
            if row not in seen:
                seen.add(row)
                sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
