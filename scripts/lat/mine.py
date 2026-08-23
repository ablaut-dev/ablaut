#!/usr/bin/env python3
"""Latin productive present-system rules + mined exceptions.

Latin is fusional with four conjugations. From the 1sg present active
indicative (the citation, macron-bearing) we read a conjugation class and a
stem, and generate the whole present system by rule:

  * present, imperfect and simple-future indicative (person/number),
  * the present imperative (2sg/2pl),
  * the present active infinitive,

for both the active paradigm and — with passive endings — the deponent one
(a verb is deponent iff its citation ends in -r). The class cannot be read
off the 1sg unambiguously (amō 1st vs regō 3rd both end -ō), so we take the
statistically dominant reading of each ending as the productive default and
MINE every verb the default gets wrong into data/lat/parts.tsv. The engine
in src/lat.rs mirrors produce() exactly and overrides it with those rows, so
the golden passes on the two-oracle agreement.

Usage: mine.py data/lat/unimorph.tsv data/lat/kaikki.tsv > data/lat/parts.tsv
"""
import sys
from collections import defaultdict

CELLS = [
    "V;IND;ACT;PRS;1;SG", "V;IND;ACT;PRS;2;SG", "V;IND;ACT;PRS;3;SG",
    "V;IND;ACT;PRS;1;PL", "V;IND;ACT;PRS;2;PL", "V;IND;ACT;PRS;3;PL",
    "V;IND;ACT;PST;IPFV;1;SG", "V;IND;ACT;PST;IPFV;2;SG",
    "V;IND;ACT;PST;IPFV;3;SG", "V;IND;ACT;PST;IPFV;1;PL",
    "V;IND;ACT;PST;IPFV;2;PL", "V;IND;ACT;PST;IPFV;3;PL",
    "V;IND;ACT;FUT;1;SG", "V;IND;ACT;FUT;2;SG", "V;IND;ACT;FUT;3;SG",
    "V;IND;ACT;FUT;1;PL", "V;IND;ACT;FUT;2;PL", "V;IND;ACT;FUT;3;PL",
    "V;IMP;ACT;PRS;2;SG", "V;IMP;ACT;PRS;2;PL",
    "V;NFIN;ACT;PRS",
]

PN = ["1;SG", "2;SG", "3;SG", "1;PL", "2;PL", "3;PL"]

# Per (class, voice) the six PRS / IPFV / FUT endings, the two imperative
# endings and the infinitive ending. Deponents take passive morphology.
# Keys: "1","2","3","3io","4"; each maps voice -> dict.
ACT = {
    "1": dict(prs=["ō", "ās", "at", "āmus", "ātis", "ant"],
              ipfv=["ābam", "ābās", "ābat", "ābāmus", "ābātis", "ābant"],
              fut=["ābō", "ābis", "ābit", "ābimus", "ābitis", "ābunt"],
              imp=["ā", "āte"], inf="āre"),
    "2": dict(prs=["eō", "ēs", "et", "ēmus", "ētis", "ent"],
              ipfv=["ēbam", "ēbās", "ēbat", "ēbāmus", "ēbātis", "ēbant"],
              fut=["ēbō", "ēbis", "ēbit", "ēbimus", "ēbitis", "ēbunt"],
              imp=["ē", "ēte"], inf="ēre"),
    "3": dict(prs=["ō", "is", "it", "imus", "itis", "unt"],
              ipfv=["ēbam", "ēbās", "ēbat", "ēbāmus", "ēbātis", "ēbant"],
              fut=["am", "ēs", "et", "ēmus", "ētis", "ent"],
              imp=["e", "ite"], inf="ere"),
    "3io": dict(prs=["iō", "is", "it", "imus", "itis", "iunt"],
                ipfv=["iēbam", "iēbās", "iēbat", "iēbāmus", "iēbātis", "iēbant"],
                fut=["iam", "iēs", "iet", "iēmus", "iētis", "ient"],
                imp=["e", "ite"], inf="ere"),
    "4": dict(prs=["iō", "īs", "it", "īmus", "ītis", "iunt"],
              ipfv=["iēbam", "iēbās", "iēbat", "iēbāmus", "iēbātis", "iēbant"],
              fut=["iam", "iēs", "iet", "iēmus", "iētis", "ient"],
              imp=["ī", "īte"], inf="īre"),
}
DEP = {
    "1": dict(prs=["or", "āris", "ātur", "āmur", "āminī", "antur"],
              ipfv=["ābar", "ābāris", "ābātur", "ābāmur", "ābāminī", "ābantur"],
              fut=["ābor", "āberis", "ābitur", "ābimur", "ābiminī", "ābuntur"],
              imp=["āre", "āminī"], inf="ārī"),
    "2": dict(prs=["eor", "ēris", "ētur", "ēmur", "ēminī", "entur"],
              ipfv=["ēbar", "ēbāris", "ēbātur", "ēbāmur", "ēbāminī", "ēbantur"],
              fut=["ēbor", "ēberis", "ēbitur", "ēbimur", "ēbiminī", "ēbuntur"],
              imp=["ēre", "ēminī"], inf="ērī"),
    "3": dict(prs=["or", "eris", "itur", "imur", "iminī", "untur"],
              ipfv=["ēbar", "ēbāris", "ēbātur", "ēbāmur", "ēbāminī", "ēbantur"],
              fut=["ar", "ēris", "ētur", "ēmur", "ēminī", "entur"],
              imp=["ere", "iminī"], inf="ī"),
    "3io": dict(prs=["ior", "eris", "itur", "imur", "iminī", "iuntur"],
                ipfv=["iēbar", "iēbāris", "iēbātur", "iēbāmur", "iēbāminī", "iēbantur"],
                fut=["iar", "iēris", "iētur", "iēmur", "iēminī", "ientur"],
                imp=["ere", "iminī"], inf="ī"),
    "4": dict(prs=["ior", "īris", "ītur", "īmur", "īminī", "iuntur"],
              ipfv=["iēbar", "iēbāris", "iēbātur", "iēbāmur", "iēbāminī", "iēbantur"],
              fut=["iar", "iēris", "iētur", "iēmur", "iēminī", "ientur"],
              imp=["īre", "īminī"], inf="īrī"),
}


def classify(cit):
    """(stem, class, deponent) with the dominant default per ending, or None."""
    if cit.endswith("eor"):
        return cit[:-3], "2", True
    if cit.endswith("ior"):
        return cit[:-3], "4", True     # largior-type dominates -ior deponents
    if cit.endswith("or"):
        return cit[:-2], "1", True     # hortor-type dominates -or deponents
    if cit.endswith("scō"):
        return cit[:-1], "3", False    # inchoatives in -scō are always 3rd
    if cit.endswith("eō"):
        return cit[:-2], "2", False
    if cit.endswith("iō"):
        return cit[:-2], "4", False    # audiō-type dominates -iō actives
    if cit.endswith("ō"):
        return cit[:-1], "1", False    # amō-type (1st conj) dominates -ō actives
    return None


def produce(cit, feat):
    c = classify(cit)
    if c is None:
        return None
    stem, cls, dep = c
    tab = (DEP if dep else ACT)[cls]
    if feat == "V;NFIN;ACT;PRS":
        return stem + tab["inf"]
    if feat == "V;IMP;ACT;PRS;2;SG":
        return stem + tab["imp"][0]
    if feat == "V;IMP;ACT;PRS;2;PL":
        return stem + tab["imp"][1]
    parts = feat.split(";")
    pn = parts[-2] + ";" + parts[-1]
    if pn not in PN:
        return None
    i = PN.index(pn)
    if feat.startswith("V;IND;ACT;PRS;"):
        return stem + tab["prs"][i]
    if feat.startswith("V;IND;ACT;PST;IPFV;"):
        return stem + tab["ipfv"][i]
    if feat.startswith("V;IND;ACT;FUT;"):
        return stem + tab["fut"][i]
    return None


def main(uni_path, kai_path):
    def read(path):
        d = defaultdict(set)
        for l in open(path):
            a = l.rstrip("\n").split("\t")
            if len(a) >= 3 and a[2].startswith("V"):
                d[(a[0], a[2])].add(a[1])
        return d
    uni, kai = read(uni_path), read(kai_path)

    def chosen(lemma, feat):
        a, b = uni.get((lemma, feat), set()), kai.get((lemma, feat), set())
        pool = (a & b) or a or b
        return sorted(pool)[0] if pool else None

    lemmas = sorted({l for (l, f) in list(uni) + list(kai)})

    total = hit = 0
    parts = {}
    for lemma in lemmas:
        if not lemma or " " in lemma:
            continue
        row = {}
        for c in CELLS:
            f = chosen(lemma, c)
            if f is None:
                row[c] = "-"
                continue
            pred = produce(lemma, c)
            if pred is not None:
                total += 1
                if pred == f:
                    hit += 1
            row[c] = f if f != pred else "-"
        if any(v != "-" for v in row.values()):
            parts[lemma] = row

    acc = 100 * hit / max(total, 1)
    sys.stderr.write(
        f"rule accuracy: {hit}/{total} = {acc:.2f}%, mined lemmas: {len(parts)}\n")
    print("# lemma(1sg.pres.act.ind)\t" + "\t".join(CELLS)
          + '  ("-" = productive default)')
    for lemma in sorted(parts):
        print(lemma + "\t" + "\t".join(parts[lemma].get(c, "-") for c in CELLS))


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
