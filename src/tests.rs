use super::*;
use Mood::*;
use Number::*;
use Person::*;
use Tense::*;

fn v(inf: &str) -> Verb {
    Verb::from_infinitive(inf).unwrap()
}

fn row(v: &Verb, tense: Tense, mood: Mood) -> [String; 6] {
    [
        v.conjugate(tense, mood, First, Singular),
        v.conjugate(tense, mood, Second, Singular),
        v.conjugate(tense, mood, Third, Singular),
        v.conjugate(tense, mood, First, Plural),
        v.conjugate(tense, mood, Second, Plural),
        v.conjugate(tense, mood, Third, Plural),
    ]
}

// ---------- weak paradigm ----------

#[test]
fn kaufen_plain_weak() {
    let v = v("kaufen");
    assert_eq!(
        row(&v, Present, Indicative),
        ["kaufe", "kaufst", "kauft", "kaufen", "kauft", "kaufen"]
    );
    assert_eq!(
        row(&v, Preterite, Indicative),
        ["kaufte", "kauftest", "kaufte", "kauften", "kauftet", "kauften"]
    );
    assert_eq!(v.past_participle(), "gekauft");
    assert_eq!(v.present_participle(), "kaufend");
    assert_eq!(v.auxiliary(), Auxiliary::Haben);
}

#[test]
fn arbeiten_epenthesis() {
    let v = v("arbeiten");
    assert_eq!(
        row(&v, Present, Indicative),
        ["arbeite", "arbeitest", "arbeitet", "arbeiten", "arbeitet", "arbeiten"]
    );
    assert_eq!(
        row(&v, Preterite, Indicative),
        ["arbeitete", "arbeitetest", "arbeitete", "arbeiteten", "arbeitetet", "arbeiteten"]
    );
    assert_eq!(v.past_participle(), "gearbeitet");
    assert_eq!(v.imperative(Singular).unwrap(), "arbeite");
}

#[test]
fn atmen_rechnen_epenthesis_after_obstruent() {
    let atmen = v("atmen");
    assert_eq!(atmen.conjugate(Present, Indicative, Second, Singular), "atmest");
    assert_eq!(atmen.conjugate(Present, Indicative, Third, Singular), "atmet");
    let rechnen = v("rechnen");
    assert_eq!(rechnen.conjugate(Present, Indicative, Second, Singular), "rechnest");
    assert_eq!(rechnen.past_participle(), "gerechnet");
}

#[test]
fn lernen_wohnen_no_epenthesis() {
    assert_eq!(v("lernen").conjugate(Present, Indicative, Second, Singular), "lernst");
    let wohnen = v("wohnen");
    assert_eq!(wohnen.conjugate(Present, Indicative, Second, Singular), "wohnst");
    assert_eq!(wohnen.past_participle(), "gewohnt");
}

#[test]
fn tanzen_s_coalescence() {
    let v = v("tanzen");
    assert_eq!(v.conjugate(Present, Indicative, Second, Singular), "tanzt");
    assert_eq!(v.conjugate(Preterite, Indicative, Second, Singular), "tanztest");
}

#[test]
fn sammeln_schwa_elision() {
    let v = v("sammeln");
    assert_eq!(
        row(&v, Present, Indicative),
        ["sammle", "sammelst", "sammelt", "sammeln", "sammelt", "sammeln"]
    );
    assert_eq!(v.past_participle(), "gesammelt");
    assert_eq!(v.imperative(Singular).unwrap(), "sammle");
    assert_eq!(v.present_participle(), "sammelnd");
}

#[test]
fn wandern_schwa_stem() {
    let v = v("wandern");
    assert_eq!(
        row(&v, Present, Indicative),
        ["wandere", "wanderst", "wandert", "wandern", "wandert", "wandern"]
    );
    assert_eq!(v.past_participle(), "gewandert");
}

#[test]
fn spielen_is_not_a_schwa_stem() {
    assert_eq!(v("spielen").conjugate(Present, Indicative, First, Singular), "spiele");
}

#[test]
fn studieren_no_ge_participle() {
    let v = v("studieren");
    assert_eq!(v.past_participle(), "studiert");
    assert_eq!(v.conjugate(Preterite, Indicative, Third, Singular), "studierte");
}

#[test]
fn konjunktiv_weak() {
    let v = v("kaufen");
    assert_eq!(
        row(&v, Present, KonjunktivI),
        ["kaufe", "kaufest", "kaufe", "kaufen", "kaufet", "kaufen"]
    );
    assert_eq!(row(&v, Present, KonjunktivII), row(&v, Preterite, Indicative));
}

#[test]
fn invalid_infinitive() {
    assert!(Verb::from_infinitive("kauf").is_err());
    assert!(Verb::weak("n").is_err());
}

// ---------- strong verbs ----------

#[test]
fn sprechen_e_i_alternation() {
    let v = v("sprechen");
    assert_eq!(
        row(&v, Present, Indicative),
        ["spreche", "sprichst", "spricht", "sprechen", "sprecht", "sprechen"]
    );
    assert_eq!(
        row(&v, Preterite, Indicative),
        ["sprach", "sprachst", "sprach", "sprachen", "spracht", "sprachen"]
    );
    assert_eq!(
        row(&v, Present, KonjunktivII),
        ["spräche", "sprächest", "spräche", "sprächen", "sprächet", "sprächen"]
    );
    assert_eq!(v.past_participle(), "gesprochen");
    assert_eq!(v.imperative(Singular).unwrap(), "sprich");
    assert_eq!(v.imperative(Plural).unwrap(), "sprecht");
}

#[test]
fn fahren_umlaut_alternation() {
    let v = v("fahren");
    assert_eq!(
        row(&v, Present, Indicative),
        ["fahre", "fährst", "fährt", "fahren", "fahrt", "fahren"]
    );
    // Umlaut verbs revert to the base stem in the imperative.
    assert_eq!(v.imperative(Singular).unwrap(), "fahr");
    assert_eq!(v.auxiliary(), Auxiliary::Sein);
    assert_eq!(v.past_participle(), "gefahren");
}

#[test]
fn halten_dental_stem_no_epenthesis() {
    let v = v("halten");
    // du hältst, er hält — never *hältest / *hältet.
    assert_eq!(v.conjugate(Present, Indicative, Second, Singular), "hältst");
    assert_eq!(v.conjugate(Present, Indicative, Third, Singular), "hält");
    // But the plain 2pl keeps epenthesis: ihr haltet.
    assert_eq!(v.conjugate(Present, Indicative, Second, Plural), "haltet");
    assert_eq!(v.conjugate(Preterite, Indicative, First, Singular), "hielt");
}

#[test]
fn lassen_coalescence_on_changed_stem() {
    let v = v("lassen");
    assert_eq!(v.conjugate(Present, Indicative, Second, Singular), "lässt");
    assert_eq!(v.conjugate(Present, Indicative, Third, Singular), "lässt");
    assert_eq!(v.imperative(Singular).unwrap(), "lass");
}

#[test]
fn singen_ablaut() {
    let v = v("singen");
    assert_eq!(
        row(&v, Preterite, Indicative),
        ["sang", "sangst", "sang", "sangen", "sangt", "sangen"]
    );
    assert_eq!(v.conjugate(Present, KonjunktivII, First, Singular), "sänge");
    assert_eq!(v.past_participle(), "gesungen");
}

#[test]
fn finden_preterite_epenthesis() {
    let v = v("finden");
    assert_eq!(
        row(&v, Preterite, Indicative),
        ["fand", "fandest", "fand", "fanden", "fandet", "fanden"]
    );
    assert_eq!(v.conjugate(Present, KonjunktivII, Third, Singular), "fände");
}

#[test]
fn sitzen_sibilant_preterite_2sg() {
    // Present coalesces (du sitzt) but the preterite takes -est (du saßest).
    let v = v("sitzen");
    assert_eq!(v.conjugate(Present, Indicative, Second, Singular), "sitzt");
    assert_eq!(v.conjugate(Preterite, Indicative, Second, Singular), "saßest");
    assert_eq!(v.past_participle(), "gesessen");
}

#[test]
fn vergessen_lexicalized_prefix_verb() {
    let v = v("vergessen");
    assert_eq!(v.conjugate(Present, Indicative, Second, Singular), "vergisst");
    // Inseparable prefix: participle without ge-.
    assert_eq!(v.past_participle(), "vergessen");
}

#[test]
fn stehen_konjunktiv_stund() {
    assert_eq!(v("stehen").conjugate(Present, KonjunktivII, First, Singular), "stünde");
}

#[test]
fn strong_konjunktiv_i_uses_base_stem() {
    assert_eq!(v("sehen").conjugate(Present, KonjunktivI, Third, Singular), "sehe");
    assert_eq!(v("geben").conjugate(Present, KonjunktivI, Second, Singular), "gebest");
}

// ---------- mixed verbs ----------

#[test]
fn denken_mixed() {
    let v = v("denken");
    assert_eq!(
        row(&v, Preterite, Indicative),
        ["dachte", "dachtest", "dachte", "dachten", "dachtet", "dachten"]
    );
    assert_eq!(v.conjugate(Present, KonjunktivII, First, Singular), "dächte");
    assert_eq!(v.past_participle(), "gedacht");
    assert_eq!(v.conjugate(Present, Indicative, Second, Singular), "denkst");
}

#[test]
fn senden_dental_mixed() {
    let v = v("senden");
    assert_eq!(v.conjugate(Preterite, Indicative, Third, Singular), "sandte");
    assert_eq!(v.conjugate(Present, KonjunktivII, Third, Singular), "sendete");
    assert_eq!(v.past_participle(), "gesandt");
}

#[test]
fn haben_full() {
    let v = v("haben");
    assert_eq!(
        row(&v, Present, Indicative),
        ["habe", "hast", "hat", "haben", "habt", "haben"]
    );
    assert_eq!(
        row(&v, Preterite, Indicative),
        ["hatte", "hattest", "hatte", "hatten", "hattet", "hatten"]
    );
    assert_eq!(
        row(&v, Present, KonjunktivII),
        ["hätte", "hättest", "hätte", "hätten", "hättet", "hätten"]
    );
    assert_eq!(v.past_participle(), "gehabt");
    assert_eq!(v.imperative(Singular).unwrap(), "hab");
    assert_eq!(v.imperative(Plural).unwrap(), "habt");
}

// ---------- preterite-presents (modals + wissen) ----------

#[test]
fn koennen_modal() {
    let v = v("können");
    assert_eq!(
        row(&v, Present, Indicative),
        ["kann", "kannst", "kann", "können", "könnt", "können"]
    );
    assert_eq!(
        row(&v, Preterite, Indicative),
        ["konnte", "konntest", "konnte", "konnten", "konntet", "konnten"]
    );
    assert_eq!(v.conjugate(Present, KonjunktivII, First, Singular), "könnte");
    assert_eq!(v.past_participle(), "gekonnt");
    assert_eq!(v.imperative(Singular), None);
    assert_eq!(v.imperative(Plural), None);
}

#[test]
fn muessen_coalescence() {
    let v = v("müssen");
    assert_eq!(v.conjugate(Present, Indicative, First, Singular), "muss");
    assert_eq!(v.conjugate(Present, Indicative, Second, Singular), "musst");
    assert_eq!(v.conjugate(Present, KonjunktivII, Third, Singular), "müsste");
}

#[test]
fn wissen_full() {
    let v = v("wissen");
    assert_eq!(
        row(&v, Present, Indicative),
        ["weiß", "weißt", "weiß", "wissen", "wisst", "wissen"]
    );
    assert_eq!(v.conjugate(Preterite, Indicative, First, Singular), "wusste");
    assert_eq!(v.conjugate(Present, KonjunktivII, First, Singular), "wüsste");
    assert_eq!(v.past_participle(), "gewusst");
    assert_eq!(v.imperative(Singular).unwrap(), "wisse");
}

// ---------- suppletives ----------

#[test]
fn sein_full() {
    let v = v("sein");
    assert_eq!(
        row(&v, Present, Indicative),
        ["bin", "bist", "ist", "sind", "seid", "sind"]
    );
    assert_eq!(
        row(&v, Preterite, Indicative),
        ["war", "warst", "war", "waren", "wart", "waren"]
    );
    assert_eq!(
        row(&v, Present, KonjunktivI),
        ["sei", "seist", "sei", "seien", "seiet", "seien"]
    );
    assert_eq!(v.conjugate(Present, KonjunktivII, First, Singular), "wäre");
    assert_eq!(v.past_participle(), "gewesen");
    assert_eq!(v.present_participle(), "seiend");
    assert_eq!(v.imperative(Singular).unwrap(), "sei");
    assert_eq!(v.imperative(Plural).unwrap(), "seid");
    assert_eq!(v.auxiliary(), Auxiliary::Sein);
}

#[test]
fn werden_full() {
    let v = v("werden");
    assert_eq!(
        row(&v, Present, Indicative),
        ["werde", "wirst", "wird", "werden", "werdet", "werden"]
    );
    assert_eq!(v.conjugate(Preterite, Indicative, Third, Singular), "wurde");
    assert_eq!(v.conjugate(Present, KonjunktivII, Third, Singular), "würde");
    assert_eq!(v.past_participle(), "geworden");
}

#[test]
fn tun_full() {
    let v = v("tun");
    assert_eq!(
        row(&v, Present, Indicative),
        ["tue", "tust", "tut", "tun", "tut", "tun"]
    );
    assert_eq!(v.conjugate(Preterite, Indicative, Second, Singular), "tatest");
    assert_eq!(v.conjugate(Present, KonjunktivII, First, Singular), "täte");
    assert_eq!(v.past_participle(), "getan");
}

// ---------- prefixed verbs ----------

#[test]
fn aufstehen_separable() {
    let v = v("aufstehen");
    assert_eq!(v.conjugate(Present, Indicative, First, Singular), "stehe auf");
    assert_eq!(v.conjugate(Present, Indicative, First, Plural), "stehen auf");
    assert_eq!(v.conjugate(Preterite, Indicative, Third, Singular), "stand auf");
    assert_eq!(v.conjugate(Present, KonjunktivII, First, Singular), "stünde auf");
    assert_eq!(v.past_participle(), "aufgestanden");
    assert_eq!(v.zu_infinitive(), "aufzustehen");
    assert_eq!(v.imperative(Singular).unwrap(), "steh auf");
    assert_eq!(v.present_participle(), "aufstehend");
    assert!(v.is_lexical());
}

#[test]
fn verstehen_inseparable() {
    let v = v("verstehen");
    assert_eq!(v.conjugate(Present, Indicative, First, Singular), "verstehe");
    assert_eq!(v.conjugate(Preterite, Indicative, Third, Singular), "verstand");
    assert_eq!(v.past_participle(), "verstanden");
    assert_eq!(v.zu_infinitive(), "zu verstehen");
}

#[test]
fn ansehen_inherits_stem_alternation() {
    let v = v("ansehen");
    assert_eq!(v.conjugate(Present, Indicative, Second, Singular), "siehst an");
    assert_eq!(v.imperative(Singular).unwrap(), "sieh an");
}

#[test]
fn abholen_separable_weak_base() {
    let v = v("abholen");
    assert_eq!(v.conjugate(Preterite, Indicative, Third, Singular), "holte ab");
    assert_eq!(v.past_participle(), "abgeholt");
    assert!(!v.is_lexical());
}

#[test]
fn erklaeren_inseparable_weak_base() {
    let v = v("erklären");
    assert_eq!(v.conjugate(Present, Indicative, Third, Singular), "erklärt");
    assert_eq!(v.past_participle(), "erklärt");
}

#[test]
fn anvertrauen_nested_prefixes() {
    let v = v("anvertrauen");
    assert_eq!(v.conjugate(Present, Indicative, First, Singular), "vertraue an");
    assert_eq!(v.past_participle(), "anvertraut");
    assert_eq!(v.zu_infinitive(), "anzuvertrauen");
}

#[test]
fn einstudieren_separable_ieren() {
    let v = v("einstudieren");
    assert_eq!(v.past_participle(), "einstudiert");
}

#[test]
fn false_splits_rejected() {
    // beten is not be+ten, zucken not zu+cken, festigen not fest+igen.
    assert_eq!(v("beten").past_participle(), "gebetet");
    assert_eq!(v("zucken").past_participle(), "gezuckt");
    assert_eq!(v("festigen").past_participle(), "gefestigt");
    // abonnieren is lexically forced weak, not ab+onnieren.
    let ab = v("abonnieren");
    assert_eq!(ab.conjugate(Present, Indicative, First, Singular), "abonniere");
    assert_eq!(ab.past_participle(), "abonniert");
}

#[test]
fn voraussetzen_fused_particle() {
    let v = v("voraussetzen");
    assert_eq!(v.conjugate(Preterite, Indicative, Second, Plural), "setztet voraus");
    assert_eq!(v.past_participle(), "vorausgesetzt");
    assert_eq!(v.zu_infinitive(), "vorauszusetzen");
}

#[test]
fn beanspruchen_frozen_inner_prefix() {
    let v = v("beanspruchen");
    assert_eq!(v.conjugate(Present, Indicative, First, Singular), "beanspruche");
    assert_eq!(v.past_participle(), "beansprucht");
}

#[test]
fn verhindern_not_ver_hin_dern() {
    let v = v("verhindern");
    assert_eq!(v.conjugate(Preterite, Indicative, Second, Plural), "verhindertet");
    assert_eq!(v.past_participle(), "verhindert");
}

#[test]
fn heranwachsen_collapsed_compound() {
    let v = v("heranwachsen");
    assert_eq!(v.conjugate(Present, Indicative, Third, Singular), "wächst heran");
    assert_eq!(v.past_participle(), "herangewachsen");
}

#[test]
fn rad_fahren_phrasal() {
    let v = v("Rad fahren");
    assert_eq!(v.conjugate(Present, Indicative, First, Singular), "fahre Rad");
    assert_eq!(v.past_participle(), "Rad gefahren");
    assert_eq!(v.present_participle(), "Rad fahrend");
    assert_eq!(v.zu_infinitive(), "Rad zu fahren");
    assert!(v.is_lexical());
}

#[test]
fn konjunktiv_i_keeps_full_endings_on_schwa_stems() {
    let v = v("sammeln");
    assert_eq!(v.conjugate(Present, KonjunktivI, Second, Singular), "sammelest");
    assert_eq!(v.conjugate(Present, KonjunktivI, Second, Plural), "sammelet");
}

#[test]
fn dual_prefix_overrides() {
    // umarmen is inseparable despite um- defaulting separable…
    let u = v("umarmen");
    assert_eq!(u.conjugate(Present, Indicative, First, Singular), "umarme");
    assert_eq!(u.past_participle(), "umarmt");
    // …umgeben inherits geben's ablaut through the frozen prefix…
    assert_eq!(v("umgeben").conjugate(Preterite, Indicative, Third, Singular), "umgab");
    // …and untertauchen is separable despite unter- defaulting inseparable.
    let t = v("untertauchen");
    assert_eq!(t.conjugate(Preterite, Indicative, Third, Singular), "tauchte unter");
    assert_eq!(t.past_participle(), "untergetaucht");
}

#[test]
fn forced_weak_whole_words() {
    // bereiten is not be+reiten, veranlassen is not veran+lassen.
    assert_eq!(v("bereiten").conjugate(Preterite, Indicative, Third, Singular), "bereitete");
    assert_eq!(v("veranlassen").conjugate(Preterite, Indicative, Third, Singular), "veranlasste");
    assert_eq!(v("wetteifern").past_participle(), "gewetteifert");
}

#[test]
fn ieren_and_schwa_base_guards() {
    // datieren is not da+tieren; rumpeln is not rum+peln.
    assert_eq!(v("datieren").conjugate(Present, Indicative, First, Singular), "datiere");
    assert_eq!(v("rumpeln").conjugate(Present, Indicative, First, Singular), "rumple");
    // …but real derivatives still split.
    assert_eq!(v("einstudieren").past_participle(), "einstudiert");
    assert_eq!(v("auswandern").past_participle(), "ausgewandert");
}

#[test]
fn mined_strong_verbs() {
    assert_eq!(v("schieben").conjugate(Preterite, Indicative, First, Singular), "schob");
    assert_eq!(v("treten").conjugate(Present, Indicative, Third, Singular), "tritt");
    assert_eq!(v("fressen").conjugate(Present, Indicative, Second, Singular), "frisst");
    assert_eq!(v("kriechen").past_participle(), "gekrochen");
    assert_eq!(v("treiben").past_participle(), "getrieben");
    assert_eq!(v("gelingen").auxiliary(), Auxiliary::Sein);
    // derivatives come free
    assert_eq!(v("anschleichen").conjugate(Preterite, Indicative, Second, Singular), "schlichst an");
    assert_eq!(v("verzeihen").past_participle(), "verziehen");
}

#[test]
fn knien_stem_keeps_e() {
    let k = v("knien");
    assert_eq!(
        row(&k, Present, Indicative),
        ["knie", "kniest", "kniet", "knien", "kniet", "knien"]
    );
    assert_eq!(k.conjugate(Preterite, Indicative, First, Singular), "kniete");
    assert_eq!(k.past_participle(), "gekniet");
}

#[test]
fn umringen_denominal_weak_base() {
    // umringen comes from the noun Ring, not strong ringen.
    let u = v("umringen");
    assert_eq!(u.conjugate(Preterite, Indicative, Third, Singular), "umringte");
    assert_eq!(u.past_participle(), "umringt");
}

#[test]
fn schwa_core_guard() {
    // gendern is not ge+ndern, dackeln is not da+ckeln.
    assert_eq!(v("gendern").past_participle(), "gegendert");
    assert_eq!(v("dackeln").conjugate(Present, Indicative, Third, Singular), "dackelt");
}

#[test]
fn adverbial_particles() {
    assert_eq!(v("zufriedenstellen").past_participle(), "zufriedengestellt");
    assert_eq!(
        v("stehenbleiben").conjugate(Preterite, Indicative, Second, Singular),
        "bliebst stehen"
    );
    assert_eq!(v("aufrechterhalten").conjugate(Present, Indicative, Third, Singular), "erhält aufrecht");
}

#[test]
fn native_vs_latinate_ieren() {
    // Native -ieren (Germanic stem): ge- participle, splittable.
    assert_eq!(v("schmieren").past_participle(), "geschmiert");
    assert_eq!(v("anschmieren").past_participle(), "angeschmiert");
    // Latinate -ieren: no ge-, never split by guesswork…
    assert_eq!(v("standardisieren").past_participle(), "standardisiert");
    assert_eq!(v("antworten").past_participle(), "geantwortet");
    // …but mined x rulings recover the real separable compounds.
    assert_eq!(v("abkommandieren").past_participle(), "abkommandiert");
    assert_eq!(
        v("abkommandieren").conjugate(Present, Indicative, First, Singular),
        "kommandiere ab"
    );
}

#[test]
fn beinhalten_beauftragen_frozen_weak() {
    assert_eq!(v("beinhalten").conjugate(Present, Indicative, Third, Singular), "beinhaltet");
    assert_eq!(v("beinhalten").past_participle(), "beinhaltet");
    assert_eq!(v("beauftragen").conjugate(Preterite, Indicative, Third, Singular), "beauftragte");
    assert_eq!(v("beauftragen").past_participle(), "beauftragt");
}

#[test]
fn haengen_intransitive_strong() {
    assert_eq!(v("hängen").conjugate(Preterite, Indicative, Third, Singular), "hing");
    assert_eq!(v("zusammenhängen").past_participle(), "zusammengehangen");
}
