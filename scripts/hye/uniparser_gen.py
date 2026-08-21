#!/usr/bin/env python3
"""Generate the uniparser Eastern Armenian oracle.

`uniparser-eastern-armenian` (Timofey Arkhangelskiy) is a rule-based
morphological *analyzer* built from a formalized description of literary
Eastern Armenian.  It is also a generator: every lexeme carries stems and
a paradigm name, and the paradigms are plain suffix chains, so the whole
inventory can be enumerated.

The library stops expanding a chain at a `<.>` continuation marker (it
compiles paradigms lazily for parsing), so this script walks the
`subsequent` links itself: `գր<.>` + the `aor-ec` paradigm gives
`գրեցի…`, `կգր<.>` + `sbjv-e` gives `կգրեմ…`.  Continuations into the
*nominal* paradigms (the case-inflected infinitive `գրելը`, declined
participles) are not followed — they are out of the verb schema.

Output: `lemma ⇥ form ⇥ features` with UniMorph-style feature bundles,
the same schema as `scripts/hye/unimorph_to_tsv.py`.
"""

import sys

from uniparser_eastern_armenian import EasternArmenianAnalyzer
from uniparser_morph.wordform import Wordform

# Continuation paradigms that are nominal (case, number, possessive):
# the infinitive and the participles decline like nouns, which is the
# noun engine's business, not the verb schema's.
NOMINAL_PREFIXES = ("N", "A1", "case-", "poss", "pl-", "nmlz", "apl")

# Tags that mark a form as belonging to another lexeme (the passive and
# causative stems have their own -ել infinitives, and UniMorph lists them
# as separate lemmas), a pre-reform spelling, or a colloquial variant.
SKIP_TAGS = {"pass", "caus", "old", "oral", "dial"}

# Lexeme-level tags that ride along on every form and say nothing about
# the slot.
CLASS_TAGS = {"V", "tr", "intr", "med", "intr/tr"}

PERSON_NUMBER = {
    ("sg", "1"): "SG;1",
    ("sg", "2"): "SG;2",
    ("sg", "3"): "SG;3",
    ("pl", "1"): "PL;1",
    ("pl", "2"): "PL;2",
    ("pl", "3"): "PL;3",
}

# Non-finite slots: uniparser tag set -> UniMorph bundle. The LGSPEC
# codes are UniMorph hye's own (see its README): LGSPEC01 resultative,
# LGSPEC02 future converb 1, LGSPEC04 connegative.
NONFINITE = {
    ("inf",): "V;NFIN",
    ("cvb", "ipfv"): "V.CVB;IPFV",
    ("cvb", "pfv"): "V.CVB;PFV",
    ("cvb", "dest"): "V.CVB;FUT;LGSPEC02",
    ("cvb", "sim"): "V.CVB;SIM",
    ("cvb", "conneg"): "V.CVB;LGSPEC04",
    ("ptcp", "sbj"): "V;V.PTCP;SUB",
    ("ptcp", "res"): "V;V.PTCP;LGSPEC01",
}

NONFINITE_NEG = {
    ("inf",): "V;NFIN;NEG",
    ("ptcp", "sbj"): "V;V.PTCP;SUB;NEG",
    ("ptcp", "res"): "V.PTCP;NEG;LGSPEC01",
}


def features(gramm, lex_gramm):
    """Map a uniparser tag string to a UniMorph bundle, or None.

    `lex_gramm` is the lexeme's own tag string. Its tags ride along on
    every form, so `caus` in a form's tags means a derived causative
    only when the lexeme itself is not already a causative verb
    (ամուսնացնել is lexically V,caus,tr — all of its forms are its own).
    """
    lexical = set(lex_gramm.split(","))
    tags = set(gramm.split(","))
    derived = tags - lexical
    # The passive and causative stems are separate lemmas on the
    # Wiktionary side (գրել → գրվել, գրեցնել); only their infinitives
    # are in schema, as the derivation slots V;PASS and V;CAUS.
    if "inf" in tags and not derived & {"old", "oral", "neg"}:
        if "caus" in derived and "pass" not in derived:
            return "V;CAUS"
        if "pass" in derived and "caus" not in derived:
            return "V;PASS"
    if derived & SKIP_TAGS:
        return None
    tags -= lexical - {"V"}
    neg = "neg" in tags
    pn = None
    for (num, person), um in PERSON_NUMBER.items():
        if num in tags and person in tags:
            pn = um
            break

    if "aor" in tags and pn:
        return f"V;IND;{pn};NEG;PST" if neg else f"V;IND;{pn};PST"
    if "sbjv" in tags and pn:
        # The subjunctive present is the future proper (գրեմ), the
        # subjunctive past the "future in the past" (գրեի); UniMorph
        # tags the latter PRF;SBJV.
        prefix = "V;PRF;SBJV" if "pst" in tags else "V;SBJV"
        return f"{prefix};{pn};NEG;FUT" if neg else f"{prefix};{pn};FUT"
    if "cond" in tags and pn:
        if neg:
            # The negative conditional is analytic (չեմ գրի), formed on
            # the connegative — no synthetic կ- form exists.
            return None
        prefix = "V;PRF;COND" if "pst" in tags else "V;COND"
        return f"{prefix};{pn};FUT"
    if "imp" in tags and "2" in tags:
        if neg:
            # Negative imperatives are analytic (մի՛ գրիր).
            return None
        return "V;IMP;SG;2" if "sg" in tags else "V;IMP;PL;2"

    key = tuple(
        t
        for t in ("inf", "cvb", "ptcp", "ipfv", "pfv", "dest", "sim", "conneg", "sbj", "res")
        if t in tags
    )
    # A declined participle or nominalized infinitive (գրողներ, գրելը)
    # carries case/number tags on top of the verbal ones: out of schema.
    if tags - CLASS_TAGS - {"neg"} != set(key):
        return None
    table = NONFINITE_NEG if neg else NONFINITE
    return table.get(key)


def is_pre_reform(form):
    """True for a form in the pre-1922 orthography.

    The grammar carries both spellings for some slots (`եղաւ` beside
    `եղավ`). In the reformed orthography ւ survives only inside ու and
    the ligature և, so a leftover ւ marks the old spelling.
    """
    return "ւ" in form.replace("ու", "").replace("և", "")


def suffix_of(flex):
    """The material a continuation inflexion appends to its base."""
    return "".join(p.flex for p in flex.flexParts[0] if p.flex not in (".", "<.>", "[.]"))


def expand(grammar, base, gramm, flex, depth=0):
    """Yield (form, tags), following `<.>` continuations."""
    if "<.>" not in base:
        yield base, gramm
        return
    if depth > 3:
        return
    for link in flex.subsequent:
        name = link.name
        if name.startswith(NOMINAL_PREFIXES) or name not in grammar.paradigms:
            # The nominative singular of a declined participle or
            # infinitive is the bare form (գրող, գրած, գրել), which is
            # the verbal slot we want; the rest of the noun paradigm is
            # not.
            yield base.replace("<.>", ""), gramm
            continue
        for sub in grammar.paradigms[name].flex:
            form = base.replace("<.>", suffix_of(sub))
            tags = gramm + "," + sub.gramm if sub.gramm else gramm
            yield from expand(grammar, form, tags, sub, depth + 1)


def main():
    analyzer = EasternArmenianAnalyzer()
    grammar = analyzer.g
    rows = set()
    for lex in grammar.lexemes:
        if not lex.gramm.startswith("V"):
            continue
        lemma = lex.lemma
        if not lemma:
            continue
        for sublex in lex.subLexemes:
            if sublex.paradigm not in grammar.paradigms:
                continue
            for flex in grammar.paradigms[sublex.paradigm].flex:
                wordform = Wordform(grammar, sublex, flex)
                if not wordform.wf:
                    continue
                for form, tags in expand(grammar, wordform.wf, wordform.gramm, flex):
                    if "<.>" in form or "." in form or is_pre_reform(form):
                        continue
                    bundle = features(tags, lex.gramm)
                    if bundle:
                        rows.add((lemma, form, bundle))
    out = sys.stdout
    for lemma, form, bundle in sorted(rows):
        out.write(f"{lemma}\t{form}\t{bundle}\n")


if __name__ == "__main__":
    main()
