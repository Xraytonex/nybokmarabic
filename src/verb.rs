//! Arabic verb templates applied to Norwegian roots (SPEC §4.8).
//!
//! Patterns are written with the digits 1–4 standing for the radicals and
//! `T` for the Form VIII infix. Every pattern was checked against standard
//! Arabic grammar (see CORRECTIONS.md); the Nybokmarabic additions are the
//! Norwegian `-r` on the present tense and the root extraction itself.

use serde::Serialize;

use crate::root::Root;
use crate::text::Gender;

/// Arabic verb forms. Triliteral roots use I–X (IX is skipped: it is
/// reserved for colours and bodily defects). Quadriliteral roots use Q1, Q2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Form {
    I,
    II,
    III,
    IV,
    V,
    VI,
    VII,
    VIII,
    X,
    Q1,
    Q2,
}

/// The order in which colliding verbs are assigned forms.
pub const TRILITERAL_ORDER: &[u8] = &[1, 2, 3, 4, 5, 6, 7, 8, 10];
pub const QUADRILITERAL_ORDER: &[u8] = &[1, 2];

impl Form {
    /// Interpret a lexicon `form = n` number for a given root.
    pub fn from_number(n: u8, root: &Root) -> Option<Form> {
        if root.is_quadriliteral() {
            return match n {
                1 => Some(Form::Q1),
                2 => Some(Form::Q2),
                _ => None,
            };
        }
        Some(match n {
            1 => Form::I,
            2 => Form::II,
            3 => Form::III,
            4 => Form::IV,
            5 => Form::V,
            6 => Form::VI,
            7 => Form::VII,
            8 => Form::VIII,
            10 => Form::X,
            _ => return None,
        })
    }

    pub fn number(self) -> u8 {
        match self {
            Form::I | Form::Q1 => 1,
            Form::II | Form::Q2 => 2,
            Form::III => 3,
            Form::IV => 4,
            Form::V => 5,
            Form::VI => 6,
            Form::VII => 7,
            Form::VIII => 8,
            Form::X => 10,
        }
    }

    fn template(self) -> &'static Template {
        match self {
            Form::I => &FORM_I,
            Form::II => &FORM_II,
            Form::III => &FORM_III,
            Form::IV => &FORM_IV,
            Form::V => &FORM_V,
            Form::VI => &FORM_VI,
            Form::VII => &FORM_VII,
            Form::VIII => &FORM_VIII,
            Form::X => &FORM_X,
            Form::Q1 => &FORM_Q1,
            Form::Q2 => &FORM_Q2,
        }
    }
}

struct Template {
    /// Past stem; the person suffix follows (`-a` for 3sg m).
    past: &'static str,
    /// Present stem; the person prefix precedes, `-u` + `r` follow.
    present: &'static str,
    /// Vowel of the present person prefix: `a` (ya-) or `u` (yu-).
    prefix_vowel: char,
    imperative: &'static str,
    active: &'static str,
    passive: &'static str,
    verbal_noun: &'static str,
    place: &'static str,
}

const FORM_I: Template = Template {
    past: "1a2a3",
    present: "12u3",
    prefix_vowel: 'a',
    imperative: "u12u3",
    active: "1ā2i3",
    passive: "ma12ū3",
    verbal_noun: "1a23",
    place: "ma12a3",
};
const FORM_II: Template = Template {
    past: "1a22a3",
    present: "1a22i3",
    prefix_vowel: 'u',
    imperative: "1a22i3",
    active: "mu1a22i3",
    passive: "mu1a22a3",
    verbal_noun: "ta12ī3",
    place: "mu1a22a3",
};
const FORM_III: Template = Template {
    past: "1ā2a3",
    present: "1ā2i3",
    prefix_vowel: 'u',
    imperative: "1ā2i3",
    active: "mu1ā2i3",
    passive: "mu1ā2a3",
    verbal_noun: "mu1ā2a3a",
    place: "mu1ā2a3",
};
const FORM_IV: Template = Template {
    past: "a12a3",
    present: "12i3",
    prefix_vowel: 'u',
    imperative: "a12i3",
    active: "mu12i3",
    passive: "mu12a3",
    verbal_noun: "i12ā3",
    place: "mu12a3",
};
const FORM_V: Template = Template {
    past: "ta1a22a3",
    present: "ta1a22a3",
    prefix_vowel: 'a',
    imperative: "ta1a22a3",
    active: "muta1a22i3",
    passive: "muta1a22a3",
    verbal_noun: "ta1a22u3",
    place: "muta1a22a3",
};
const FORM_VI: Template = Template {
    past: "ta1ā2a3",
    present: "ta1ā2a3",
    prefix_vowel: 'a',
    imperative: "ta1ā2a3",
    active: "muta1ā2i3",
    passive: "muta1ā2a3",
    verbal_noun: "ta1ā2u3",
    place: "muta1ā2a3",
};
const FORM_VII: Template = Template {
    past: "in1a2a3",
    present: "n1a2i3",
    prefix_vowel: 'a',
    imperative: "in1a2i3",
    active: "mun1a2i3",
    passive: "mun1a2a3",
    verbal_noun: "in1i2ā3",
    place: "mun1a2a3",
};
const FORM_VIII: Template = Template {
    past: "i1Ta2a3",
    present: "1Ta2i3",
    prefix_vowel: 'a',
    imperative: "i1Ta2i3",
    active: "mu1Ta2i3",
    passive: "mu1Ta2a3",
    verbal_noun: "i1Ti2ā3",
    place: "mu1Ta2a3",
};
const FORM_X: Template = Template {
    past: "ista12a3",
    present: "sta12i3",
    prefix_vowel: 'a',
    imperative: "ista12i3",
    active: "musta12i3",
    passive: "musta12a3",
    verbal_noun: "isti12ā3",
    place: "musta12a3",
};
const FORM_Q1: Template = Template {
    past: "1a23a4",
    present: "1a23i4",
    prefix_vowel: 'u',
    imperative: "1a23i4",
    active: "mu1a23i4",
    passive: "mu1a23a4",
    verbal_noun: "1a23a4a",
    place: "mu1a23a4",
};
const FORM_Q2: Template = Template {
    past: "ta1a23a4",
    present: "ta1a23a4",
    prefix_vowel: 'a',
    imperative: "ta1a23a4",
    active: "muta1a23i4",
    passive: "muta1a23a4",
    verbal_noun: "ta1a23u4",
    place: "muta1a23a4",
};

/// Fill a pattern with the radicals of a root.
fn fill(pattern: &str, root: &Root) -> String {
    let mut out = String::new();
    let mut chars = pattern.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '1'..='4' => {
                let i = c as usize - '1' as usize;
                // Form VIII: a weak first radical (w, y) assimilates to the
                // infix, giving itt- (ittaṣala from w-ṣ-l).
                if i == 0 && chars.peek() == Some(&'T') && matches!(root.radical(0), "w" | "y") {
                    out.push('t');
                } else {
                    out.push_str(root.radical(i));
                }
            }
            'T' => out.push_str(form_viii_infix(root.radical(0))),
            other => out.push(other),
        }
    }
    out
}

/// The Form VIII infix `-t-` assimilates to some first radicals.
fn form_viii_infix(first: &str) -> &'static str {
    match first {
        "d" => "d",
        "z" => "d",
        "th" => "th",
        "dh" => "dh",
        "ṣ" | "ḍ" => "ṭ",
        "ṭ" => "ṭ",
        "ẓ" => "ẓ",
        _ => "t",
    }
}

/// Grammatical person for tense marking.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Person {
    #[serde(rename = "1sg")]
    First,
    #[serde(rename = "2sg")]
    Second,
    #[serde(rename = "3sgm")]
    ThirdMasc,
    #[serde(rename = "3sgf")]
    ThirdFem,
    #[serde(rename = "1pl")]
    FirstPlural,
    #[serde(rename = "3pl")]
    ThirdPlural,
}

impl Person {
    pub fn parse(s: &str) -> Option<Person> {
        Some(match s {
            "1sg" | "i" => Person::First,
            "2sg" | "you" => Person::Second,
            "3sgm" | "3sg" | "he" => Person::ThirdMasc,
            "3sgf" | "she" => Person::ThirdFem,
            "1pl" | "we" => Person::FirstPlural,
            "3pl" | "they" => Person::ThirdPlural,
            _ => return None,
        })
    }

    /// Third person singular agreeing with a subject of `gender`.
    pub fn third(gender: Gender) -> Person {
        match gender.verb_agreement() {
            Gender::Masculine => Person::ThirdMasc,
            _ => Person::ThirdFem,
        }
    }

    fn past_suffix(self) -> &'static str {
        match self {
            Person::First => "tu",
            Person::Second => "ta",
            Person::ThirdMasc => "a",
            Person::ThirdFem => "at",
            Person::FirstPlural => "nā",
            Person::ThirdPlural => "ū",
        }
    }

    fn present_prefix(self, vowel: char) -> String {
        let consonant = match self {
            Person::First => "",
            Person::Second | Person::ThirdFem => "t",
            Person::ThirdMasc | Person::ThirdPlural => "y",
            Person::FirstPlural => "n",
        };
        format!("{consonant}{vowel}")
    }

    fn present_ending(self) -> &'static str {
        match self {
            Person::ThirdPlural => "ūna",
            _ => "u",
        }
    }
}

/// A conjugable verb: a root in a form.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Verb {
    pub root: Root,
    pub form: Form,
}

impl Verb {
    pub fn new(root: Root, form: Form) -> Self {
        Verb { root, form }
    }

    pub fn past(&self, person: Person) -> String {
        format!(
            "{}{}",
            fill(self.form.template().past, &self.root),
            person.past_suffix()
        )
    }

    /// Arabic person prefix, Arabic mood ending, Norwegian `-r`.
    pub fn present(&self, person: Person) -> String {
        let t = self.form.template();
        format!(
            "{}{}{}r",
            person.present_prefix(t.prefix_vowel),
            fill(t.present, &self.root),
            person.present_ending()
        )
    }

    pub fn imperative(&self) -> String {
        fill(self.form.template().imperative, &self.root)
    }

    pub fn active_participle(&self) -> String {
        fill(self.form.template().active, &self.root)
    }

    pub fn passive_participle(&self) -> String {
        fill(self.form.template().passive, &self.root)
    }

    pub fn verbal_noun(&self) -> String {
        fill(self.form.template().verbal_noun, &self.root)
    }

    pub fn noun_of_place(&self) -> String {
        fill(self.form.template().place, &self.root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::root::extract;

    fn verb(inf: &str, n: u8) -> Verb {
        let root = extract(inf);
        let form = Form::from_number(n, &root).unwrap();
        Verb::new(root, form)
    }

    #[test]
    fn form_i_paradigm_for_hente() {
        let v = verb("hente", 1);
        assert_eq!(v.past(Person::ThirdMasc), "hanata");
        assert_eq!(v.present(Person::ThirdMasc), "yahnutur");
        assert_eq!(v.imperative(), "uhnut");
        assert_eq!(v.active_participle(), "hānit");
        assert_eq!(v.passive_participle(), "mahnūt");
        assert_eq!(v.verbal_noun(), "hant");
        assert_eq!(v.noun_of_place(), "mahnat");
    }

    #[test]
    fn quadriliteral_paradigm_for_skrive() {
        let v = verb("skrive", 1);
        assert_eq!(v.past(Person::ThirdMasc), "sakrava");
        assert_eq!(v.present(Person::ThirdMasc), "yusakrivur");
        assert_eq!(v.imperative(), "sakriv");
        assert_eq!(v.active_participle(), "musakriv");
        assert_eq!(v.passive_participle(), "musakrav");
        assert_eq!(v.verbal_noun(), "sakrava");
        assert_eq!(v.noun_of_place(), "musakrav");
    }

    #[test]
    fn person_marking() {
        let v = verb("hente", 1);
        assert_eq!(v.present(Person::First), "ahnutur");
        assert_eq!(v.present(Person::ThirdFem), "tahnutur");
        assert_eq!(v.present(Person::FirstPlural), "nahnutur");
        assert_eq!(v.present(Person::ThirdPlural), "yahnutūnar");
        assert_eq!(v.past(Person::First), "hanattu");
        assert_eq!(v.past(Person::ThirdFem), "hanatat");
        assert_eq!(v.past(Person::FirstPlural), "hanatnā");
        assert_eq!(v.past(Person::ThirdPlural), "hanatū");
    }

    #[test]
    fn form_ii_doubles_the_middle_radical() {
        let v = verb("hente", 2);
        assert_eq!(v.past(Person::ThirdMasc), "hannata");
        assert_eq!(v.present(Person::ThirdMasc), "yuhannitur");
        assert_eq!(v.verbal_noun(), "tahnīt");
    }

    #[test]
    fn form_ii_doubles_a_digraph_whole() {
        let v = verb("vaske", 2);
        assert_eq!(extract("vaske").display(), "V-S-K");
        let w = verb("tusje", 2);
        assert_eq!(w.past(Person::ThirdMasc), "tashshawa");
        assert_eq!(v.past(Person::ThirdMasc), "vassaka");
    }

    #[test]
    fn form_viii_assimilates_the_infix() {
        assert_eq!(verb("dele", 8).past(Person::ThirdMasc), "iddalawa");
        assert_eq!(verb("zoome", 8).past(Person::ThirdMasc), "izdamawa");
        assert_eq!(verb("lese", 8).past(Person::ThirdMasc), "iltasawa");
    }

    #[test]
    fn form_x() {
        assert_eq!(verb("hente", 10).past(Person::ThirdMasc), "istahnata");
        assert_eq!(verb("hente", 10).present(Person::ThirdMasc), "yastahnitur");
    }

    #[test]
    fn form_numbers_round_trip() {
        let tri = extract("hente");
        for &n in TRILITERAL_ORDER {
            assert_eq!(Form::from_number(n, &tri).unwrap().number(), n);
        }
        assert!(Form::from_number(9, &tri).is_none());
        let quad = extract("skrive");
        assert_eq!(Form::from_number(2, &quad), Some(Form::Q2));
        assert!(Form::from_number(3, &quad).is_none());
    }
}
