#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Esperanto verb extraction to the
shared `lemma ⇥ form ⇥ features` TSV.

Esperanto is perfectly regular: every verb is cited by its `-i`
infinitive and conjugates by pure suffixation off the invariant stem
(infinitive minus `-i`). kaikki keys each entry on that infinitive and
lists the full `eo-conj` table. Only the entries that actually carry the
conjugation template are emitted (2894 of them; the rest are non-verb
homographs Wiktextract mislabels `pos: verb` — borrowings, Latin/Malay
look-alikes — that have no inflected forms and so contribute no rows).

The kaikki tag sets are mapped onto canonical bundles the golden harness
also generates:

  present / past / future / conditional / volitive / infinitive
     → V;PRS / V;PST / V;FUT / V;COND / V;VOL / V;NFIN

  participle × voice(active|passive) × tense(present|past|future)
             × form(adjective|noun-from-verb|adverbial)
             × [adjective/noun: number(sg|pl) × case(nom|acc)]
     → V.PTCP;<ACT|PASS>;<PRS|PST|FUT>;<ADJ|N>;<SG|PL>;<NOM|ACC>
       V.PTCP;<ACT|PASS>;<PRS|PST|FUT>;ADV

Usage: python3 scripts/epo/kaikki_to_tsv.py data/epo/kaikki-verbs.jsonl
"""

import json
import sys


def canonical(tags):
    """Map a kaikki tag set to a canonical bundle, or None to skip."""
    t = set(tags)
    if t & {"table-tags", "inflection-template", "alternative"}:
        return None
    if "participle" in t:
        voice = "ACT" if "active" in t else "PASS" if "passive" in t else None
        if voice is None:
            return None
        tense = "PST" if "past" in t else "FUT" if "future" in t else "PRS"
        if "adverbial" in t:
            return f"V.PTCP;{voice};{tense};ADV"
        form = "N" if "noun-from-verb" in t else "ADJ"
        num = "PL" if "plural" in t else "SG"
        case = "ACC" if "accusative" in t else "NOM"
        return f"V.PTCP;{voice};{tense};{form};{num};{case}"
    # Finite / non-finite. The noisy secondary table double-tags some
    # cells (infinitive+plural, volitive as imperative+past); the
    # priority order below collapses them onto the right bundle, and
    # because they carry the same surface form it is harmless anyway.
    if "infinitive" in t:
        return "V;NFIN"
    if "volitive" in t or "imperative" in t:
        return "V;VOL"
    if "conditional" in t:
        return "V;COND"
    if "future" in t:
        return "V;FUT"
    if "past" in t:
        return "V;PST"
    if "present" in t:
        return "V;PRS"
    return None


def main(path):
    rows = set()
    for line in open(path, encoding="utf-8"):
        d = json.loads(line)
        lemma = d.get("word", "")
        if not lemma or " " in lemma or not lemma.endswith("i"):
            continue
        for fm in d.get("forms", []):
            form = fm.get("form", "")
            if not form or " " in form or form.startswith("-"):
                continue
            feat = canonical(fm.get("tags", []))
            if feat:
                rows.add((lemma, form, feat))
    out = sys.stdout
    for lemma, form, feat in sorted(rows):
        out.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
