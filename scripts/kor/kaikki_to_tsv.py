#!/usr/bin/env python3
"""Convert the kaikki.org (Wiktextract) Korean verb JSONL into the
lemma<TAB>form<TAB>features TSV the Korean golden harness reads.

Usage: python3 scripts/kor/kaikki_to_tsv.py data/kor/kaikki-verbs.jsonl \
           > data/kor/kaikki.tsv

kaikki lists ~49 whole-word conjugated forms per verb, tagged by speech
level and mood. We keep the four the NIKL Basic Dictionary also attests
as whole words, so the two oracles can be intersected:

  {indicative, informal, non-past}          -> V;INTIMATE   (먹어)
  {causative, informal, polite}             -> V;CONN;NI    (먹으니)
  {determiner, formal, present}             -> V;DET;PRS    (먹는)
  {formal, indicative, non-past, polite}    -> V;FORM;PRS   (먹습니다)

The honorific -시- variants kaikki files under the same tags (하셔, 하시니)
are kept as extra variants; they never hurt (agreement is by overlap).
"""
import json
import sys

WANT = {
    frozenset({"indicative", "informal", "non-past"}): "V;INTIMATE",
    frozenset({"causative", "informal", "polite"}): "V;CONN;NI",
    frozenset({"determiner", "formal", "present"}): "V;DET;PRS",
    frozenset({"formal", "indicative", "non-past", "polite"}): "V;FORM;PRS",
}


def main(path):
    seen = set()
    for line in open(path, encoding="utf-8"):
        entry = json.loads(line)
        lemma = entry.get("word", "")
        if not lemma:
            continue
        for f in entry.get("forms", []):
            if f.get("source") != "conjugation":
                continue
            feat = WANT.get(frozenset(f.get("tags", [])))
            if feat is None:
                continue
            form = f.get("form", "").strip()
            if not form or form in ("-", "—"):
                continue
            row = (lemma, form, feat)
            if row not in seen:
                seen.add(row)
                sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    main(sys.argv[1])
