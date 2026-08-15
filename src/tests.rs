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
    assert_eq!(v.imperative(Singular).unwrap(), "fahre");
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
    assert_eq!(v.imperative(Singular).unwrap(), "lasse");
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
    assert_eq!(v.imperative(Singular).unwrap(), "habe");
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
