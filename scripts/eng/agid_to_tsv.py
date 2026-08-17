#!/usr/bin/env python3
"""Convert AGID infl.txt (SCOWL/wordlist project) to the shared TSV.

V lines carry 3 fields (past | pres-ptcp | 3sg; the past doubles as
the participle) or 4 (past | past-ptcp | pres-ptcp | 3sg). Variant
annotations (?, <, ~, !, cross-reference digits) are stripped; every
listed variant is kept.
"""
import re
import sys


def clean(field):
    out = []
    for v in field.split(","):
        # '!' marks attested misspellings (stoped), '<' cross-links a
        # nonstandard base — both are noise, not variants.
        if "!" in v or "<" in v:
            continue
        v = re.sub(r"[?~]|\{.*?\}|\d+", "", v).strip()
        if v and " " not in v:
            out.append(v)
    return out


def main(path):
    for line in open(path, encoding="utf-8", errors="replace"):
        m = re.match(r"^(\S+) V\??: (.+)$", line.strip())
        if not m:
            continue
        lemma, rest = m.groups()
        if not lemma.isascii():
            continue
        fields = [clean(f) for f in rest.split(" | ")]
        if len(fields) == 3:
            past, presp, third = fields
            pastp = past
        elif len(fields) == 4:
            past, pastp, presp, third = fields
        else:
            continue
        print(f"{lemma}\t{lemma}\tV;NFIN")
        for forms, feat in [(past, "V;PST"), (pastp, "V.PTCP;PST"),
                            (presp, "V.PTCP;PRS"), (third, "V;PRS;3;SG")]:
            for f in forms:
                print(f"{lemma}\t{f}\t{feat}")


if __name__ == "__main__":
    main(sys.argv[1])
