#!/usr/bin/env python3
"""Convert UniMorph hin to the shared `lemma ⇥ form ⇥ features` TSV.

UniMorph hin (Batsuren & Cotterell; English-Wiktionary lineage) is a
fully-generated paradigm: 258 verb lemmas × 211 feature bundles. Every
lemma carries the same slots — the synthetic core (infinitive, oblique
infinitive, subjunctive, synthetic future, three imperatives, the
imperfective and perfective participles) and the analytic layer
(participle + होना auxiliary: habitual/perfective/progressive × five
auxiliary tense-moods × gender × person-number).

Its feature bundles already *are* the schema the harness uses, so the
only work here is one normalization: the bare subjunctive and synthetic
future are cited with their subject pronoun (`मैं उठूँ`, `तू उठेगा`).
The pronoun is syntax, not part of the verb form, and the other oracle
(kaikki) cites these forms bare, so the leading pronoun is stripped.

Usage: python3 scripts/hin/unimorph_to_tsv.py data/hin/unimorph-hin.txt
"""

import re
import sys

# The optional euphonic य-glide: a य between a vowel and a following
# ए/ई ending (कमाये → कमाए, उठिये → उठिए, गये → गए). One oracle writes
# it, the other does not; both are standard. Dropping the glide turns
# the following dependent vowel sign into its independent letter (य + े
# → ए, य + ी → ई), since two vowel signs cannot sit on one consonant.
_VOWEL = "ािीुूेैोौआइईउऊएऐओऔ"
_INDEP = {"े": "ए", "ी": "ई"}
GLIDE = re.compile(rf"(?<=[{_VOWEL}])य([ेी])")

# The subject pronouns UniMorph prefixes to the bare subjunctive and the
# synthetic future. Everything after the first space is the verb form.
PRONOUNS = {"मैं", "तू", "तुम", "यह", "ये", "वह", "वे", "हम", "आप"}


def normalize_orthography(form):
    """Fold the two orthographic choices the two oracles split on, so a
    genuinely identical form is spelt the same in both.

    1. **Candrabindu → anusvara** (कमाऊँ → कमाऊं): nasalization on a
       vowel is written either way in modern Hindi; the two oracles pick
       differently. Anusvara is the productive choice.
    2. **Optional य-glide** between a vowel and a following ए/ई ending
       (कमाये → कमाए, कमायी → कमाई, उठिये → उठिए): the glide is optional
       orthography; one oracle always writes it, the other never. It is
       dropped here (the masculine-singular ...ाया, on which the oracles
       agree, is left alone — its glide precedes आ, not ए/ई).
    3. **Optional feminine-plural nasal** (उठीं → उठी, होतीं → होती): the
       anusvara that marks feminine-plural agreement is written by one
       oracle and dropped by the other; both are standard. Folding it
       lets the feminine-plural cells be scored instead of excluded.
    """
    form = GLIDE.sub(lambda m: _INDEP[m.group(1)], form.replace("ँ", "ं"))
    return form.replace("ीं", "ी").replace("ईं", "ई")


def strip_pronoun(form):
    """Drop a leading subject pronoun (`मैं उठूँ` → `उठूँ`)."""
    head, sep, rest = form.partition(" ")
    if sep and head in PRONOUNS:
        return rest
    return form


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
            form = normalize_orthography(strip_pronoun(form))
            if not form:
                continue
            rows.add((normalize_orthography(lemma), form, features))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
