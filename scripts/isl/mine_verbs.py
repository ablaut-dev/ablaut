#!/usr/bin/env python3
"""Mine the principal-parts lexicon (data/isl/verbs.tsv) from the BÍN gold
paradigm, restricted to the lemmas the ō-class rule engine gets wrong.

Bases are reduced by prefix (taka covers endurtaka) but only when the
collapse is *safe*: a base B may absorb a derivative only if every gold
verb ending in B agrees with prefixation. An unsafe base is flagged
exact-match-only. A tense column is emitted only when it differs from
the ō-class rule prediction, so near-regular verbs stay compact.

Run after a golden_isl pass wrote target/golden_isl_mismatches.tsv, or
after scripts/isl/capture_irregulars.sh refreshed scripts/isl/irregulars.txt.
"""
import collections

PN = {("SG", "1"): 0, ("SG", "2"): 1, ("SG", "3"): 2,
      ("PL", "1"): 3, ("PL", "2"): 4, ("PL", "3"): 5}

PRS = ["a", "ar", "ar", "um", "ið", "a"]
PST = ["aði", "aðir", "aði", "uðum", "uðuð", "uðu"]
SPRS = ["i", "ir", "i", "um", "ið", "i"]
SPST = ["aði", "aðir", "aði", "uðum", "uðuð", "uðu"]
TENSE_ENDINGS = {"V;IND;PRS": PRS, "V;IND;PST": PST,
                 "V;SBJV;PRS": SPRS, "V;SBJV;PST": SPST}


VOWELS = "aáeéiíoóuúyýæö"


def u_umlaut(stem, ending):
    """u-umlaut targets only the stem's final vowel, and only when it is
    a short 'a' (matches src/isl.rs)."""
    if not ending.startswith("u"):
        return stem
    last = max((i for i, c in enumerate(stem) if c in VOWELS), default=-1)
    if last < 0 or stem[last] != "a":
        return stem
    return stem[:last] + "ö" + stem[last + 1:]


def rule_form(lemma, tense, i):
    """The ō-class engine's prediction for a slot, or None if the lemma
    is not -a (only lexicon verbs reach that case)."""
    if not lemma.endswith("a"):
        return None
    stem = lemma[:-1]
    e = TENSE_ENDINGS[tense][i]
    return u_umlaut(stem, e) + e


def load_sets(path):
    g = collections.defaultdict(lambda: collections.defaultdict(set))
    for line in open(path, encoding="utf-8"):
        lemma, form, feat = line.rstrip("\n").split("\t")
        g[lemma][feat].add(form)
    return g


def merge(bn, ka):
    """One canonical form per (lemma, feature): a form both oracles agree
    on wins; else BÍN (authoritative); else kaikki fills the gap."""
    g = collections.defaultdict(dict)
    for lemma in set(bn) | set(ka):
        feats = set(bn.get(lemma, {})) | set(ka.get(lemma, {}))
        for feat in feats:
            b = bn.get(lemma, {}).get(feat, set())
            k = ka.get(lemma, {}).get(feat, set())
            both = b & k
            pick = sorted(both)[0] if both else (sorted(b)[0] if b else sorted(k)[0])
            g[lemma][feat] = pick
    return g


gold = merge(load_sets("data/isl/bin.tsv"), load_sets("data/isl/kaikki.tsv"))
all_lemmas = list(gold)
err = sorted({l.strip() for l in open("scripts/isl/irregulars.txt") if l.strip()},
             key=lambda s: (len(s), s))


def matches_prefix(L, B):
    if not L.endswith(B) or L == B:
        return False
    pre = L[:-len(B)]
    pl, pb = gold[L], gold[B]
    # The base must cover every slot the derivative has: a defective base
    # (væsa, attested only in the supine) must not absorb a full-paradigm
    # derivative (hvæsa) and leave its finite forms to the ō-fallback.
    if not set(pl) <= set(pb):
        return False
    return all(pl[f] == pre + pb[f] for f in pl)


def is_safe(B):
    for L in all_lemmas:
        if L != B and L.endswith(B):
            pre = L[:-len(B)]
            pl, pb = gold[L], gold[B]
            common = set(pl) & set(pb)
            if common and not all(pl[f] == pre + pb[f] for f in common):
                return False
    return True


kept, exact = [], set()
safe_cache = {}
for L in err:
    if not gold.get(L):
        continue
    if any(B not in exact and matches_prefix(L, B) for B in kept):
        continue
    if L not in safe_cache:
        safe_cache[L] = is_safe(L)
    if not safe_cache[L]:
        exact.add(L)
    kept.append(L)


def col(lemma, tense):
    """The six forms of a tense, or '-' when every attested slot already
    equals the ō-class rule (the engine's fallback covers it). Slots
    absent from gold (defective verbs) are padded with the rule
    prediction: they are never scored, and padding lets a partially
    attested tense still store its real forms (hvella → hvellur)."""
    pl = gold[lemma]
    slots = [None] * 6
    for feat, form in pl.items():
        parts = feat.split(";")
        if ";".join(parts[:-2]) == tense and (parts[-2], parts[-1]) in PN:
            slots[PN[(parts[-2], parts[-1])]] = form
    if all(s is None for s in slots):
        return "-"
    # Non -a lemmas (contracted strong verbs þvo/fá, impersonal ske) have
    # no ō-rule prediction: store the raw six-pack, padding any missing
    # slot (impersonals attest only the 3rd person) with an attested form
    # so the scored slots carry their real value.
    if not lemma.endswith("a"):
        fill = next(s for s in slots if s is not None)
        return ",".join(s if s is not None else fill for s in slots)
    filled = [slots[i] if slots[i] is not None else rule_form(lemma, tense, i)
              for i in range(6)]
    if all(filled[i] == rule_form(lemma, tense, i) for i in range(6)):
        return "-"
    return ",".join(filled)


rows = []
for L in sorted(kept):
    pl = gold[L]
    supine = pl.get("V.PTCP;PST", "-")
    if supine != "-" and L.endswith("a") and supine == L[:-1] + "að":
        supine = "-"
    presp = pl.get("V;V.PTCP;PRS", "-")
    if presp != "-" and presp == L + "ndi":
        presp = "-"
    imp2 = pl.get("V;IMP;SG;2", "-")
    if imp2 != "-" and imp2 == L:
        imp2 = "-"
    imp2pl = pl.get("V;IMP;PL;2", "-")
    # The engine's imp2pl fallback is the verb's *actual* present 2pl
    # (the stored override if any, else the ō-rule), so blank imp2pl only
    # when it equals that — not merely the ō-rule (vera: eruð vs verið).
    if imp2pl != "-" and imp2pl == pl.get("V;IND;PRS;PL;2"):
        imp2pl = "-"
    row = [L,
           col(L, "V;IND;PRS"), col(L, "V;IND;PST"),
           col(L, "V;SBJV;PRS"), col(L, "V;SBJV;PST"),
           supine, presp, imp2, imp2pl,
           "1" if L in exact else "-"]
    rows.append("\t".join(row))

hdr = ("# Icelandic irregular verbs — mined from BÍN gold (∩ kaikki where\n"
       "# they agree). Base lemmas; prefixed derivatives match by suffix\n"
       "# unless the base is exact-only (col 9). A '-' tense falls through\n"
       "# to the ō-class rule engine.\n"
       "# Regenerate: python3 scripts/isl/mine_verbs.py\n"
       "# lemma\tpres\tpast\tsubjpres\tsubjpast\tsupine\tpresp\timp2\timp2pl\texact\n")
open("data/isl/verbs.tsv", "w", encoding="utf-8").write(hdr + "\n".join(rows) + "\n")
print(f"erroring lemmas: {len(err)}, stored: {len(kept)}, exact-only: {len(exact)}")
