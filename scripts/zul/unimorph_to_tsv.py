#!/usr/bin/env python3
"""Convert UniMorph zul (Zulu) to the shared `lemma ⇥ form ⇥ features` TSV
in the canonical schema shared with the kaikki adapter.

Zulu is Bantu and agglutinative: the finite verb is a slot template —
subject concord + tense/aspect marker + ROOT + final vowel. UniMorph zul
tags the subject two ways: the four grammatical persons (`1;SG`, `2;SG`,
`1;PL`, `2;PL`) and the noun classes (`BANTU1`..`BANTU17`). The TAM is a
small set: present (`PRS`), recent/remote past (`RCT;PST` / `RMT;PST`),
their progressives (`…;PROG`), future (`FUT`), subjunctive (`SBJV`),
participle (`V.PTCP`), plus the infinitive (`NFIN`) and imperative
(`IMP`).

Canonicalisation (identical in the kaikki adapter, so the two oracles
align):

1. **Subject token.** `1;SG`→`1SG`, `2;PL`→`2PL`, `BANTU7`→`CL7`.
2. **TAM token.** `PRS`, `FUT`, `RCT_PST`, `RCT_PST_PROG`, `RMT_PST`,
   `RMT_PST_PROG`, `SBJV`, `PTCP`. The bundle is `V;<SUBJ>;<TAM>`;
   the subjectless slots are `V;NFIN`, `V;IMP;2SG`, `V;IMP;2PL`.
3. **Macron stripping.** UniMorph writes the long past vowel with a
   macron (`ngāfika`, `ngifikē`); kaikki writes it plain. Both adapters
   strip `ā`→`a`, `ē`→`e` so the remote past and the short recent past
   line up across the two oracles.

Usage: python3 scripts/zul/unimorph_to_tsv.py data/zul/unimorph-zul.txt
"""

import sys

MACRON = str.maketrans({"ā": "a", "ē": "e", "ī": "i", "ō": "o", "ū": "u"})


def subject(tags):
    """Canonical subject token (1SG, 2PL, CL7, …), or None."""
    t = set(tags)
    person = "1" if "1" in t else "2" if "2" in t else None
    num = "SG" if "SG" in t else "PL" if "PL" in t else None
    if person and num:
        return f"{person}{num}"
    for tag in tags:
        if tag.startswith("BANTU"):
            n = tag[len("BANTU"):]
            if n.isdigit():
                return f"CL{n}"
    return None


def canonical(features):
    """Map a UniMorph zul bundle to canonical `V;<SUBJ>;<TAM>`, or None."""
    tags = features.split(";")
    t = set(tags)
    if "V" not in t:
        return None
    if "N" in t or "ADJ" in t:
        return None
    if "NFIN" in t:
        return "V;NFIN"
    if "IMP" in t:
        # The LGSPEC3 variant (final -e imperative) is a separate, kaikki-
        # unattested slot; keep only the plain imperative for alignment.
        if "LGSPEC3" in t:
            return None
        num = "2SG" if "SG" in t else "2PL" if "PL" in t else None
        return f"V;IMP;{num}" if num else None
    subj = subject(tags)
    if subj is None:
        return None
    progressive = "PROG" in t
    if "SBJV" in t:
        tam = "SBJV"
    elif "V.PTCP" in t or "PTCP" in t:
        tam = "PTCP"
    elif "FUT" in t:
        tam = "FUT"
    elif "RCT" in t and "PST" in t:
        tam = "RCT_PST_PROG" if progressive else "RCT_PST"
    elif "RMT" in t and "PST" in t:
        tam = "RMT_PST_PROG" if progressive else "RMT_PST"
    elif "PRS" in t:
        tam = "PRS"
    else:
        return None
    return f"V;{subj};{tam}"


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
            if not feat:
                continue
            form = form.translate(MACRON)
            rows.add((lemma, form, feat))
    out = sys.stdout
    for lemma, form, feat in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
