//! Turning English words into grammatical tokens, shared by `ident` and
//! `prose`.

use serde::Serialize;

use crate::english::Inflection;
use crate::lexicon::{Entry, Lexicon, Pos, Sense};
use crate::nominal::{Adjective, Noun};
use crate::segment::Via;
use crate::text::{Number, Standard};
use crate::verb::Verb;

/// Where a word sits, which decides between its senses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Context {
    /// Function names start with a verb.
    VerbFirst,
    /// Inside a noun phrase.
    NounPhrase,
    /// After `is`/`has` in a boolean: adjectives and participles first.
    Predicate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Quant {
    All,
    Each,
    Some,
    Any,
}

#[derive(Clone, Debug)]
pub enum Tok<'a> {
    Verb(&'a Entry, Verb, Inflection),
    Noun(Noun, Number),
    Adj(Adjective),
    Adv(String),
    Part(String),
    Quant(String, Quant),
    Num(u64),
    Of,
    Not,
    Copula,
    Has,
    Def,
    Indef,
    Dual,
    Pass(String),
    /// Object pronoun: the Arabic suffix for the verb, the Norwegian word.
    Pron {
        suffix: String,
        object: String,
    },
}

/// What a word turned into, for `--json` output.
#[derive(Clone, Debug, Serialize)]
pub struct WordNote {
    pub english: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lemma: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub standard: Option<Standard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub proposed: bool,
    /// The identifier word this one was read from (`fh` for `file`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// How it was read: as an abbreviation or as part of a compound.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub via: Option<Via>,
}

impl WordNote {
    fn function_word(english: &str, kind: &str) -> WordNote {
        WordNote {
            english: english.to_string(),
            kind: kind.to_string(),
            key: None,
            lemma: None,
            standard: None,
            root: None,
            proposed: false,
            from: None,
            via: None,
        }
    }

    fn acronym(english: &str, noun: &Noun) -> WordNote {
        WordNote {
            lemma: Some(noun.lemma.clone()),
            standard: Some(noun.standard),
            ..WordNote::function_word(english, "acronym")
        }
    }

    fn from_sense(english: &str, kind: &str, e: &Entry) -> WordNote {
        let arabic = matches!(e.pos, Pos::Part | Pos::Det);
        WordNote {
            english: english.to_string(),
            kind: kind.to_string(),
            key: Some(e.key.clone()),
            lemma: Some(if arabic {
                e.ar.clone()
            } else {
                e.lemma().to_string()
            }),
            standard: if arabic { None } else { Some(e.standard()) },
            root: (e.pos == Pos::Verb).then(|| e.root().display()),
            proposed: e.proposed,
            from: None,
            via: None,
        }
    }
}

fn function_word(word: &str) -> Option<Tok<'static>> {
    Some(match word {
        "of" => Tok::Of,
        "not" | "no" | "non" => Tok::Not,
        "is" | "are" | "was" | "were" | "be" | "been" | "being" | "am" => Tok::Copula,
        "has" | "have" | "had" | "having" => Tok::Has,
        "the" => Tok::Def,
        "a" | "an" => Tok::Indef,
        "pair" | "pairs" | "both" => Tok::Dual,
        _ => return None,
    })
}

/// `of`, `not`, `is`, `has`, the articles and `pair`: words the grammar
/// handles itself.
pub fn is_function_word(word: &str) -> bool {
    function_word(word).is_some()
}

fn rank(sense: &Sense<'_>, ctx: Context) -> Option<u8> {
    use Inflection::*;
    let verbish = matches!(sense.inflection, Base | ThirdPerson);
    let r = match (sense.entry.pos, sense.inflection, ctx) {
        (Pos::Verb, Base | ThirdPerson, Context::VerbFirst) => 0,
        (Pos::Adj, _, Context::Predicate) => 0,
        (Pos::Verb, PastParticiple, Context::Predicate) => 1,
        (Pos::Verb, Base | ThirdPerson, Context::Predicate) => 2,
        (Pos::Noun, Base | Plural, _) => 3,
        (Pos::Adj, Base, _) => 4,
        (Pos::Num | Pos::Part | Pos::Det | Pos::Pron, Base, _) => 5,
        (Pos::Verb, Agent, _) => 6,
        (Pos::Verb, PastParticiple, _) => 7,
        (Pos::Verb, Gerund, _) => 8,
        (Pos::Adv, Base, _) => 9,
        (Pos::Verb, _, _) if verbish => 10,
        _ => return None,
    };
    Some(r)
}

/// Pick the best sense of `word` for `ctx` and turn it into a token.
pub fn classify<'a>(lex: &'a Lexicon, word: &str, ctx: Context) -> Option<(Tok<'a>, WordNote)> {
    let found = classify_raw(lex, word, ctx)?;
    // Inside a noun phrase a finite verb (`retry_count`, `max_retries`) is
    // read as its verbal noun.
    if ctx == Context::NounPhrase {
        if let (Tok::Verb(_, verb, infl), note) = &found {
            let number = if *infl == Inflection::ThirdPerson {
                Number::Pl
            } else {
                Number::Sg
            };
            let mut note = note.clone();
            note.kind = "verbal noun".into();
            return Some((Tok::Noun(Noun::derived(&verb.verbal_noun()), number), note));
        }
    }
    Some(found)
}

fn classify_raw<'a>(lex: &'a Lexicon, word: &str, ctx: Context) -> Option<(Tok<'a>, WordNote)> {
    if let Some(tok) = function_word(word) {
        return Some((tok, WordNote::function_word(word, "function")));
    }
    if word.chars().all(|c| c.is_ascii_digit()) {
        if let Ok(v) = word.parse() {
            return Some((Tok::Num(v), WordNote::function_word(word, "number")));
        }
    }
    let senses = lex.senses(word);
    let best = senses
        .iter()
        .filter_map(|s| rank(s, ctx).map(|r| (r, s)))
        .min_by_key(|(r, _)| *r)
        .map(|(_, s)| s.clone());
    match best {
        Some(sense) => make(sense, word).ok(),
        None => acronym(lex, word),
    }
}

/// An acronym (`http`, plural `urls`) as a neuter noun kept as written.
fn acronym<'a>(lex: &Lexicon, word: &str) -> Option<(Tok<'a>, WordNote)> {
    let w = word.to_lowercase();
    let (written, number) = match lex.acronym(&w) {
        Some(written) => (written, Number::Sg),
        None => (lex.acronym(w.strip_suffix('s')?)?, Number::Pl),
    };
    let noun = Noun::acronym(written);
    let note = WordNote::acronym(word, &noun);
    Some((Tok::Noun(noun, number), note))
}

fn make<'a>(sense: Sense<'a>, word: &str) -> Result<(Tok<'a>, WordNote), String> {
    let e = sense.entry;
    let note = |kind: &str| WordNote::from_sense(word, kind, e);
    Ok(match (e.pos, sense.inflection) {
        (Pos::Noun, Inflection::Plural) => {
            (Tok::Noun(Noun::from_entry(e), Number::Pl), note("noun"))
        }
        (Pos::Noun, _) => (Tok::Noun(Noun::from_entry(e), Number::Sg), note("noun")),
        (Pos::Adj, _) => (Tok::Adj(Adjective::from_entry(e)), note("adjective")),
        (Pos::Adv, _) => (Tok::Adv(e.lemma().to_string()), note("adverb")),
        (Pos::Part, _) => (Tok::Part(e.ar.clone()), note("particle")),
        (Pos::Det, _) => {
            let q = match e.english.as_str() {
                "all" => Quant::All,
                "some" => Quant::Some,
                "any" => Quant::Any,
                _ => Quant::Each,
            };
            (Tok::Quant(e.ar.clone(), q), note("quantifier"))
        }
        (Pos::Num, _) => (Tok::Num(e.value.unwrap_or(0)), note("numeral")),
        (Pos::Pron, _) => (
            Tok::Pron {
                suffix: e.ar.clone(),
                object: e
                    .forms()
                    .first()
                    .cloned()
                    .unwrap_or_else(|| e.lemma().to_string()),
            },
            note("object pronoun"),
        ),
        (Pos::Verb, infl) => {
            let verb = e.verb()?;
            match infl {
                Inflection::Agent => (
                    Tok::Noun(Noun::derived(&verb.active_participle()), Number::Sg),
                    note("active participle"),
                ),
                Inflection::PastParticiple => (
                    Tok::Adj(Adjective::participle(&verb.passive_participle())),
                    note("passive participle"),
                ),
                Inflection::Gerund => (
                    Tok::Noun(Noun::derived(&verb.verbal_noun()), Number::Sg),
                    note("verbal noun"),
                ),
                _ => (Tok::Verb(e, verb, infl), note("verb")),
            }
        }
    })
}

/// Like [`classify`], but a word that only makes sense as a plural agent
/// (`parsers`) is retried as its singular.
pub fn classify_word<'a>(
    lex: &'a Lexicon,
    word: &str,
    ctx: Context,
) -> Option<(Tok<'a>, WordNote)> {
    if let Some(found) = classify(lex, word, ctx) {
        return Some(found);
    }
    let singular = word.strip_suffix('s')?;
    match classify(lex, singular, ctx)? {
        (Tok::Noun(n, _), note) => Some((Tok::Noun(n, Number::Pl), note)),
        _ => None,
    }
}

/// The Norwegian negator, selected by the parity rule (`ikke` has four
/// letters, so it is always Nynorsk `ikkje`).
pub fn negator() -> &'static str {
    match Standard::for_lemma("ikke") {
        Standard::Bokmal => "ikke",
        Standard::Nynorsk => "ikkje",
    }
}

/// Words that pass through untranslated: single letters and anything with
/// a digit in it (`x`, `utf8`, `user2`).
pub fn passes_through(word: &str) -> bool {
    word.chars().count() == 1
        || (word.chars().any(|c| c.is_ascii_digit()) && !word.chars().all(|c| c.is_ascii_digit()))
}
