#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Portuguese verb JSONL into the
lemma<TAB>form<TAB>features TSV the Portuguese golden harness reads.

Usage: python3 scripts/por/kaikki_to_tsv.py data/por/kaikki-verbs.jsonl \
           > data/por/kaikki.tsv

The European and Brazilian preterite doublets (falámos/falamos, tagged
Brazil) land in the same slot as variants — the AO90 analogue of the
French 1990 doublets. Skipped: negative imperatives (não + subjunctive,
multiword) and clitic combined forms.
"""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
GENDER = {"masculine": "MASC", "feminine": "FEM"}
SKIP = {"table-tags", "inflection-template", "negative", "combined-form", "multiword-construction"}


def features(tags):
    if tags & SKIP:
        return None
    p = next((PERSON[t] for t in tags if t in PERSON), None)
    n = next((NUMBER[t] for t in tags if t in NUMBER), None)
    if "infinitive" in tags:
        if "impersonal" in tags:
            return "V;NFIN"
        if p and n:
            return f"V;NFIN;{n};{p}"
        return None
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
    if "pluperfect" in tags:
        return f"V;IND;PST;PQP;{n};{p}"
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
            if not form or form == "-" or form.startswith("—") or " " in form:
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
