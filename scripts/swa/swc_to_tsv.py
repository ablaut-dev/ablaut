#!/usr/bin/env python3
"""Convert UniMorph swc (Swahili) to the shared `lemma ⇥ form ⇥ features`
TSV, in the canonical schema shared with the kaikki adapter.

Two adaptations align swc with kaikki (and with every other ablaut
oracle):

1. **Lemma normalization.** swc uses the *bare stem* as the lemma
   (`soma`, `fua`, `jibu`), which is exactly the head kaikki keys its
   conjugating entries under, so the two lemma columns already agree; no
   `ku-` has to be stripped or added.

2. **Noun-class decoding.** swc encodes the subject concord with LGSPEC
   tags: LGSPEC1..6 are the six class pairs (1/2, 3/4, 5/6, 7/8, 9/10,
   11/10), read as singular or plural off the `SG`/`PL` tag. Persons are
   the tagless `1;SG`, `2;SG`, `1;PL`, `2;PL`. The result is a `CL<n>`
   or person subject token identical to the one kaikki_to_tsv.py emits.

swc carries no negatives (they are kaikki-only, so never double-covered)
and no imperatives, and its LGSPEC7/8/9 tags (rare class readings) are
dropped.

Usage: python3 scripts/swa/swc_to_tsv.py data/swa/unimorph-swc.txt
"""

import sys

# LGSPEC1..6 → (singular class, plural class). Class 11 pluralizes as 10.
CLASS_PAIR = {
    "LGSPEC1": (1, 2),
    "LGSPEC2": (3, 4),
    "LGSPEC3": (5, 6),
    "LGSPEC4": (7, 8),
    "LGSPEC5": (9, 10),
    "LGSPEC6": (11, 10),
}


def subject(tags):
    """The canonical subject token for a swc tag set, or None to skip.

    Person is marked two ways in swc: the finite tenses use `1`/`2`/`3`
    with `SG`/`PL`, while the a-tense uses `INDF1`/`INDF2`/`INDF3`. Both
    are handled here so the a-tense (gnomic) lines up with kaikki.
    """
    t = set(tags)
    person = "1" if ("1" in t or "INDF1" in t) else "2" if ("2" in t or "INDF2" in t) else None
    third = "3" in t or "INDF3" in t
    num = "SG" if "SG" in t else "PL" if "PL" in t else None
    if person and num:
        return f"{person}{num}"
    if third:
        pair = next((CLASS_PAIR[g] for g in tags if g in CLASS_PAIR), None)
        if pair is None:
            return None
        return f"CL{pair[0] if 'SG' in t else pair[1]}"
    return None


def canonical(features):
    """Map a swc feature bundle to the canonical `V;TAM[;SUBJ]`, or None."""
    tags = features.split(";")
    t = set(tags)
    if "N" in t or "ADJ" in t:
        return None
    if "NFIN" in t:
        return "V;NFIN"
    if "V" not in t or "FIN" not in t:
        return None
    subj = subject(tags)
    # The a-tense (gnomic): swc tags it INDF (INDF1/2/3), sometimes with a
    # doubled IND. It carries the class the same way finite forms do.
    if any(x.startswith("INDF") for x in tags):
        return f"V;GNOM;{subj}" if subj else None
    if "SUBJ" in t:
        if "COM" in t:  # situative -ki-
            return f"V;SIT;{subj}" if subj else None
        if "LGSPEC10" in t:  # consecutive-subjunctive -ka-...-e
            return None
        return f"V;SBJV;{subj}" if subj else None  # true subjunctive -e
    if "IND" not in t:
        return None
    if "HAB" in t:
        return "V;HAB"
    if "LGSPEC10" in t:  # consecutive -ka-
        return f"V;SEQ;{subj}" if subj else None
    if "COND" in t:
        tam = "CONDP" if "PRES" in t else "CONDPST"
        return f"V;{tam};{subj}" if subj else None
    if "PRF" in t:
        return f"V;PRF;{subj}" if subj else None
    if "FUT" in t:
        return f"V;FUT;{subj}" if subj else None
    if "PST" in t:
        return f"V;PST;{subj}" if subj else None
    if "PRES" in t:
        return f"V;PRS;{subj}" if subj else None
    return None


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 3:
                continue
            lemma, form, features = (f.strip() for f in fields)
            if not form or " " in form:
                continue
            feat = canonical(features)
            if feat:
                rows.add((lemma, form, feat))
    out = sys.stdout
    for lemma, form, feat in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
