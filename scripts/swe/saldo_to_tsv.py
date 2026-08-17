#!/usr/bin/env python3
"""Convert SALDO's morphology (saldom.xml, LMF) into the
lemma<TAB>form<TAB>features TSV the Swedish golden harness reads.

Usage: python3 scripts/swe/saldo_to_tsv.py data/swe/saldom.xml \
           > data/swe/saldo.tsv

Swedish conjugates without person/number; the paradigm is
infinitive / present / past / supine / imperative, each with an
s-passive, plus the participles and the relic subjunctives.
"""
import re
import sys

MSD = {
    "inf aktiv": "V;NFIN;ACT",
    "inf s-form": "V;NFIN;PASS",
    "pres ind aktiv": "V;IND;PRS;ACT",
    "pres ind s-form": "V;IND;PRS;PASS",
    "pret ind aktiv": "V;IND;PST;ACT",
    "pret ind s-form": "V;IND;PST;PASS",
    "sup aktiv": "V;SUP;ACT",
    "sup s-form": "V;SUP;PASS",
    "imper": "V;IMP",
    "pres konj aktiv": "V;SBJV;PRS;ACT",
    "pres konj s-form": "V;SBJV;PRS;PASS",
    "pret konj aktiv": "V;SBJV;PST;ACT",
    "pret konj s-form": "V;SBJV;PST;PASS",
}

ENTRY = re.compile(r"<LexicalEntry>(.*?)</LexicalEntry>", re.S)
FEAT = re.compile(r'<feat att="(\w+)" val="([^"]*)"/>')


def main(path):
    seen = set()
    buf = []
    inside = False
    for line in open(path, encoding="utf-8"):
        if "<LexicalEntry>" in line:
            inside = True
            buf = []
        if inside:
            buf.append(line)
        if "</LexicalEntry>" in line and inside:
            inside = False
            entry = "".join(buf)
            feats = FEAT.findall(entry)
            lemma, pos = None, None
            for att, val in feats:
                if att == "writtenForm" and lemma is None:
                    lemma = val
                if att == "partOfSpeech":
                    pos = val
            if pos != "vb" or not lemma or " " in lemma:
                continue
            form = None
            for att, val in feats:
                if att == "writtenForm":
                    form = val
                elif att == "msd":
                    feat = MSD.get(val)
                    if feat and form and " " not in form:
                        row = (lemma, form, feat)
                        if row not in seen:
                            seen.add(row)
                            sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
