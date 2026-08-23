#!/usr/bin/env python3
"""kaikki.org Galician verbs -> shared TSV (lemma<TAB>form<TAB>features).

Scope: the synthetic (one-word) core of the Galician verb. Ten
person/number tense blocks — present, imperfect, preterite, synthetic
pluperfect, future and conditional (indicative); present, imperfect and
future subjunctive; and the inflected (personal) infinitive — plus the
2sg/2pl imperative, the impersonal infinitive (the citation), the gerund
and the four past-participle gender/number forms. Periphrastic (compound)
tenses are out of scope.

Wiktextract quirk that we exploit: in the Galician tables the third-person
row is emitted with the tag `error-unrecognized-form` and NO explicit
person tag (verified: 0 of ~70k such rows ever carry a person tag), so
`error-unrecognized-form` is read as third person. That recovers the whole
6-cell paradigm; without it every third-person form would be lost. The
reintegrationist (Portuguese-orthography) `gl-reinteg-conj` table's forms
are kept too — they land in the same cells as extra accepted variants, so
the standard RAG spelling the engine emits still scores as correct.

Usage: kaikki_to_tsv.py data/glg/kaikki-verbs.jsonl > data/glg/kaikki.tsv"""
import json
import sys

P = {"first-person": "1", "second-person": "2", "third-person": "3"}
N = {"singular": "SG", "plural": "PL"}


def person(t):
    for k, v in P.items():
        if k in t:
            return v
    # Untagged third-person row: wiktextract marks it error-unrecognized-form.
    if "error-unrecognized-form" in t:
        return "3"
    return None


def feature(tags):
    t = {x.lower() for x in tags}
    if t & {"table-tags", "inflection-template", "romanization",
            "alternative", "canonical", "no-table-tags"}:
        return None
    # Participle: keyed by gender+number only (person tags on it are noise).
    if "participle" in t:
        if "short-form" in t or "long-form" in t:
            return None
        g = "MASC" if "masculine" in t else ("FEM" if "feminine" in t else None)
        n = N.get("plural" if "plural" in t else "singular")
        return f"V;PTCP;{g};{n}" if g else None
    if "gerund" in t:
        return "V;GER"
    if "impersonal" in t and "infinitive" in t:
        return "V;NFIN"
    n = "PL" if "plural" in t else ("SG" if "singular" in t else None)
    p = person(t)
    if "infinitive" in t:  # inflected (personal) infinitive
        return f"V;INF;{p};{n}" if p and n else None
    if "imperative" in t:
        if "negative" in t or p != "2" or not n:
            return None
        return f"V;IMP;2;{n}"
    if not (p and n):
        return None
    if "subjunctive" in t:
        if "present" in t:
            return f"V;SBJV;PRS;{p};{n}"
        if "imperfect" in t:
            return f"V;SBJV;PST;{p};{n}"
        if "future" in t:
            return f"V;SBJV;FUT;{p};{n}"
        return None
    if "conditional" in t:
        return f"V;COND;{p};{n}"
    if "indicative" in t:
        if "present" in t:
            return f"V;IND;PRS;{p};{n}"
        if "imperfect" in t:
            return f"V;IND;IPFV;{p};{n}"
        if "preterite" in t:
            return f"V;IND;PRET;{p};{n}"
        if "pluperfect" in t:
            return f"V;IND;PLUP;{p};{n}"
        if "future" in t:
            return f"V;IND;FUT;{p};{n}"
    return None


def main(path):
    for line in open(path):
        try:
            e = json.loads(line)
        except Exception:
            continue
        if e.get("pos") != "verb":
            continue
        lemma = e.get("word", "")
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
