//! Nouns, adjectives, the triple article (SPEC §4.4), iḍāfa (§4.5),
//! adjective agreement (§4.6) and number (§4.7).

use serde::Serialize;

use crate::lexicon::Entry;
use crate::root::consonants;
use crate::text::{strip_final, syllables, Gender, Number, Standard};

/// A noun in its selected standard, with its whole paradigm.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Noun {
    pub lemma: String,
    pub gender: Gender,
    pub standard: Standard,
    pub def: String,
    pub pl: String,
    pub def_pl: String,
    /// Acronyms (`HTTP`) put a hyphen before every ending, the dual too.
    pub acronym: bool,
}

impl Noun {
    pub fn from_entry(e: &Entry) -> Noun {
        let gender = e.gender().unwrap_or(Gender::Masculine);
        let standard = e.standard();
        let forms = e.forms();
        if forms.len() == 3 && forms.iter().all(|f| f != "-") {
            Noun {
                lemma: e.lemma().to_string(),
                gender,
                standard,
                def: forms[0].clone(),
                pl: forms[1].clone(),
                def_pl: forms[2].clone(),
                acronym: false,
            }
        } else {
            Noun::regular(e.lemma(), gender, standard)
        }
    }

    /// Nouns made from Arabic templates (participles, verbal nouns) are their
    /// own lemma for the parity rule. A final `-a` looks like the Arabic
    /// feminine ending, so those are feminine; the rest are masculine.
    pub fn derived(word: &str) -> Noun {
        let gender = if word.ends_with('a') {
            Gender::Feminine
        } else {
            Gender::Masculine
        };
        Noun::regular(word, gender, Standard::for_lemma(word))
    }

    /// An acronym kept as written (`HTTP`): neuter, and every ending follows
    /// a hyphen, as Norwegian writes abbreviations (`HTTP-et`, `API-er`).
    pub fn acronym(written: &str) -> Noun {
        let standard = Standard::for_lemma(written);
        let (pl, def_pl) = match standard {
            Standard::Bokmal => (format!("{written}-er"), format!("{written}-ene")),
            Standard::Nynorsk => (written.to_string(), format!("{written}-a")),
        };
        Noun {
            lemma: written.to_string(),
            gender: Gender::Neuter,
            standard,
            def: format!("{written}-et"),
            pl,
            def_pl,
            acronym: true,
        }
    }

    /// Regular Norwegian inflection, used for words the dictionary does not
    /// supply (proposed words and Arabic-derived nouns).
    pub fn regular(lemma: &str, gender: Gender, standard: Standard) -> Noun {
        let e_final = lemma.ends_with('e');
        let bare = strip_final(lemma, 'e');
        let (def, pl, def_pl) = match (standard, gender) {
            (_, Gender::Feminine) => {
                let base = strip_final(lemma, 'a');
                let base = strip_final(base, 'e');
                (
                    format!("{base}a"),
                    format!("{base}er"),
                    format!("{base}ene"),
                )
            }
            (Standard::Bokmal, Gender::Masculine) => {
                if lemma.ends_with("er") {
                    (
                        format!("{lemma}en"),
                        format!("{lemma}e"),
                        format!("{lemma}ne"),
                    )
                } else {
                    (
                        format!("{bare}en"),
                        format!("{bare}er"),
                        format!("{bare}ene"),
                    )
                }
            }
            (Standard::Nynorsk, Gender::Masculine) => (
                format!("{bare}en"),
                format!("{bare}ar"),
                format!("{bare}ane"),
            ),
            (Standard::Bokmal, Gender::Neuter) => {
                let pl = if syllables(lemma) <= 1 {
                    lemma.to_string()
                } else {
                    format!("{bare}er")
                };
                (format!("{bare}et"), pl, format!("{bare}ene"))
            }
            (Standard::Nynorsk, Gender::Neuter) => {
                let def = if e_final {
                    format!("{lemma}t")
                } else {
                    format!("{lemma}et")
                };
                (def, lemma.to_string(), format!("{bare}a"))
            }
        };
        Noun {
            lemma: lemma.to_string(),
            gender,
            standard,
            def,
            pl,
            def_pl,
            acronym: false,
        }
    }

    /// Norwegian determiner for the triple article.
    fn determiner(&self, plural: bool) -> &'static str {
        match (plural, self.gender, self.standard) {
            (true, _, Standard::Bokmal) => "de",
            (true, _, Standard::Nynorsk) => "dei",
            (false, Gender::Neuter, _) => "det",
            (false, _, _) => "den",
        }
    }

    /// Indefinite form: the bare noun (or plural, or dual).
    pub fn indefinite(&self, number: Number) -> String {
        match number {
            Number::Sg => self.lemma.clone(),
            Number::Pl => self.pl.clone(),
            Number::Dual if self.acronym => format!("{}-ān", self.lemma),
            Number::Dual => dual(&self.lemma),
        }
    }

    /// The Norwegian definite form alone, which a possessive follows in
    /// Norwegian (`namnet mitt`) and an Arabic possessive suffix replaces
    /// (`namnet-hā`).
    pub fn suffixed(&self, number: Number) -> String {
        match number {
            Number::Sg => self.def.clone(),
            Number::Pl => self.def_pl.clone(),
            Number::Dual => dual(&self.def_pl),
        }
    }

    /// Triple definiteness: Arabic `al-` (always assimilated to the `d` of
    /// the determiner, so `ad-`), the Norwegian determiner, and the Norwegian
    /// suffixed article.
    pub fn definite(&self, number: Number) -> String {
        match number {
            Number::Sg => format!("ad-{}-{}", self.determiner(false), self.def),
            Number::Pl => format!("ad-{}-{}", self.determiner(true), self.def_pl),
            Number::Dual => format!("ad-{}-{}", self.determiner(true), dual(&self.def_pl)),
        }
    }
}

/// The Arabic dual: an unstressed final `-e` drops, then `-ān`.
pub fn dual(word: &str) -> String {
    format!("{}ān", strip_final(word, 'e'))
}

/// Arabic sun letters (the article assimilates to them). Norwegian letters
/// with no Arabic counterpart (g, p, v, c, x) are moon letters.
const SUN: &[&str] = &[
    "th", "dh", "sh", "t", "d", "r", "z", "s", "l", "n", "ṣ", "ḍ", "ṭ", "ẓ",
];

/// `al-` before `word`, assimilated per the sun-letter rule. The first
/// consonant is read with the root digraph rule, so `skj-` counts as `sh`.
pub fn al(word: &str) -> String {
    let lower = word.to_lowercase();
    let first_is_consonant = lower
        .chars()
        .next()
        .is_some_and(|c| !crate::text::is_vowel(c));
    if first_is_consonant {
        if let Some(c) = consonants(&lower).first() {
            if SUN.contains(&c.as_str()) {
                return format!("a{c}-");
            }
        }
    }
    "al-".to_string()
}

/// An adjective in its selected standard.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Adjective {
    pub base: String,
    pub neuter: String,
    pub def: String,
    pub pl: String,
    /// Arabic participles used as adjectives do not inflect for gender.
    pub invariant: bool,
}

impl Adjective {
    pub fn from_entry(e: &Entry) -> Adjective {
        let forms = e.forms();
        if forms.len() == 3 {
            Adjective {
                base: e.lemma().to_string(),
                neuter: forms[0].clone(),
                def: forms[1].clone(),
                pl: forms[2].clone(),
                invariant: false,
            }
        } else {
            Adjective::regular(e.lemma())
        }
    }

    pub fn regular(base: &str) -> Adjective {
        let neuter = if ["ig", "isk", "sk", "t"].iter().any(|s| base.ends_with(s)) {
            base.to_string()
        } else {
            format!("{base}t")
        };
        let weak = if base.ends_with('e') {
            base.to_string()
        } else {
            format!("{base}e")
        };
        Adjective {
            base: base.to_string(),
            neuter,
            def: weak.clone(),
            pl: weak,
            invariant: false,
        }
    }

    /// A passive or active participle used attributively.
    pub fn participle(word: &str) -> Adjective {
        Adjective {
            base: word.to_string(),
            neuter: word.to_string(),
            def: format!("{word}e"),
            pl: word.to_string(),
            invariant: true,
        }
    }

    pub fn indefinite(&self, gender: Gender, number: Number) -> String {
        match number {
            Number::Dual => dual(&self.pl),
            Number::Pl => self.pl.clone(),
            Number::Sg if gender == Gender::Neuter && !self.invariant => self.neuter.clone(),
            Number::Sg => self.base.clone(),
        }
    }

    /// Definite: its own assimilated `al-` plus the Norwegian weak `-e`.
    pub fn definite(&self, number: Number) -> String {
        let form = if number == Number::Dual {
            dual(&self.def)
        } else {
            self.def.clone()
        };
        format!("{}{}", al(&form), form)
    }
}

/// A noun phrase: nouns in English compound order (modifiers first, head
/// last), adjectives, number and definiteness.
#[derive(Clone, Debug, Default)]
pub struct NounPhrase {
    pub nouns: Vec<Noun>,
    pub adjectives: Vec<Adjective>,
    pub number: Option<Number>,
    pub definite: bool,
    /// Gender to agree with when there is no noun (predicate adjectives).
    pub agree: Option<Gender>,
    /// A possessor that is not singular (`the names of the users`): its
    /// position in Arabic order and its number. Other possessors are
    /// singular, as compounds are.
    pub possessor_number: Option<(usize, Number)>,
    /// An Arabic possessive suffix on the last noun (`its name`), which
    /// then takes no article (§18.8).
    pub possessive: Option<String>,
}

impl NounPhrase {
    pub fn is_empty(&self) -> bool {
        self.nouns.is_empty() && self.adjectives.is_empty()
    }

    /// Render in Arabic order: head first, possessors after, only the last
    /// noun takes the triple article. Adjectives follow the whole construct.
    pub fn render(&self) -> Vec<String> {
        let number = self.number.unwrap_or(Number::Sg);
        let arabic: Vec<&Noun> = self.nouns.iter().rev().collect();
        let n = arabic.len();
        // A chain always ends in a definite possessor (§4.5).
        let definite = self.definite || n > 1;
        let mut out = Vec::new();
        for (i, noun) in arabic.iter().enumerate() {
            let is_head = i == 0;
            let is_last = i + 1 == n;
            let num = match self.possessor_number {
                _ if is_head => number,
                Some((at, n)) if at == i => n,
                _ => Number::Sg,
            };
            out.push(if let (true, Some(suffix)) = (is_last, &self.possessive) {
                format!("{}-{suffix}", noun.suffixed(num))
            } else if is_last && definite {
                noun.definite(num)
            } else {
                noun.indefinite(num)
            });
        }
        let head_gender = arabic
            .first()
            .map(|h| h.gender)
            .or(self.agree)
            .unwrap_or(Gender::Masculine);
        for adj in &self.adjectives {
            out.push(if definite {
                adj.definite(number)
            } else {
                adj.indefinite(head_gender, number)
            });
        }
        out
    }

    pub fn head_gender(&self) -> Gender {
        self.nouns
            .last()
            .map(|n| n.gender)
            .or(self.agree)
            .unwrap_or(Gender::Masculine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexicon::Lexicon;

    fn noun(key: &str) -> Noun {
        Noun::from_entry(Lexicon::core().by_key(key).unwrap())
    }

    fn adj(key: &str) -> Adjective {
        Adjective::from_entry(Lexicon::core().by_key(key).unwrap())
    }

    #[test]
    fn acronyms_take_hyphenated_endings() {
        let crc = Noun::acronym("CRC");
        assert_eq!(crc.standard, Standard::Bokmal);
        assert_eq!(crc.definite(Number::Sg), "ad-det-CRC-et");
        assert_eq!(crc.indefinite(Number::Pl), "CRC-er");
        assert_eq!(crc.indefinite(Number::Dual), "CRC-ān");
        let json = Noun::acronym("JSON");
        assert_eq!(json.standard, Standard::Nynorsk);
        assert_eq!(json.indefinite(Number::Pl), "JSON");
        assert_eq!(json.definite(Number::Pl), "ad-dei-JSON-a");
        assert_eq!(Noun::acronym("gzip").indefinite(Number::Dual), "gzip-ān");
    }

    #[test]
    fn triple_definiteness() {
        assert_eq!(noun("house").definite(Number::Sg), "ad-det-huset");
        assert_eq!(noun("user").definite(Number::Sg), "ad-den-brukaren");
        assert_eq!(noun("user").definite(Number::Pl), "ad-dei-brukarane");
        assert_eq!(noun("house").definite(Number::Pl), "ad-de-husa");
    }

    #[test]
    fn feminine_is_mandatory_where_allowed() {
        assert_eq!(noun("file").definite(Number::Sg), "ad-den-fila");
        assert_eq!(noun("book").definite(Number::Sg), "ad-den-boka");
    }

    #[test]
    fn indefinite_is_bare_and_dual_adds_an() {
        assert_eq!(noun("user").indefinite(Number::Sg), "brukar");
        assert_eq!(noun("user").indefinite(Number::Pl), "brukarar");
        assert_eq!(noun("user").indefinite(Number::Dual), "brukarān");
        assert_eq!(noun("page").indefinite(Number::Dual), "sidān");
    }

    #[test]
    fn sun_and_moon_letters() {
        assert_eq!(al("store"), "as-");
        assert_eq!(al("gyldige"), "al-");
        assert_eq!(al("nye"), "an-");
        assert_eq!(al("skjønne"), "ash-");
        assert_eq!(al("kjære"), "al-");
        assert_eq!(al("åpne"), "al-");
        assert_eq!(al("tomme"), "at-");
    }

    #[test]
    fn adjective_follows_and_agrees() {
        let np = NounPhrase {
            nouns: vec![noun("house")],
            adjectives: vec![adj("big")],
            number: None,
            definite: true,
            agree: None,
            possessor_number: None,
            possessive: None,
        };
        assert_eq!(np.render().join(" "), "ad-det-huset as-store");
        let indef = NounPhrase {
            definite: false,
            ..np
        };
        assert_eq!(indef.render().join(" "), "hus stort");
    }

    #[test]
    fn idafa_puts_head_first_and_only_last_is_definite() {
        let np = NounPhrase {
            nouns: vec![noun("user"), noun("name")],
            ..NounPhrase::default()
        };
        assert_eq!(np.render().join(" "), "namn ad-den-brukaren");
        let chain = NounPhrase {
            nouns: vec![noun("user"), noun("account"), noun("name")],
            ..NounPhrase::default()
        };
        assert_eq!(chain.render().join(" "), "namn konto ad-den-brukaren");
    }

    #[test]
    fn regular_fallback() {
        let n = Noun::regular("dings", Gender::Masculine, Standard::Nynorsk);
        assert_eq!((n.def.as_str(), n.pl.as_str()), ("dingsen", "dingsar"));
        let d = Noun::derived("tālik");
        assert_eq!(d.gender, Gender::Masculine);
        assert_eq!(d.def, "tāliken");
    }
}
