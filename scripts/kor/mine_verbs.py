#!/usr/bin/env python3
"""Mine the Korean irregular-verb lexicon (data/kor/verbs.tsv) from the
two-oracle agreement.

Korean conjugation is regular given the stem's final jamo plus one of a
few irregular classes (ㅂ/ㄷ/ㅅ/르/러/우/ㅎ). This script reproduces the
engine's own class rules, then for every verb both oracles agree on it
picks the class whose generated forms match the agreed gold — storing
only the ones that are not Regular. A verb's class is thus *attested*,
not guessed: the stored class is exactly the one that reproduces the
surface forms kaikki and the Basic Dictionary independently confirm.

Run after the gold TSVs exist:
  python3 scripts/kor/mine_verbs.py data/kor/krdict.tsv data/kor/kaikki.tsv
"""
import collections
import sys

LEAD = list("ㄱㄲㄴㄷㄸㄹㅁㅂㅃㅅㅆㅇㅈㅉㅊㅋㅌㅍㅎ")
VOWEL = list("ㅏㅐㅑㅒㅓㅔㅕㅖㅗㅘㅙㅚㅛㅜㅝㅞㅟㅠㅡㅢㅣ")
L_R, L_O, L_H = LEAD.index("ㄹ"), LEAD.index("ㅇ"), LEAD.index("ㅎ")
V_A, V_EO, V_AE, V_EU = (VOWEL.index(x) for x in ("ㅏ", "ㅓ", "ㅐ", "ㅡ"))
T_NONE, T_L = 0, 8  # ㄹ coda index

CONTR = {
    VOWEL.index("ㅏ"): VOWEL.index("ㅏ"), VOWEL.index("ㅓ"): VOWEL.index("ㅓ"),
    VOWEL.index("ㅗ"): VOWEL.index("ㅘ"), VOWEL.index("ㅜ"): VOWEL.index("ㅝ"),
    VOWEL.index("ㅣ"): VOWEL.index("ㅕ"), VOWEL.index("ㅐ"): VOWEL.index("ㅐ"),
    VOWEL.index("ㅔ"): VOWEL.index("ㅔ"), VOWEL.index("ㅚ"): VOWEL.index("ㅙ"),
    VOWEL.index("ㅕ"): VOWEL.index("ㅕ"),
}
CODA_B, CODA_D, CODA_S, CODA_H = (
    "ㄱㄲㄳㄴㄵㄶㄷㄹㄺㄻㄼㄽㄾㄿㅀㅁㅂㅄㅅㅆㅇㅈㅊㅋㅌㅍㅎ".index(c) + 1
    for c in "ㅂㄷㅅㅎ"
)


def dec(ch):
    c = ord(ch) - 0xAC00
    return (c // 588, (c % 588) // 28, c % 28) if 0 <= c < 11172 else None


def comp(l, v, t):
    return chr(0xAC00 + (l * 21 + v) * 28 + t)


def harmony_a(v):
    return v in (VOWEL.index("ㅏ"), VOWEL.index("ㅗ"))


def intimate(stem, cls):
    head, (l, v, t) = stem[:-1], dec(stem[-1])
    if t == T_NONE and l == L_H and v == V_A:  # 하 -> 해
        return head + comp(L_H, V_AE, 0)
    if cls == "b":
        base = head + comp(l, v, 0)
        return base + ("와" if stem in ("돕", "곱") else "워")
    if cls == "d":
        return head + comp(l, v, T_L) + comp(L_O, V_A if harmony_a(v) else V_EO, 0)
    if cls == "s":
        return head + comp(l, v, 0) + comp(L_O, V_A if harmony_a(v) else V_EO, 0)
    if cls == "reu":
        d2 = dec(head[-1])
        base = head[:-1] + comp(d2[0], d2[1], T_L)
        return base + comp(L_R, V_A if harmony_a(d2[1]) else V_EO, 0)
    if cls == "reo":
        return stem + comp(L_R, V_EO, 0)
    if cls == "u":
        return head + comp(l, V_EO, 0)
    if cls == "h":
        return head + comp(l, V_AE if v == V_A else v, 0)
    if cls == "eae":  # 그러 -> 그래
        return head + comp(l, V_AE, 0)
    # regular
    if t != T_NONE:
        return head + comp(l, v, t) + comp(L_O, V_A if harmony_a(v) else V_EO, 0)
    if v == V_EU:
        a = dec(head[-1]) if head else None
        return head + comp(l, V_A if (a and harmony_a(a[1])) else V_EO, 0)
    if v in CONTR:
        return head + comp(l, CONTR[v], 0)
    return head + comp(l, v, 0) + comp(L_O, V_A if harmony_a(v) else V_EO, 0)


def conn_ni(stem, cls):
    head, (l, v, t) = stem[:-1], dec(stem[-1])
    if cls == "b":
        return head + comp(l, v, 0) + "우니"
    if cls == "d":
        return head + comp(l, v, T_L) + "으니"
    if cls == "s":
        return head + comp(l, v, 0) + "으니"
    if cls == "h":
        return head + comp(l, v, 0) + "니"
    if t == T_NONE or t == T_L:
        return head + comp(l, v, 0) + "니"
    return head + comp(l, v, t) + "으니"


CANDIDATES = {
    CODA_B: ["b"], CODA_D: ["d"], CODA_S: ["s"], CODA_H: ["h"],
}


def candidate_classes(stem):
    d = dec(stem[-1])
    if d is None:
        return []
    l, v, t = d
    if t == 0 and l == L_R and v == V_EU:  # 르-final
        return ["reu", "reo"]
    if stem == "푸":
        return ["u"]
    # coda-less ㅓ after another syllable: the 그러다/어쩌다 contraction.
    if t == 0 and v == V_EO and len(stem) >= 2:
        return ["eae"]
    return CANDIDATES.get(t, [])


def load(path):
    g = collections.defaultdict(lambda: collections.defaultdict(set))
    for line in open(path, encoding="utf-8"):
        lemma, form, feat = line.rstrip("\n").split("\t")
        g[lemma][feat].add(form)
    return g


def merge(a, b):
    """Per-lemma before-vowel evidence for class assignment: a slot both
    oracles attest with overlap contributes their union; a slot only one
    attests contributes that one (the other is silent, not contradicting,
    so 모르다's 몰라 — which the Basic Dictionary omits — is still usable);
    a slot both attest but disjointly (짝짓다) is dropped as unreliable."""
    g = collections.defaultdict(lambda: collections.defaultdict(set))
    for lemma in set(a) | set(b):
        av, bv = a.get(lemma, {}), b.get(lemma, {})
        for feat in set(av) | set(bv):
            x, y = av.get(feat, set()), bv.get(feat, set())
            if x and y:
                if x & y:
                    g[lemma][feat] = x | y
            else:
                g[lemma][feat] = x | y
    return g


def score(stem, cls, gold):
    """How many of the agreed before-vowel slots this class reproduces."""
    n = 0
    if "V;INTIMATE" in gold and intimate(stem, cls) in gold["V;INTIMATE"]:
        n += 1
    if "V;CONN;NI" in gold and conn_ni(stem, cls) in gold["V;CONN;NI"]:
        n += 1
    return n


def main(krdict, kaikki):
    gold = merge(load(krdict), load(kaikki))
    rows = []
    for lemma in sorted(gold):
        if not lemma.endswith("다"):
            continue
        stem = lemma[:-1]
        cands = candidate_classes(stem)
        if not cands:
            continue
        reg = score(stem, "regular", gold[lemma])
        best = max(cands, key=lambda c: score(stem, c, gold[lemma]))
        # Store the irregular class only when it strictly beats Regular
        # on the agreed evidence (so 곱다/따르다 stay regular).
        if score(stem, best, gold[lemma]) > reg:
            rows.append((lemma, best))
    hdr = (
        "# Korean irregular verbs — lemma<TAB>class, mined from the "
        "kaikki ∩ krdict\n# agreement (scripts/kor/mine_verbs.py). Class "
        "codes: b=ㅂ, d=ㄷ, s=ㅅ,\n# reu=르(ㄹㄹ), reo=러, u=우, h=ㅎ. "
        "Regular verbs (plain codas, ㄹ-stems,\n# ㅡ-deletion, 하다) are "
        "predictable and never stored.\n"
    )
    with open("data/kor/verbs.tsv", "w", encoding="utf-8") as f:
        f.write(hdr)
        for lemma, cls in rows:
            f.write(f"{lemma}\t{cls}\n")
    counts = collections.Counter(c for _, c in rows)
    print(f"stored {len(rows)} irregular verbs: {dict(counts)}")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
