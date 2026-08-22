#!/usr/bin/env python3
"""Convert UniMorph urd to the shared `lemma ⇥ form ⇥ features` TSV.

UniMorph urd (added 2017, English-Wiktionary lineage — the same source
kaikki draws on, so it is *not* an independent agreement partner; see
docs/urd/oracles.md) is a fully-generated paradigm whose feature bundles
already *are* the schema the harness uses. It is emitted here as a
documented spot-check oracle, not the gate.

Two normalizations:

1. **Script.** Forms are folded with the Perso-Arabic rules of norm.py
   (the twin of src/perso_arabic.rs) so they compare byte-for-byte with
   the engine and kaikki. UniMorph urd is already unvocalized, so this is
   mostly the Arabic→Perso-Arabic letter folding.

2. **Subject pronouns.** The bare subjunctive and synthetic future are
   cited with their subject pronoun (میں اتروں, تو اترے گا, and the
   third-person یہ وہ اتریں). The pronoun is syntax, not part of the verb
   form, so every leading pronoun token is stripped.

Usage: python3 scripts/urd/unimorph_to_tsv.py data/urd/unimorph-urd.txt
"""

import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from norm import normalize  # noqa: E402

# The subject pronouns UniMorph prefixes to the bare subjunctive and the
# synthetic future (third person cites both یہ and وہ).
PRONOUNS = {"میں", "تو", "تم", "یہ", " یے", "وہ", "ہم", "آپ", "ہم", "یے"}


def strip_pronouns(form):
    """Drop every leading subject pronoun token (`میں اتروں` → `اتروں`,
    `یہ وہ اتریں` → `اتریں`)."""
    toks = form.split()
    while toks and toks[0] in PRONOUNS:
        toks = toks[1:]
    return " ".join(toks)


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 3:
                continue
            lemma, form, features = (f.strip() for f in fields)
            if not features.startswith("V"):
                continue
            form = normalize(strip_pronouns(form))
            if not form:
                continue
            rows.add((normalize(lemma), form, features))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
