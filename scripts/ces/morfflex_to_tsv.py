#!/usr/bin/env python3
"""Convert MorfFlex CZ (LINDAT; lemma \t PDT-tag \t form) to the
shared TSV. Prague positional tags: 1 POS, 2 SUBPOS, 3 GENDER,
4 NUMBER, 8 PERSON, 9 TENSE, 11 NEGATION, 12 VOICE, 15 VARIANT.

Only affirmative rows are kept (negation is a prefix, not a slot);
every spelling variant goes into the union.
"""
import re
import sys

# PDT gender char → our gender codes, by number.
GENDER = {
    "M": ["MA"], "I": ["MI"], "F": ["F"], "N": ["N"],
    "Y": ["MA", "MI"], "H": ["F", "N"], "Z": ["MA", "MI", "N"],
    "X": ["MA", "MI", "F", "N"],
}
GENDER_BY_NUM = {
    ("Q", "S"): ["F"], ("Q", "P"): ["N"],
    ("T", "P"): ["MI", "F"], ("T", "S"): [],
}


def genders(g, num):
    if (g, num) in GENDER_BY_NUM:
        return GENDER_BY_NUM[(g, num)]
    return GENDER.get(g, [])


def main(path):
    # The dump is huge; stream and filter to verbs of clean lemmas.
    strip = re.compile(r"[-`_].*$")
    for line in sys.stdin if path == "-" else open(path, encoding="utf-8"):
        parts = line.rstrip("\n").split("\t")
        if len(parts) != 3:
            continue
        lemma_raw, tag, form = parts
        if not tag.startswith("V") or tag[10] == "N":
            continue
        # pos14 's' fuses the -s clitic (abdikovalas); pos15 6/7 are
        # colloquial spellings — neither is a paradigm slot.
        if tag[13] != "-" or tag[14] in "67":
            continue
        lemma = strip.sub("", lemma_raw)
        if not lemma or " " in lemma or " " in form or not form:
            continue
        sub, g, num, person = tag[1], tag[2], tag[3], tag[7]
        feats = []
        if sub == "f":
            feats = ["V;NFIN"]
        elif sub == "B" and num in "SP" and person in "123":
            n = "SG" if num == "S" else "PL"
            feats = [f"V;IND;PRS;{n};{person}"]
        elif sub == "i" and num in "SP" and person in "123":
            n = "SG" if num == "S" else "PL"
            feats = [f"V;IMP;{n};{person}"]
        elif sub in "ps" and num in "SPW":
            kind = "PST" if sub == "p" else "PASS"
            if num == "W":
                # Q/W rows cover feminine singular and neuter plural
                # with one form (abdikovala).
                feats = [f"V.PTCP;{kind};F;SG", f"V.PTCP;{kind};N;PL"]
            else:
                n = "SG" if num == "S" else "PL"
                feats = [f"V.PTCP;{kind};{gg};{n}" for gg in genders(g, num)]
        elif sub in "em":
            tense = "PRS" if sub == "e" else "PST"
            if num == "P":
                feats = [f"V;CVB;{tense};PL"]
            elif g in ("M", "Y", "Z"):
                feats = [f"V;CVB;{tense};M"]
            elif g in ("F", "N", "H", "Q"):
                feats = [f"V;CVB;{tense};FN"]
        for feat in feats:
            print(f"{lemma}\t{form}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "-")
