#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Romanian verb JSONL into the
lemma<TAB>form<TAB>features TSV the Romanian golden harness reads.

Usage: python3 scripts/ron/kaikki_to_tsv.py data/ron/kaikki-verbs.jsonl \
           > data/ron/kaikki.tsv

The infinitive is listed with its particle ("a vorbi") and the
subjunctive with să — both markers are stripped so the slots align with
dexonline's bare forms. Negative imperatives (nu + infinitive) are
skipped. Romanian's future is analytic (voi vorbi) and out of scope.
"""
import json
import sys

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
SKIP = {"table-tags", "inflection-template", "negative", "combined-form",
        "multiword-construction"}


def features(tags):
    if tags & SKIP:
        return None
    p = next((PERSON[t] for t in tags if t in PERSON), None)
    n = next((NUMBER[t] for t in tags if t in NUMBER), None)
    if "infinitive" in tags:
        return "V;NFIN"
    if "gerund" in tags:
        return "V;GER"
    if "participle" in tags:
        return "V.PTCP;PST;MASC;SG"
    if "imperative" in tags:
        return f"V;IMP;{n};{p}" if n and p else None
    if not (p and n):
        return None
    if "subjunctive" in tags:
        if "present" in tags:
            return f"V;SBJV;PRS;{n};{p}"
        return None
    if "indicative" in tags:
        if "present" in tags:
            return f"V;IND;PRS;{n};{p}"
        if "imperfect" in tags:
            return f"V;IND;PST;IPFV;{n};{p}"
        if "pluperfect" in tags:
            return f"V;IND;PST;PQP;{n};{p}"
        if "perfect" in tags:
            return f"V;IND;PST;PFV;{n};{p}"
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
            for marker in ("a ", "să "):
                if form.startswith(marker):
                    form = form[len(marker):]
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
