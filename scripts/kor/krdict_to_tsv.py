#!/usr/bin/env python3
"""Convert the NIKL 한국어기초사전 (Korean Basic Dictionary) LMF-XML dump
into the lemma<TAB>form<TAB>features TSV the Korean golden harness reads.

Usage: python3 scripts/kor/krdict_to_tsv.py data/kor/krdict-xml/*.xml \
           > data/kor/krdict.tsv

Each 동사 (verb) entry carries a handful of curated 활용 (conjugation)
WordForms — whole, attested surface words, editorially maintained by the
National Institute of Korean Language, wholly independent of Wiktionary.
They are the diagnostic forms that reveal a verb's irregular class:

  ...니 (not ...니다)   -> V;CONN;NI    (먹으니, 사니, 하니)
  ...ㅂ니다 / ...습니다  -> V;FORM;PRS   (먹습니다, 삽니다)
  ...는                 -> V;DET;PRS    (먹는, 사는)
  the -아/어 form       -> V;INTIMATE   (먹어, 하여/해, 꾸미어/꾸며)

The Basic Dictionary lists the -아/어 form in its full (uncontracted)
spelling (꾸미어, 하여); kaikki lists the contracted one (꾸며, 해). We emit
both the raw form and its regular contraction as variants of V;INTIMATE
so the two oracles overlap on that slot (mirroring the Catalan
diaeresis-variant handling). 준말 (contracted) sub-forms are emitted too.

Imperative 활용 (오너라, 가거라) are skipped — kaikki files them elsewhere.
"""
import glob
import re
import sys

# --- Hangul jamo compose/decompose and 아/어 contraction ---
LEAD = list("ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ")
VOWEL = list("ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ")


def dec(ch):
    c = ord(ch) - 0xAC00
    if 0 <= c < 11172:
        return (c // 588, (c % 588) // 28, c % 28)
    return None


def comp(l, v, t):
    return chr(0xAC00 + (l * 21 + v) * 28 + t)


# stem-final vowel + 아/어 -> combined vowel (coda-less stems only)
CONTR = {
    VOWEL.index("ㅣ"): VOWEL.index("ㅕ"),
    VOWEL.index("ㅏ"): VOWEL.index("ㅏ"),
    VOWEL.index("ㅓ"): VOWEL.index("ㅓ"),
    VOWEL.index("ㅗ"): VOWEL.index("ㅘ"),
    VOWEL.index("ㅜ"): VOWEL.index("ㅝ"),
    VOWEL.index("ㅐ"): VOWEL.index("ㅐ"),
    VOWEL.index("ㅔ"): VOWEL.index("ㅔ"),
    VOWEL.index("ㅚ"): VOWEL.index("ㅙ"),
    VOWEL.index("ㅕ"): VOWEL.index("ㅕ"),
}


def contract(form):
    """Contract a full -어/아 form (…V + 어/아[coda]) as krdict spells it."""
    if len(form) < 2:
        return form
    d1, d2 = dec(form[-2]), dec(form[-1])
    if not d1 or not d2:
        return form
    l1, v1, t1 = d1
    l2, v2, t2 = d2
    if t1 != 0:
        return form  # stem syllable has a coda: no contraction
    # 하 + 여 -> 해
    if (l1, v1) == (LEAD.index("ㅎ"), VOWEL.index("ㅏ")) and l2 == LEAD.index("ㅇ") and v2 == VOWEL.index("ㅕ"):
        return form[:-2] + comp(LEAD.index("ㅎ"), VOWEL.index("ㅐ"), t2)
    if l2 != LEAD.index("ㅇ") or v2 not in (VOWEL.index("ㅓ"), VOWEL.index("ㅏ")):
        return form
    if v1 in CONTR:
        return form[:-2] + comp(l1, CONTR[v1], t2)
    return form


WF = re.compile(
    r'type" val="활용" />\s*<feat att="writtenForm" val="([^"]+)"'
)
JUN = re.compile(r'준말" />\s*<feat att="writtenForm" val="([^"]+)"')
IMP = re.compile(r"(너라|거라|아라|어라|여라|라)$")


def classify(form):
    if form.endswith("니다"):
        return "V;FORM;PRS"
    if form.endswith("니"):
        return "V;CONN;NI"
    if form.endswith("는"):
        return "V;DET;PRS"
    if IMP.search(form):
        return None
    return "V;INTIMATE"


def emit(out, lemma, form):
    feat = classify(form)
    if feat is None:
        return
    out.add((lemma, form, feat))
    if feat == "V;INTIMATE":
        c = contract(form)
        if c != form:
            out.add((lemma, c, feat))


def main(paths):
    out = set()
    for path in paths:
        data = open(path, encoding="utf-8").read()
        for m in re.finditer(r"<LexicalEntry.*?</LexicalEntry>", data, re.S):
            blk = m.group(0)
            lm = re.search(r'writtenForm" val="([^"]+)"', blk)
            if not lm or "동사" not in blk:
                continue
            lemma = lm.group(1)
            if not lemma.endswith("다"):
                continue
            for w in re.finditer(r"<WordForm>(.*?)</WordForm>", blk, re.S):
                wb = w.group(1)
                if '"활용"' not in wb:
                    continue
                mm = WF.search(wb)
                if mm:
                    emit(out, lemma, mm.group(1))
                for j in JUN.findall(wb):
                    emit(out, lemma, j)
    for lemma, form, feat in sorted(out):
        sys.stdout.write(f"{lemma}\t{form}\t{feat}\n")


if __name__ == "__main__":
    files = []
    for p in sys.argv[1:]:
        files.extend(glob.glob(p))
    main(files)
