#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Urdu verb JSONL into the shared
`lemma ⇥ form ⇥ features` TSV, using the *same* UniMorph-style feature
bundles the apertium adapter (scripts/urd/apertium_to_tsv.py) emits, so
the harness can intersect them. The bundle schema is the one UniMorph urd
already uses (and the Hindi harness shares) — Urdu *is* Hindustani in the
Perso-Arabic script, so the morphological description is identical.

kaikki tags an Urdu paradigm with the `ur-conj` template, which — unlike
its Hindi sibling `hi-conj` — emits **no person tags at all**: the
subjunctive, the synthetic future and the whole analytic (participle +
copula) layer are printed without any first/second/third-person
distinction (and the bare subjunctive is even tagged
`error-unrecognized-form`). Person-resolved cells therefore cannot be
recovered from kaikki, so this adapter emits only the slots kaikki *does*
tag unambiguously — the **person-independent** core:

  * the infinitive and oblique infinitive,
  * the three imperatives (تو / تم / آپ), keyed by singular/plural/formal,
  * the imperfective and perfective participles, keyed by gender/number.

That core is exactly the surface kaikki shares with the independent
apertium-urd oracle (the two-oracle gate); the person-resolved finite
paradigm is verified against apertium and UniMorph separately — see
docs/urd/oracles.md.

Every form is normalized with the Perso-Arabic rules of norm.py (the twin
of src/perso_arabic.rs): short-vowel diacritics stripped, ZWNJ/directional
marks dropped, Arabic letters folded. That collapses kaikki's fully
vocalized spelling (اُتَرْنا) onto the bare orthography the engine and the
other oracle use (اترنا).

Usage: python3 scripts/urd/kaikki_to_tsv.py data/urd/kaikki-verbs.jsonl
"""

import json
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
from norm import normalize  # noqa: E402

# Tags that mark a form as outside the shared standard paradigm: the
# citation cruft, the derived nominals/adjectivals, the transliteration,
# the Devanagari (Hindi) doublet kaikki prints alongside, and the
# register/dialect layers.
SKIP = {
    "romanization", "Hindi", "table-tags", "inflection-template", "stem",
    "conjunctive", "agentive", "adjectival", "prospective", "vocative",
    "alternative", "rare", "obsolete", "regional", "literary", "uncommon",
    "dated", "archaic", "dialectal", "poetic", "class", "canonical",
    "intransitive", "transitive", "verb", "error-unrecognized-form",
}


def gender(tags):
    if "masculine" in tags:
        return "MASC"
    if "feminine" in tags:
        return "FEM"
    return None


def fold_fem_plural_nasal(form):
    """Drop the optional feminine-plural nasal (گرجیں → گرجی, گرجتیں →
    گرجتی): the ں that marks feminine-plural agreement on a participle is
    written by kaikki and dropped by apertium; both are standard. Folding
    it lets the single feminine participle cell — which the engine, like
    Hindi, does not split by number — be scored. Safe here because kaikki
    emits no other word-final یں (it carries no person-resolved
    subjunctive)."""
    return form[:-1] if form.endswith("یں") else form


def bundles(form, tags):
    """The feature bundle(s) for one kaikki (form, tags) row — restricted
    to the person-independent slots kaikki tags reliably (see module
    docstring). Multi-word (analytic) and person-resolved rows are
    dropped."""
    t = set(tags)
    if t & SKIP or " " in form.strip():
        return []

    if "infinitive" in t:
        if "oblique" in t:
            return ["V;NFIN;LGSPEC2"]
        # The citation infinitive is the masculine singular (اترنا); the
        # masculine-plural verbal noun (اترنے) coincides with the oblique
        # and must not be relabelled as the citation form.
        if "feminine" in t or "plural" in t:
            return []
        return ["V;NFIN;LGSPEC1"]

    if "imperative" in t:
        if "future" in t or "third-person" in t:
            return []  # اترنا (future imperative) / 3rd-person imperative
        if "formal" in t:
            return ["V;2;PL;IMP;FORM"]   # آپ — اترئیے
        if "singular" in t:
            return ["V;2;SG;IMP;INFM"]   # تو — اتر
        if "plural" in t:
            return ["V;2;SG;IMP;FORM"]   # تم — اترو
        return []

    # Bare participle: an aspect tag, gender, no finite mood.
    finite_mood = t & {"indicative", "subjunctive", "presumptive", "counterfactual",
                       "future", "present", "past"}
    if (t & {"habitual", "perfective"}) and not finite_mood:
        g = gender(t)
        if g is None:
            return []
        asp = "IPFV" if "habitual" in t else "PFV"
        # masculine oblique-singular shares the plural form (اترتے/اترے).
        num = "PL" if ("plural" in t or ("oblique" in t and g == "MASC")) else "SG"
        return [f"V;V.PTCP;{g};{num};{asp}"]

    return []


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            d = json.loads(line)
            lemma = d.get("word")
            if not lemma:
                continue
            for f in d.get("forms", []):
                form = normalize((f.get("form") or "").strip())
                tags = f.get("tags")
                if not form or not tags:
                    continue
                for features in bundles(form, tags):
                    rows.add((normalize(lemma), fold_fem_plural_nasal(form), features))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
