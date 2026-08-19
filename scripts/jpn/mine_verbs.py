#!/usr/bin/env python3
"""Mine the one lexical fact Japanese conjugation needs — the inflection
class of each verb — from IPADIC's 活用型, restricted to the verbs the two
oracles share (kaikki ∩ IPADIC). Everything else (the katsuyou-kei stems,
the onbin of the past た-form) is exceptionless and lives in the engine;
the class is the single thing an -る verb's surface cannot reveal (着る
ichidan vs 切る godan).

Class codes written to data/jpn/verbs.tsv:

    ichidan     一段          (食べる, 見る)
    godan       五段, row from the last kana; standard onbin
    godan_iku   五段・カ行促音便  (行く-family: past 行った, not *行いた)
    godan_u     五段・ワ行ウ音便  (問う-family: literary past 問うた)
    rspecial    五段・ラ行特殊   (なさる/くださる/いらっしゃる: 連用 なさい)
    suru        サ変          (愛する → 愛し)
    zuru        サ変・−ズル     (感ずる → 感じ)
    kuru        カ変          (来る → 来/き)

IPADIC keys the *homograph* dictionary forms する and くる to a godan ラ行
entry (摩る, 繰る), so those two are forced to their irregular class here.

Run after fetch_kaikki.sh and fetch_ipadic.sh. Regenerate:
    python3 scripts/jpn/mine_verbs.py
"""
import collections
import json

KAIKKI = "data/jpn/kaikki-verbs.jsonl"
IPADIC = "data/jpn/Verb.utf8.csv"  # UTF-8 conversion produced by fetch_ipadic.sh
OUT = "data/jpn/verbs.tsv"

TYPE_TO_CLASS = {
    "一段": "ichidan",
    "一段・クレル": "ichidan",
    "五段・カ行イ音便": "godan",
    "五段・カ行促音便": "godan_iku",
    "五段・ガ行": "godan",
    "五段・サ行": "godan",
    "五段・タ行": "godan",
    "五段・ナ行": "godan",
    "五段・バ行": "godan",
    "五段・マ行": "godan",
    "五段・ラ行": "godan",
    "五段・ラ行特殊": "rspecial",
    "五段・ワ行促音便": "godan",
    "五段・ワ行ウ音便": "godan_u",
    "サ変・スル": "suru",
    "サ変・−スル": "suru",
    "サ変・−ズル": "zuru",
    "カ変・来ル": "kuru",
    "カ変・クル": "kuru",
}
# Irregular verbs IPADIC records only under a godan homograph.
FORCE = {"する": "suru", "くる": "kuru", "来る": "kuru"}


def kaikki_lemmas():
    lemmas = set()
    for line in open(KAIKKI, encoding="utf-8"):
        w = json.loads(line).get("word", "").strip()
        if w and " " not in w:
            lemmas.add(w)
    return lemmas


def ipadic_types():
    t = {}
    for line in open(IPADIC, encoding="utf-8"):
        p = line.rstrip("\n").split(",")
        if len(p) < 11 or p[4] != "動詞":
            continue
        t.setdefault(p[10], p[8])
    return t


def main():
    kk = kaikki_lemmas()
    ipa = ipadic_types()
    rows = []
    counts = collections.Counter()
    for lemma in sorted(kk & set(ipa)):
        cls = FORCE.get(lemma) or TYPE_TO_CLASS.get(ipa[lemma])
        if cls is None:
            continue  # classical-only class (下二・/上二・/四段・): skip
        rows.append((lemma, cls))
        counts[cls] += 1
    hdr = (
        "# Japanese verbs — inflection class mined from IPADIC 活用型,\n"
        "# restricted to the kaikki ∩ IPADIC agreement set.\n"
        "# Regenerate: python3 scripts/jpn/mine_verbs.py\n"
        "# lemma\\tclass\n"
    )
    with open(OUT, "w", encoding="utf-8") as f:
        f.write(hdr)
        for lemma, cls in rows:
            f.write(f"{lemma}\t{cls}\n")
    print(f"verbs: {len(rows)}")
    for c, n in counts.most_common():
        print(f"  {c}: {n}")


if __name__ == "__main__":
    main()
