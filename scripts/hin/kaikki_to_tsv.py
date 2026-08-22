#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Hindi verb JSONL into the shared
`lemma ⇥ form ⇥ features` TSV, using the *same* UniMorph feature bundles
scripts/hin/unimorph_to_tsv.py emits, so the harness can intersect them.

kaikki tags a Hindi paradigm differently from UniMorph in three ways
this adapter bridges:

1. **Collapsed persons.** kaikki gives one row for cells that share a
   surface form and tags every person it covers (`उठता है` is tagged
   second- *and* third-person singular; `उठते हैं` first- *and*
   third-person plural). Each person is fanned out to its own
   `(person, number)` bundle. The formal 2nd person (आप) has no
   UniMorph finite cell and is dropped; तू is 2;SG, तुम is 2;PL, to
   match UniMorph.

2. **Aspect lives in the form, not the tags.** The analytic present
   `उठता है` (habitual), `उठा है` (perfect) and `उठ रहा है`
   (progressive) carry *identical* kaikki tag sets — the aspect is only
   visible in the content word. So aspect is read off the form: a
   `रह`-token means progressive; a content word ending in the
   त-participle (ता/ते/ती/तीं) is habitual; anything else is the
   perfect. This is adapter bookkeeping, not conjugation — it only
   labels forms kaikki already spells out.

3. **Extra moods.** kaikki spells out a future-indicative analytic
   (`उठा होगा` "will have risen") and a future-subjunctive analytic
   (`उठा होए`) that UniMorph does not carry; both are dropped so the two
   oracles describe the same cell set. The presumptive (होगा) and the
   present-subjunctive analytic (हो) are kept, mapped to UniMorph's
   LGSPEC3 and SBJV.

Usage: python3 scripts/hin/kaikki_to_tsv.py data/hin/kaikki-verbs.jsonl
"""

import json
import re
import sys

# The optional euphonic य-glide (see scripts/hin/unimorph_to_tsv.py).
_VOWEL = "ािीुूेैोौआइईउऊएऐओऔ"
_INDEP = {"े": "ए", "ी": "ई"}
GLIDE = re.compile(rf"(?<=[{_VOWEL}])य([ेी])")

# Tags that mark a form as outside the shared standard paradigm: the
# citation cruft, the derived nominals/adjectivals, and the register or
# dialect layers UniMorph does not carry.
SKIP = {
    "romanization", "Urdu", "table-tags", "inflection-template", "stem",
    "conjunctive", "agentive", "adjectival", "prospective", "vocative",
    "alternative", "rare", "obsolete", "regional", "literary", "uncommon",
    "dated", "archaic", "dialectal", "poetic", "class", "canonical",
    "intransitive", "transitive", "verb",
}

PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
IPFV_ENDINGS = ("ता", "ते", "ती", "तीं")

# Every finite form of the होना auxiliary, so an analytic form can be
# told from a bare participle by whether its last token is one of these
# (a compound lemma like `अंगारा बनना` puts an invariable noun first, so
# token position alone will not do).
AUX = {
    "हूँ", "है", "हैं", "हो", "हों",          # present / present-subjunctive
    "था", "थी", "थे", "थीं",                    # past
    "हूँगा", "होगा", "होगे", "होंगे",           # presumptive (masc)
    "हूँगी", "होगी", "होंगी",                    # presumptive (fem)
    "होता", "होती", "होते", "होतीं",           # counterfactual
    "होऊँ", "होए", "होओ", "होएँ",              # future-subjunctive
}


def numbers(tags):
    # The feminine तू (2nd-person) analytic forms carry a person tag but
    # no number; तू is singular, so an untagged number defaults to SG.
    return "PL" if "plural" in tags else "SG"


def persons(tags):
    """The UniMorph persons a finite row covers. The formal 2nd person
    (आप) has no UniMorph cell and is dropped. A row with no person tag —
    the past (था) and counterfactual (होता) auxiliaries do not inflect
    for person — applies to all three persons of its number."""
    if "formal" in tags:
        return []
    ps = [PERSON[t] for t in ("first-person", "second-person", "third-person") if t in tags]
    return ps if ps else ["1", "2", "3"]


def gender(tags):
    if "masculine" in tags:
        return "MASC"
    if "feminine" in tags:
        return "FEM"
    return None


PROG_MARKERS = {"रहा", "रही", "रहे"}


def aspect(form):
    """habitual / perfective / progressive of an analytic form, read off
    the token just before the auxiliary (the participle or रहा)."""
    body = form.split(" ")[:-1]  # drop the auxiliary
    if PROG_MARKERS.intersection(body):
        return "PROG"
    if body and body[-1].endswith(IPFV_ENDINGS):
        return "HAB"
    return "PFV"


def aux_mood(tags):
    """The auxiliary tense-mood of an analytic form, or None to drop it."""
    if "counterfactual" in tags:
        return "LGSPEC4"
    if "presumptive" in tags:
        return "LGSPEC3"
    if "subjunctive" in tags:
        # Present-subjunctive copula (हो) is UniMorph's SBJV; the
        # future-subjunctive copula (होए) is not carried.
        return "SBJV" if "present" in tags else None
    if "indicative" in tags:
        if "present" in tags:
            return "PRS"
        if "past" in tags:
            return "PST"
        return None  # future-indicative analytic: not in UniMorph
    return None


def bundles(form, tags):
    """The UniMorph feature bundle(s) for one kaikki (form, tags) row."""
    t = set(tags)
    if t & SKIP:
        return []
    has_aux = form.split(" ")[-1] in AUX

    if "infinitive" in t:
        if "feminine" in t:
            return []  # gendered infinitive: no UniMorph cell
        return ["V;NFIN;LGSPEC2"] if "oblique" in t else ["V;NFIN;LGSPEC1"]

    if "imperative" in t:
        if "future" in t or "third-person" in t:
            return []  # उठियो / उठियेगा / 3rd-person imperative
        if "formal" in t:
            return ["V;2;PL;IMP;FORM"]   # आप — उठिए
        if "singular" in t:
            return ["V;2;SG;IMP;INFM"]   # तू — उठ
        if "plural" in t:
            return ["V;2;SG;IMP;FORM"]   # तुम — उठो (UniMorph's label)
        return []

    # Bare participle: an aspect tag, no finite mood, no auxiliary.
    finite_mood = t & {"indicative", "subjunctive", "presumptive", "counterfactual"}
    if not has_aux and (t & {"habitual", "perfective"}) and not finite_mood:
        g = gender(t)
        if g is None:
            return []
        asp = "IPFV" if "habitual" in t else "PFV"
        # masculine oblique-singular shares the plural form (उठते).
        num = "PL" if ("plural" in t or ("oblique" in t and g == "MASC")) else "SG"
        return [f"V;V.PTCP;{g};{num};{asp}"]

    num = numbers(t)
    ps = persons(t)
    if not ps:
        return []

    # Bare subjunctive and synthetic future (kaikki tags both "future").
    if not has_aux and "future" in t:
        if "subjunctive" in t:
            return [f"V;{p};{num};SBJV" for p in ps]
        if "indicative" in t:
            g = gender(t)
            if g is None:
                return []
            return [f"V;{p};{num};SBJV;{g}" for p in ps]
        return []

    # Analytic finite forms (participle + auxiliary).
    if has_aux:
        mood = aux_mood(t)
        g = gender(t)
        if mood is None or g is None:
            return []
        # The perfect analytic layer is UniMorph's PRF; the bare
        # perfective participle above is PFV.
        asp = {"PFV": "PRF"}.get(aspect(form), aspect(form))
        return [f"V;{p};{num};{asp};{mood};{g}" for p in ps]

    return []


def normalize_orthography(form):
    """Fold the two orthographic choices the oracles split on, identically
    to scripts/hin/unimorph_to_tsv.py: candrabindu → anusvara (कमाऊँ →
    कमाऊं) and the optional य-glide after stem-final आ before an ए/ई
    ending (कमाये → कमाए, कमायी → कमाई), and the optional feminine-plural
    nasal (उठीं → उठी). Applied to the *emitted* form only — aspect and
    auxiliary detection above run on the raw kaikki spelling (the
    auxiliary list is spelt with candrabindu)."""
    form = GLIDE.sub(lambda m: _INDEP[m.group(1)], form.replace("ँ", "ं"))
    return form.replace("ीं", "ी").replace("ईं", "ई")


def main(path):
    rows = set()
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            d = json.loads(line)
            lemma = d.get("word")
            if not lemma:
                continue
            for f in d.get("forms", []):
                form = (f.get("form") or "").strip()
                tags = f.get("tags")
                if not form or not tags:
                    continue
                for features in bundles(form, tags):
                    rows.add((normalize_orthography(lemma),
                              normalize_orthography(form), features))
    out = sys.stdout
    for lemma, form, features in sorted(rows):
        out.write(f"{lemma}\t{form}\t{features}\n")


if __name__ == "__main__":
    main(sys.argv[1])
