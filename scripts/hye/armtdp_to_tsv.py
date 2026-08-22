#!/usr/bin/env python3
"""Convert the UD Armenian-ArmTDP treebank to the shared verb TSV.

This is a *spot check*, not one of the two oracles: a treebank attests
the forms that happen to occur in its texts, so it cannot fill a
paradigm. What it can do is corroborate the core irregulars — գալ,
տալ, ուտել, անել, … — which UniMorph hye omits entirely and which are
therefore hand-supplied in scripts/hye/manual.tsv. Those verbs are the
frequent ones, so a corpus covers them well.

Only the tags that map onto the harness schema unambiguously are
emitted. Case-marked infinitives and participles (գրելը, գրողին) are
nominal inflection and are dropped, except the dative infinitive, whose
surface form is the future converb (գրելու). Voice=Pass tokens belong
to the derived -վել lexeme, not to the lemma's own paradigm, and are
dropped too.

Usage: python3 scripts/hye/armtdp_to_tsv.py data/hye/armtdp/*.conllu
"""

import sys

PERSON_NUMBER = {("Sing", "1"): "SG;1", ("Sing", "2"): "SG;2", ("Sing", "3"): "SG;3",
                 ("Plur", "1"): "PL;1", ("Plur", "2"): "PL;2", ("Plur", "3"): "PL;3"}


def features(feats, form):
    f = dict(kv.split("=", 1) for kv in feats.split("|") if "=" in kv)
    if f.get("Voice") == "Pass":
        return None
    neg = ";NEG" if f.get("Polarity") == "Neg" else ""
    verbform = f.get("VerbForm")

    if verbform == "Inf":
        case = f.get("Case")
        if case is None:
            return f"V;NFIN{neg}"
        if case == "Dat" and f.get("Definite") == "Ind" and form.endswith("ու"):
            return "V.CVB;FUT;LGSPEC02"
        return None

    if verbform == "Part":
        if f.get("Case") or f.get("Definite"):
            return None
        aspect = f.get("Aspect")
        if aspect == "Imp":
            return "V.CVB;IPFV"
        if aspect == "Dur":
            return f"V;V.PTCP;SUB{neg}"
        if aspect == "Prosp":
            return "V.CVB;FUT;LGSPEC02"
        if aspect == "Perf":
            # Perfective converb and resultative participle share the
            # tag; their suffixes do not.
            if form.endswith("ած"):
                return "V.PTCP;NEG;LGSPEC01" if neg else "V;V.PTCP;LGSPEC01"
            return f"V.CVB;PFV{neg}" if not neg else None
        return None

    if verbform != "Fin":
        return None
    pn = PERSON_NUMBER.get((f.get("Number", ""), f.get("Person", "")))
    if not pn:
        return None
    mood, tense = f.get("Mood"), f.get("Tense")
    if mood == "Imp":
        number = "SG" if f.get("Number") == "Sing" else "PL"
        return f"V;IMP;{number};2" if not neg else None
    if mood == "Ind" and tense == "Past" and f.get("Aspect") == "Perf":
        return f"V;IND;{pn}{neg};PST"
    if mood == "Sub":
        head = "V;PRF;SBJV" if tense == "Past" else "V;SBJV"
        return f"{head};{pn}{neg};FUT"
    if mood == "Cnd" and not neg:
        head = "V;PRF;COND" if tense == "Past" else "V;COND"
        return f"{head};{pn};FUT"
    return None


def main(paths):
    rows = set()
    for path in paths:
        with open(path, encoding="utf-8") as fh:
            for line in fh:
                if line.startswith("#") or not line.strip():
                    continue
                fields = line.rstrip("\n").split("\t")
                if len(fields) < 10 or "-" in fields[0] or "." in fields[0]:
                    continue
                form, lemma, upos, feats = fields[1], fields[2], fields[3], fields[5]
                if upos != "VERB" or not (lemma.endswith("ել") or lemma.endswith("ալ")):
                    continue
                bundle = features(feats, form.lower())
                if bundle:
                    rows.add((lemma, form.lower(), bundle))
    out = sys.stdout
    for lemma, form, bundle in sorted(rows):
        out.write(f"{lemma}\t{form}\t{bundle}\n")


if __name__ == "__main__":
    main(sys.argv[1:])
