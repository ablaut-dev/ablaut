#!/usr/bin/env python3
"""Mine the irregular-verb lexicon (data/cat/verbs.tsv) from the FreeLing
gold paradigm, restricted to the lemmas the rule engine gets wrong.

Bases are reduced by prefix (córrer covers concórrer) but only when the
collapse is *safe*: a base B may absorb a derivative only if every gold
verb ending in B agrees with prefixation. An unsafe base (dir is a suffix
of the regular accedir) is flagged exact-match-only and its real compounds
are stored as their own rows. Run after a golden_cat pass wrote
target/golden_cat_mismatches.tsv."""
import collections

PN = {("SG","1"):0,("SG","2"):1,("SG","3"):2,("PL","1"):3,("PL","2"):4,("PL","3"):5}

def load_sets(path):
    g = collections.defaultdict(lambda: collections.defaultdict(set))
    for line in open(path, encoding="utf-8"):
        lemma, form, feat = line.rstrip("\n").split("\t")
        g[lemma][feat].add(form)
    return g

def merge(fr, ka):
    """One canonical form per (lemma, feature): a form the two oracles
    agree on wins (drops FreeLing data glitches like imprimir → impr);
    otherwise FreeLing, else kaikki fills the gap (haver's present is
    only in kaikki)."""
    g = collections.defaultdict(dict)
    for lemma in set(fr) | set(ka):
        feats = set(fr.get(lemma, {})) | set(ka.get(lemma, {}))
        for feat in feats:
            f = fr.get(lemma, {}).get(feat, set())
            k = ka.get(lemma, {}).get(feat, set())
            both = f & k
            pick = sorted(both)[0] if both else (sorted(f)[0] if f else sorted(k)[0])
            g[lemma][feat] = pick
    return g

gold = merge(load_sets("data/cat/freeling.tsv"), load_sets("data/cat/kaikki.tsv"))
all_lemmas = list(gold)
# The complete irregular set, captured once against an empty lexicon
# (scripts/cat/capture_irregulars.sh); a stable input keeps the mine
# reproducible instead of chasing a shrinking mismatch file.
err = sorted({l.strip() for l in open("scripts/cat/irregulars.txt") if l.strip()},
             key=lambda s: (len(s), s))

# index gold lemmas by suffix tail for fast "who ends in B" queries
by_tail = collections.defaultdict(list)
for lem in all_lemmas:
    by_tail[lem[-3:]].append(lem)

def matches_prefix(L, B):
    """True if L == prefix+B and every shared gold slot is prefix+B's."""
    if not L.endswith(B) or L == B:
        return False
    pre = L[:-len(B)]
    pl, pb = gold[L], gold[B]
    common = set(pl) & set(pb)
    return bool(common) and all(pl[f] == pre + pb[f] for f in common)

def is_safe(B):
    """B is safe as a suffix-base if no gold verb ending in B contradicts
    prefixation (i.e. B never hijacks a differently-inflected verb)."""
    for L in all_lemmas:
        if L != B and L.endswith(B):
            pre = L[:-len(B)]
            pl, pb = gold[L], gold[B]
            common = set(pl) & set(pb)
            if common and not all(pl[f] == pre + pb[f] for f in common):
                return False
    return True

# choose bases
kept, exact = [], set()
safe_cache = {}
for L in err:
    if not gold.get(L):
        continue
    covered = False
    for B in kept:
        if B in exact:
            continue
        if matches_prefix(L, B):
            covered = True
            break
    if covered:
        continue
    if L not in safe_cache:
        safe_cache[L] = is_safe(L)
    if not safe_cache[L]:
        exact.add(L)
    kept.append(L)

def col(lemma, tense):
    pl = gold[lemma]
    slots = [None]*6
    for feat, form in pl.items():
        parts = feat.split(";")
        if ";".join(parts[:-2]) == tense and (parts[-2], parts[-1]) in PN:
            slots[PN[(parts[-2], parts[-1])]] = form
    return "-" if any(s is None for s in slots) else ",".join(slots)

rows = []
for L in sorted(kept):
    pl = gold[L]
    fut1 = pl.get("V;IND;FUT;SG;1", "")
    row = [L,
           col(L,"V;IND;PRS"), col(L,"V;IND;PST;IPFV"), col(L,"V;IND;PST;PFV"),
           col(L,"V;SBJV;PRS"), col(L,"V;SBJV;PST"),
           fut1[:-1] if fut1.endswith("é") else "-",
           pl.get("V.PTCP;PST;MASC;SG","-"), pl.get("V;GER","-"),
           pl.get("V;IMP;SG;2","-"), "1" if L in exact else "-",
           pl.get("V.PTCP;PST;FEM;SG","-"), pl.get("V;IMP;PL;2","-")]
    rows.append("\t".join(row))

hdr = ("# Catalan irregular verbs — mined from FreeLing gold. Base lemmas;\n"
       "# prefixed derivatives match by suffix unless the base is exact (col 10).\n"
       "# Regenerate: python3 scripts/cat/mine_verbs.py\n"
       "# lemma\tpres\timpf\tpret\tsubjpres\tsubjimpf\tfut_stem\tpp\tger\timp2\texact\tppf\timp2pl\n")
open("data/cat/verbs.tsv","w",encoding="utf-8").write(hdr + "\n".join(rows) + "\n")
print(f"erroring lemmas: {len(err)}, stored: {len(kept)}, exact-only: {len(exact)}")
