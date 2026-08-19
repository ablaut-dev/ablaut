#!/usr/bin/env python3
"""Mine the irregular-verb lexicon (data/rus/verbs.tsv) from the Russian
gold paradigm, restricted to the lemmas the rule engine gets wrong.

The engine strips the reflexive postfix (-ся/-сь) and conjugates the bare
base, so the lexicon stores *non-reflexive* base lemmas: одеть covers
одеться, надеть, переодеться. Reflexive gold forms are projected back to
their base by stripping the postfix. Bases are then reduced by prefix
(писать covers написать, переписать) but only when the collapse is safe
— a base that is a suffix of a differently-inflected verb is flagged
exact-match-only (column 4) and stored on its own.

Gold is OpenCorpora (canonical, hand-checked) with kaikki filling any
gap. Run after scripts/rus/capture_irregulars.sh wrote
scripts/rus/irregulars.txt.
"""
import collections
import os

os.chdir(os.path.join(os.path.dirname(__file__), "..", ".."))

VOWELS = "аеёиоуыэюя"

# The six non-past person/number slots, present (impf) or future (perf).
NONPAST = [("SG", "1"), ("SG", "2"), ("SG", "3"), ("PL", "1"), ("PL", "2"), ("PL", "3")]
PAST = ["V;IND;PST;MASC;SG", "V;IND;PST;FEM;SG", "V;IND;PST;NEUT;SG", "V;IND;PST;PL"]
IMP = ["V;IMP;SG;2", "V;IMP;PL;2"]


HUSHERS = "жшчщ"


def classify(base):
    n = len(base)
    if (base.endswith("овать") or base.endswith("евать")) and n > 6:
        return "ova"
    if base.endswith("нуть") and n > 4:
        return "nu"
    if base.endswith("ить") and n > 4:
        return "i"
    if base.endswith("ть"):
        return "aj"
    return "athematic"


def mutate_1sg(stem):
    for frm, to in (("ст", "щ"), ("зд", "зж")):
        if stem.endswith(frm):
            return stem[: -len(frm)] + to, False
    if not stem:
        return stem, False
    last, body = stem[-1], stem[:-1]
    table = {"т": "ч", "д": "ж", "с": "ш", "з": "ж", "к": "ч", "г": "ж", "х": "ш"}
    labial = {"б": "бл", "п": "пл", "в": "вл", "ф": "фл", "м": "мл"}
    if last in table:
        return body + table[last], False
    if last in labial:
        return body + labial[last], True
    return stem, False


def ova_stem(base):
    body = base[:-5]
    if base.endswith("евать"):
        return body + ("ю" if body and body[-1] in VOWELS else "у")
    return body + "у"


def rule_paradigm(base):
    """Mirror src/rus.rs: the rule-engine forms of a non-reflexive base.
    Returns {feature: form} for the finite slots it predicts."""
    cls = classify(base)
    out = {}
    NP = ["V;IND;PRS;SG;1", "V;IND;PRS;SG;2", "V;IND;PRS;SG;3",
          "V;IND;PRS;PL;1", "V;IND;PRS;PL;2", "V;IND;PRS;PL;3"]
    je_end = ["ю", "ешь", "ет", "ем", "ете", "ют"]
    if cls == "ova":
        s = ova_stem(base)
        np = [s + e for e in je_end]
        imp = s + "й"
    elif cls == "nu":
        s = base[:-3]
        np = [s + e for e in ["у", "ешь", "ет", "ем", "ете", "ут"]]
        imp = s + "и"
    elif cls == "i":
        r = base[:-3]
        m, labial = mutate_1sg(r)
        v1 = "у" if (not labial and m and m[-1] in HUSHERS) else "ю"
        e3 = "ат" if r and r[-1] in HUSHERS else "ят"
        np = [m + v1, r + "ишь", r + "ит", r + "им", r + "ите", r + e3]
        imp = r + "и"
    else:  # aj / athematic
        s = base[:-2]
        np = [s + e for e in je_end]
        imp = s + "й"
    for f, form in zip(NP, np):
        out[f] = form
    stem = base[:-2]
    for f, e in zip(PAST, ["л", "ла", "ло", "ли"]):
        out[f] = stem + e
    out["V;IMP;SG;2"] = imp
    out["V;IMP;PL;2"] = imp + "те"
    return out


def load(path):
    g = collections.defaultdict(lambda: collections.defaultdict(set))
    for line in open(path, encoding="utf-8"):
        lemma, form, feat = line.rstrip("\n").split("\t")
        g[lemma][feat].add(form)
    return g


def is_reflexive(lemma):
    return lemma.endswith(("ся", "сь"))


def strip_postfix(form):
    """Drop a trailing reflexive postfix from a surface form."""
    if form.endswith(("ся", "сь")):
        return form[:-2]
    return form


def project_sets(oc, ka):
    """Project every lemma to its non-reflexive base, stripping the
    reflexive postfix from the base key and each form. Returns
    {base: {feature: [opencorpora_forms, kaikki_forms]}}."""
    base = collections.defaultdict(
        lambda: collections.defaultdict(lambda: [set(), set()])
    )
    for src_idx, src in ((0, oc), (1, ka)):
        for lemma, feats in src.items():
            refl = is_reflexive(lemma)
            key = strip_postfix(lemma) if refl else lemma
            for feat, forms in feats.items():
                for form in forms:
                    base[key][feat][src_idx].add(strip_postfix(form) if refl else form)
    return base


def stem(form):
    """A form minus its non-past/imperative inflectional ending, so two
    readings can be told apart by stem (потреплешь → потрепл vs потрепишь
    → потреп) while an OpenCorpora ending typo (держут vs держат, both
    stem держ) does not masquerade as a different stem."""
    for end in ("ёшь", "ешь", "ишь", "ете", "ите", "ем", "ём", "им",
                "ет", "ёт", "ит", "ут", "ют", "ат", "ят", "у", "ю", "й",
                "и", "ь", "те"):
        if form.endswith(end) and len(form) > len(end):
            return form[: -len(end)]
    return form


def canonical(sets, feat, rule_form, ref=None):
    """Choose one stored form for a slot: an OpenCorpora form the rule
    engine does *not* already produce (the irregular reading — писать's
    пишу over the regular писаю), else any OpenCorpora form, else kaikki.

    When a lemma spells two lexemes the same way (потрепать = perfective
    потреплю *and* a 2nd-conjugation reading потрепишь), `ref` (the 1sg
    pick) keeps the paradigm inside one lexeme: a candidate whose stem
    equals the reference stem wins."""
    o, k = sets.get(feat, [set(), set()])
    cand = o or k
    if not cand:
        return None
    pool = [c for c in cand if c != rule_form] or list(cand)
    if ref:
        ref_stem = stem(ref)
        pool.sort(key=lambda c: (stem(c) != ref_stem, c))
    else:
        pool.sort()
    return pool[0]


def nonpast_column(sets, rule):
    """The six non-past forms. A slot's gold comes from V;IND;FUT if
    present (быть → буду, not archaic есмь), else V;IND;PRS. Returns "-"
    when the rule already produces every slot."""
    def feat_of(n, p):
        fut, prs = f"V;IND;FUT;{n};{p}", f"V;IND;PRS;{n};{p}"
        fut_forms = sets.get(fut)
        return fut if (fut_forms and (fut_forms[0] or fut_forms[1])) else prs

    # 1sg is chosen first and anchors the rest of the paradigm to one
    # lexeme's reading.
    ref = canonical(sets, feat_of("SG", "1"), rule.get("V;IND;PRS;SG;1"))
    vals, present, irregular = [], False, False
    for n, p in NONPAST:
        r = rule.get(f"V;IND;PRS;{n};{p}")
        c = canonical(sets, feat_of(n, p), r, ref)
        vals.append(c)
        if c is not None:
            present = True
            if c != r:
                irregular = True
    return _assemble(vals, [rule.get(f"V;IND;PRS;{n};{p}") for n, p in NONPAST], present, irregular), ref


def _assemble(vals, rule_forms, present, irregular):
    if not present or not irregular:
        return "-"
    filled = [v if v is not None else rule_forms[i] for i, v in enumerate(vals)]
    if any(x is None for x in filled):
        fallback = next(v for v in vals if v)
        filled = [x if x else fallback for x in filled]
    return ",".join(filled)


def simple_column(sets, feats, rule, ref=None):
    vals = [canonical(sets, f, rule.get(f), ref) for f in feats]
    present = any(v is not None for v in vals)
    irregular = any(v is not None and v != rule.get(feats[i]) for i, v in enumerate(vals))
    return _assemble(vals, [rule.get(f) for f in feats], present, irregular)


def main():
    oc = load("data/rus/opencorpora.tsv")
    ka = load("data/rus/kaikki.tsv")
    base_sets = project_sets(oc, ka)
    rules = {b: rule_paradigm(b) for b in base_sets}
    # A single canonical form per (base, feature) for the prefix-collapse
    # consistency checks (irregular reading preferred, OpenCorpora first).
    base_gold = {}
    for b, sets in base_sets.items():
        r = rules[b]
        m = {}
        for feat in sets:
            c = canonical(sets, feat, r.get(feat))
            if c is not None:
                m[feat] = c
        m["V;NFIN"] = b
        base_gold[b] = m

    # Erroring lemmas → their non-reflexive bases. Stable input.
    err_lemmas = {l.strip() for l in open("scripts/rus/irregulars.txt") if l.strip()}
    err_bases = set()
    for lemma in err_lemmas:
        b = strip_postfix(lemma) if is_reflexive(lemma) else lemma
        if b in base_gold:
            err_bases.add(b)
    err = sorted(err_bases, key=lambda s: (len(s), s))

    all_bases = list(base_gold)

    def matches_prefix(L, B):
        if not L.endswith(B) or L == B:
            return False
        pre = L[: -len(B)]
        gl, gb = base_gold[L], base_gold[B]
        common = set(gl) & set(gb)
        return bool(common) and all(gl[f] == pre + gb[f] for f in common)

    def is_safe(B):
        for L in all_bases:
            if L != B and L.endswith(B):
                pre = L[: -len(B)]
                gl, gb = base_gold[L], base_gold[B]
                common = set(gl) & set(gb)
                if common and not all(gl[f] == pre + gb[f] for f in common):
                    return False
        return True

    kept, exact = [], set()
    safe_cache = {}
    for L in err:
        if any((B not in exact) and matches_prefix(L, B) for B in kept):
            continue
        if L not in safe_cache:
            safe_cache[L] = is_safe(L)
        if not safe_cache[L]:
            exact.add(L)
        kept.append(L)

    rows = []
    stored = 0
    for L in sorted(kept):
        sets, r = base_sets[L], rules[L]
        np_col, ref = nonpast_column(sets, r)
        # The imperative shares the non-past stem, so anchor it to the same
        # reading (потрепли, not потрепи); the past uses its own reading.
        cols = [np_col, simple_column(sets, PAST, r), simple_column(sets, IMP, r, ref)]
        # A base whose every column matches the rule carries no
        # information (the engine reaches the same forms unaided): skip it,
        # unless it is exact-only (its presence blocks a longer base from
        # hijacking a regular verb).
        if all(c == "-" for c in cols) and L not in exact:
            continue
        stored += 1
        rows.append("\t".join([L, *cols, "1" if L in exact else "-"]))

    hdr = (
        "# Russian irregular verbs — mined from OpenCorpora gold (kaikki\n"
        "# fills gaps). Non-reflexive base lemmas; reflexives and prefixed\n"
        "# derivatives match by suffix unless the base is exact (col 4).\n"
        "# Regenerate: python3 scripts/rus/mine_verbs.py\n"
        "# lemma\tnonpast(6)\tpast(4:masc,fem,neut,pl)\timp(2:sg,pl)\texact\n"
    )
    open("data/rus/verbs.tsv", "w", encoding="utf-8").write(hdr + "\n".join(rows) + "\n")
    print(f"erroring bases: {len(err)}, kept: {len(kept)}, stored rows: {stored}, exact-only: {len(exact)}")


if __name__ == "__main__":
    main()
