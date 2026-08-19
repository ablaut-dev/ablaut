#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Russian verb JSONL into the
lemma<TAB>form<TAB>features TSV that the Russian golden harness reads.

Usage: python3 scripts/rus/kaikki_to_tsv.py data/rus/kaikki-verbs.jsonl \
           > data/rus/kaikki.tsv

Russian specifics handled here:

* Stress marks (the combining acute U+0301 that kaikki prints on every
  form) are stripped: they are a pronunciation aid, absent from
  OpenCorpora and from ordinary orthography, so they must go before the
  two oracles can agree.
* The pre-reform ("dated") orthography layer (ъ, і, -ый → -ій) is
  skipped entirely.
* Imperfective verbs take a *periphrastic* future (буду + infinitive);
  those forms contain a space and are dropped — only the synthetic
  non-past is scored. A perfective verb's synthetic non-past is tagged
  `future` by Wiktionary and lands in V;IND;FUT; an imperfective's is
  tagged `present` and lands in V;IND;PRS.
* Participles and adverbial participles (gerunds) are out of the core
  finite paradigm and are not emitted.

Feature schema (shared with scripts/rus/opencorpora_to_tsv.py):
  V;NFIN
  V;IND;PRS;{SG,PL};{1,2,3}      imperfective synthetic non-past
  V;IND;FUT;{SG,PL};{1,2,3}      perfective synthetic non-past
  V;IND;PST;{MASC,FEM,NEUT};SG   past, singular, by gender
  V;IND;PST;PL                   past, plural (no gender)
  V;IMP;{SG,PL};2                imperative, 2nd person
"""
import json
import sys
import unicodedata

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
NUMBER = {"singular": "SG", "plural": "PL"}
GENDER = {"masculine": "MASC", "feminine": "FEM", "neuter": "NEUT"}
SKIP = {
    "table-tags",
    "inflection-template",
    "class",
    "dated",
    "participle",
    "adverbial",
    "active",
    "passive",
    "romanization",
    "canonical",
    "error-unknown-tag",
    "multiword-construction",
}


def destress(form):
    """Drop the combining acute (and rare grave); keep ё, a real letter."""
    return "".join(
        c
        for c in unicodedata.normalize("NFC", form)
        if c not in ("́", "̀")
    )


def features(tags):
    """Map a kaikki tag set to a feature string, or None."""
    if tags & SKIP:
        return None
    p = next((PERSON[t] for t in tags if t in PERSON), None)
    n = next((NUMBER[t] for t in tags if t in NUMBER), None)
    if "infinitive" in tags:
        return "V;NFIN"
    if "imperative" in tags:
        return f"V;IMP;{n};2" if n else None
    if "past" in tags:
        if n == "PL":
            return "V;IND;PST;PL"
        g = next((GENDER[t] for t in tags if t in GENDER), None)
        return f"V;IND;PST;{g};SG" if g else None
    if not (p and n):
        return None
    if "future" in tags:
        return f"V;IND;FUT;{n};{p}"
    if "present" in tags:
        return f"V;IND;PRS;{n};{p}"
    return None


def main(path):
    seen = set()
    for line in open(path):
        entry = json.loads(line)
        lemma = entry.get("word", "")
        forms = [f for f in entry.get("forms", []) if f.get("source") == "conjugation"]
        if not lemma or not forms:
            continue
        lemma = destress(lemma)
        for f in forms:
            form = destress(f.get("form", "").strip())
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
