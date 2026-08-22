#!/usr/bin/env python3
"""Convert UniMorph guj to the shared `lemma ⇥ form ⇥ features` TSV.

UniMorph guj (Batsuren & Cotterell; English-Wiktionary lineage) is a
generated verb paradigm: 90 verb lemmas, each cited by its `-વું`
infinitive, with person/number agreement on the present, future,
present-progressive and imperative, plus single (neuter) forms for the
past, the past-progressive, the two converbs (LGSPEC1/2), the verbal
noun (V.MSDR), and the conditional/counterfactual (LGSPEC3/4).

The file also carries nouns and adjectives (the whole `guj` UniMorph is
one file); only the `V` rows are kept.

This adapter emits the bounded cell set the engine models and both
oracles describe, dropping the layers left out of scope:

* the **negatives** (`…;NEG`): Gujarati negates analytically with the
  particle નહીં / ન before the positive form (નહીં કરું), which is
  syntax, not a verb form;
* the **passive** (`V;PASS` કરાય), **potential** (`V;POT` કરી શકવું),
  **optative** (`V;OPT`), the **present subjunctive** (`V;SBJV;PRS;POS`
  કરતું હોવું) and the **future progressive** (`V;IND;FUT;PROG;POS`),
  which the engine deliberately does not enumerate.

Usage: python3 scripts/guj/unimorph_to_tsv.py data/guj/unimorph-guj.txt
"""

import sys

# Bundles dropped whole (see the module docstring).
DROP = {
    "V;PASS",
    "V;POT",
    "V;OPT",
    "V;SBJV;PRS;POS",
    "V;IND;FUT;PROG;POS",
}


def keep(features):
    if not features.startswith("V"):
        return False
    if ";NEG" in features:
        return False
    return features not in DROP


def normalize(form):
    """Fold candrabindu to anusvara (one spelling of nasalization) and
    drop the trailing exclamation UniMorph writes on plain imperatives
    (ઇચ્છો! → ઇચ્છો) — punctuation, not part of the verb form."""
    return form.replace("ઁ", "ં").rstrip("!").strip()


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            fields = line.rstrip("\n").split("\t")
            if len(fields) != 3:
                continue
            lemma, form, features = (f.strip() for f in fields)
            if not keep(features) or not lemma or not form:
                continue
            rows.add((normalize(lemma), normalize(form), features))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
