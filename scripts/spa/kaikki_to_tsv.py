#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Spanish verb JSONL into the
lemma<TAB>form<TAB>features TSV that the Spanish golden harness reads.

Usage: python3 scripts/spa/kaikki_to_tsv.py data/spa/kaikki-verbs.jsonl \
           > data/spa/kaikki.tsv

Skipped on purpose: vos-forms (voseo is a variant layer, not the core
paradigm), combined clitic forms (háblame — the compositional layer's
business), and negative imperatives (no + subjunctive, multiword). The
-se imperfect subjunctive lands in the same V;SBJV;PST slot as the -ra
one, mirroring FreeLing, so the two spellings are variants.
The feature schema matches scripts/spa/freeling_to_tsv.py.
"""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
GENDER = {"masculine": "MASC", "feminine": "FEM"}
SKIP = {
    "table-tags",
    "inflection-template",
    "vos-form",
    "combined-form",
    "negative",
    "multiword-construction",
    "second-person-semantically",
}


def features(tags):
    """Map a kaikki tag set to a feature string, or None."""
    if tags & SKIP:
        return None
    p = next((PERSON[t] for t in tags if t in PERSON), None)
    n = next((NUMBER[t] for t in tags if t in NUMBER), None)
    if "infinitive" in tags:
        return "V;NFIN"
    if "gerund" in tags:
        return "V;GER"
    if "participle" in tags:
        g = next((GENDER[t] for t in tags if t in GENDER), None)
        if g and n:
            return f"V.PTCP;PST;{g};{n}"
        return None
    if "imperative" in tags:
        return f"V;IMP;{n};{p}" if n and p else None
    if not (p and n):
        return None
    if "subjunctive" in tags:
        if "imperfect" in tags:
            return f"V;SBJV;PST;{n};{p}"
        if "future" in tags:
            return f"V;SBJV;FUT;{n};{p}"
        if "present" in tags:
            return f"V;SBJV;PRS;{n};{p}"
        return None
    if "conditional" in tags:
        return f"V;COND;{n};{p}"
    if "indicative" in tags:
        if "present" in tags:
            return f"V;IND;PRS;{n};{p}"
        if "imperfect" in tags:
            return f"V;IND;PST;IPFV;{n};{p}"
        if "preterite" in tags:
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
            row = (lemma, form, feat)
            if row not in seen:
                seen.add(row)
                sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
