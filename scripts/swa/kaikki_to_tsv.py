#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Swahili verb JSONL into the
canonical `lemma ⇥ form ⇥ features` TSV, the same schema swc_to_tsv.py
emits, so the golden harness can intersect the two oracles.

kaikki keys its conjugating verb entries under the *bare stem* (`soma`,
`fua`), matching swc's lemma column. Its conjugation tables spell out
fully-inflected forms only for the person subjects and noun classes 1/2
(plus, for the gnomic/a-tense, every class); the other class × tense
cells are template strings like `positive subject concord + -lisoma`,
which carry a space and are dropped here. That is why the scored
agreement is the productive core — the infinitive, imperative, habitual,
present, subjunctive and the all-class gnomic — while past/future/
perfect/conditional survive only where kaikki spells a real word (the
person and class-1/2 cells).

Negatives are present in kaikki and absent from swc, so they are emitted
(tagged `;NEG`) but never enter the two-oracle agreement.

Usage: python3 scripts/swa/kaikki_to_tsv.py data/swa/kaikki-verbs.jsonl
"""

import json
import sys

# kaikki tags a class-pair together (e.g. class-1 & class-2); singular
# picks the first, plural the second. Locative classes are singular-only.
CLASS_PAIR = {
    ("class-1", "class-2"): (1, 2),
    ("class-3", "class-4"): (3, 4),
    ("class-5", "class-6"): (5, 6),
    ("class-7", "class-8"): (7, 8),
    ("class-9", "class-10"): (9, 10),
}
CLASS_SINGLE = {
    "class-11": 11,
    "class-15": 15,
    "class-16": 16,
    "class-17": 17,
    "class-18": 18,
}

SKIP = {
    "table-tags",
    "inflection-template",
    "error-unrecognized-form",
    "object-concord",
    "relative",
    "reflexive",
}


def subject(t):
    """The canonical subject token for a kaikki tag set, or None."""
    if "first-person" in t:
        return "1SG" if "singular" in t else "1PL"
    if "second-person" in t:
        return "2SG" if "singular" in t else "2PL"
    if "third-person" in t:
        for pair, (sg, pl) in CLASS_PAIR.items():
            if set(pair) <= t:
                return f"CL{sg if 'singular' in t else pl}"
        for tag, n in CLASS_SINGLE.items():
            if tag in t:
                return f"CL{n}"
    return None


def canonical(t):
    """Map a kaikki tag set to the canonical `V;TAM[;SUBJ][;NEG]`, or None."""
    neg = ";NEG" if "negative" in t else ""
    if "infinitive" in t:
        return f"V;NFIN{neg}"
    if "imperative" in t:
        if "singular" in t:
            return "V;IMP;SG"
        if "plural" in t:
            return "V;IMP;PL"
        return None
    if "habitual" in t:
        return "V;HAB"
    subj = subject(t)
    if subj is None:
        return None  # third-person-generic template rows carry no real form
    if "gnomic" in t:
        return f"V;GNOM;{subj}"  # the a-tense; polarity always positive
    if "subjunctive" in t and "consecutive" not in t:
        return f"V;SBJV;{subj}{neg}"
    if "present" in t and "irrealis" not in t:
        return f"V;PRS;{subj}{neg}"
    if "past" in t and "irrealis" not in t:
        return f"V;PST;{subj}{neg}"
    if "future" in t:
        return f"V;FUT;{subj}{neg}"
    if "perfect" in t:
        return f"V;PRF;{subj}{neg}"
    if "consecutive" in t:
        return f"V;SEQ;{subj}{neg}"
    if "if-when-form" in t:
        return f"V;SIT;{subj}{neg}"
    if "irrealis" in t and "present" in t:
        return f"V;CONDP;{subj}{neg}"
    if "irrealis" in t and "past" in t:
        return f"V;CONDPST;{subj}{neg}"
    return None


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            entry = json.loads(line)
            lemma = entry.get("word", "")
            forms = [
                f for f in entry.get("forms", []) if f.get("source") == "conjugation"
            ]
            if not lemma or not forms:
                continue
            for f in forms:
                form = f.get("form", "").strip()
                # Drop template remnants: kaikki renders the class × tense
                # cells it cannot spell as strings carrying a space
                # ("positive subject concord + -lisoma") or a leading
                # dash ("-waambie"); only whole words are real forms.
                if not form or not form.isalpha():
                    continue
                tags = set(f.get("tags", []))
                if tags & SKIP:
                    continue
                feat = canonical(tags)
                if feat:
                    rows.add((lemma, form, feat))
    out = sys.stdout
    for lemma, form, feat in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
