//! Template-based translation of simple English sentences (SPEC §4.9–4.11).
//!
//! Supported shapes, each optionally negated:
//!   SUBJECT VERB [OBJECT] [PREP NP]* [ADVERB]*      present or past
//!   SUBJECT is/are/was/were PREDICATE                verbless sentence
//!   SUBJECT has/have OBJECT                          possession with ʿinda
//!   Does/Did SUBJECT VERB …? / Is SUBJECT PREDICATE? yes/no questions
//! Anything else is rejected rather than guessed.

use serde::Serialize;

use crate::classify::{classify_word, negator, Context, Quant, Tok};
use crate::english::Inflection;
use crate::lexicon::{Lexicon, Pos};
use crate::nominal::NounPhrase;
use crate::numbers::render_counted;
use crate::text::{Gender, Number};
use crate::verb::{Person, Verb};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ProseError {
    Unknown { words: Vec<String> },
    Unsupported { sentence: String, reason: String },
}

impl std::fmt::Display for ProseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProseError::Unknown { words } => write!(f, "not in the lexicon: {}", words.join(", ")),
            ProseError::Unsupported { sentence, reason } => {
                write!(
                    f,
                    "cannot translate \"{sentence}\": {reason}. Simplify the sentence."
                )
            }
        }
    }
}

const AUXILIARIES: &[&str] = &[
    "does", "do", "did", "is", "are", "was", "were", "has", "have", "had", "not",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tense {
    Present,
    Past,
}

/// A parsed noun phrase plus its counting and quantifier.
#[derive(Default)]
struct Np {
    phrase: NounPhrase,
    count: Option<u64>,
    quant: Option<String>,
}

impl Np {
    fn render(self, lex: &Lexicon) -> Vec<String> {
        let mut out: Vec<String> = self.quant.into_iter().collect();
        out.extend(render_counted(lex, self.phrase, self.count));
        out
    }

    fn gender(&self) -> Gender {
        self.phrase.head_gender()
    }
}

struct Parser<'a> {
    lex: &'a Lexicon,
    words: Vec<String>,
    i: usize,
    sentence: String,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&str> {
        self.words.get(self.i).map(String::as_str)
    }

    fn eat(&mut self, options: &[&str]) -> Option<String> {
        let w = self.peek()?.to_string();
        if options.contains(&w.as_str()) {
            self.i += 1;
            return Some(w);
        }
        None
    }

    fn unsupported(&self, reason: &str) -> ProseError {
        ProseError::Unsupported {
            sentence: self.sentence.clone(),
            reason: reason.to_string(),
        }
    }

    fn unknown_or(&self, reason: &str) -> ProseError {
        match self.peek() {
            Some(w)
                if self.lex.senses(w).is_empty()
                    && classify_word(self.lex, w, Context::NounPhrase).is_none()
                    && !AUXILIARIES.contains(&w) =>
            {
                ProseError::Unknown {
                    words: vec![w.to_string()],
                }
            }
            _ => self.unsupported(reason),
        }
    }

    /// A word that can be a finite verb.
    fn verb_sense(&self, word: &str) -> Option<Verb> {
        self.lex
            .senses(word)
            .into_iter()
            .find(|s| {
                s.entry.pos == Pos::Verb
                    && matches!(
                        s.inflection,
                        Inflection::Base | Inflection::ThirdPerson | Inflection::PastParticiple
                    )
            })
            .and_then(|s| s.entry.verb().ok())
    }

    /// A word that can only be a finite verb (`reads`), never a noun.
    fn finite_only(&self, word: &str) -> bool {
        let senses = self.lex.senses(word);
        senses.iter().any(|s| {
            s.entry.pos == Pos::Verb
                && matches!(s.inflection, Inflection::Base | Inflection::ThirdPerson)
        }) && senses.iter().all(|s| s.entry.pos == Pos::Verb)
    }

    /// A finite verb and its tense. An ambiguous form (`read`) is present.
    fn verb_tense(&self, word: &str) -> Option<(Verb, Tense)> {
        let senses = self.lex.senses(word);
        let verb = |infl: &[Inflection]| {
            senses
                .iter()
                .find(|s| s.entry.pos == Pos::Verb && infl.contains(&s.inflection))
                .cloned()
        };
        let sense = verb(&[Inflection::Base, Inflection::ThirdPerson])
            .or_else(|| verb(&[Inflection::PastParticiple]))?;
        let tense = if sense.inflection == Inflection::PastParticiple {
            Tense::Past
        } else {
            Tense::Present
        };
        Some((sense.entry.verb().ok()?, tense))
    }

    /// Parse a noun phrase. With `stop_at_verb`, a word that can be a verb
    /// ends the phrase once it has a noun (`the user writes`).
    fn noun_phrase(&mut self, stop_at_verb: bool) -> Result<Option<Np>, ProseError> {
        let mut np = Np::default();
        let mut of: Option<NounPhrase> = None;
        let mut any = false;
        while let Some(w) = self.peek().map(str::to_string) {
            if AUXILIARIES.contains(&w.as_str()) {
                break;
            }
            if stop_at_verb && !np.phrase.nouns.is_empty() && self.verb_sense(&w).is_some() {
                break;
            }
            if self.finite_only(&w) {
                break;
            }
            let Some((tok, _)) = classify_word(self.lex, &w, Context::NounPhrase) else {
                if self.lex.senses(&w).is_empty() {
                    return Err(ProseError::Unknown { words: vec![w] });
                }
                break;
            };
            match tok {
                Tok::Def => np.phrase.definite = true,
                Tok::Indef => np.phrase.definite = false,
                Tok::Num(n) => np.count = Some(n),
                Tok::Dual => np.phrase.number = Some(Number::Dual),
                // English adjectives precede the noun; one after it is a
                // predicate and belongs to the caller.
                Tok::Adj(_) if !np.phrase.nouns.is_empty() => break,
                Tok::Adj(a) => np.phrase.adjectives.push(a),
                Tok::Noun(n, num) => {
                    np.phrase.nouns.push(n);
                    if np.phrase.number != Some(Number::Dual) {
                        np.phrase.number = Some(num);
                    }
                }
                Tok::Quant(ar, q) => {
                    np.quant = Some(ar);
                    np.phrase.definite = matches!(q, Quant::All | Quant::Some);
                    np.phrase.number = Some(if np.phrase.definite {
                        Number::Pl
                    } else {
                        Number::Sg
                    });
                }
                Tok::Of if !np.phrase.nouns.is_empty() => {
                    of = Some(std::mem::take(&mut np.phrase));
                }
                _ => break,
            }
            any = true;
            self.i += 1;
        }
        if let Some(head) = of {
            let mut nouns = std::mem::take(&mut np.phrase.nouns);
            nouns.extend(head.nouns);
            np.phrase = NounPhrase {
                nouns,
                adjectives: head.adjectives,
                number: head.number,
                definite: head.definite,
                agree: None,
            };
        }
        Ok(any.then_some(np))
    }

    /// Prepositional phrases and adverbs after the core of the clause.
    fn tail(&mut self) -> Result<Vec<String>, ProseError> {
        let mut out = Vec::new();
        let mut adverbs = Vec::new();
        while let Some(w) = self.peek().map(str::to_string) {
            match classify_word(self.lex, &w, Context::NounPhrase) {
                Some((Tok::Part(ar), _)) => {
                    self.i += 1;
                    out.push(ar);
                    let np = self
                        .noun_phrase(false)?
                        .ok_or_else(|| self.unknown_or("a preposition needs a noun after it"))?;
                    out.extend(np.render(self.lex));
                }
                Some((Tok::Adv(a), _)) => {
                    self.i += 1;
                    adverbs.push(a);
                }
                _ => return Err(self.unknown_or("unexpected word")),
            }
        }
        out.extend(adverbs);
        Ok(out)
    }
}

/// Norwegian pronoun after the verb (Norwegian needs a subject) and the
/// Arabic person on the verb (Arabic needs agreement). Both are satisfied.
fn pronoun(word: &str) -> Option<(Person, &'static str)> {
    Some(match word {
        "i" => (Person::First, "jeg"),
        "you" => (Person::Second, "du"),
        "he" => (Person::ThirdMasc, "han"),
        "she" => (Person::ThirdFem, "hun"),
        "it" => (Person::ThirdFem, "det"),
        "we" => (Person::FirstPlural, "vi"),
        "they" => (Person::ThirdPlural, "dei"),
        _ => return None,
    })
}

fn expand_contractions(sentence: &str) -> String {
    let mut s = format!(" {} ", sentence.to_lowercase());
    for (from, to) in [
        ("doesn't", "does not"),
        ("don't", "do not"),
        ("didn't", "did not"),
        ("isn't", "is not"),
        ("aren't", "are not"),
        ("wasn't", "was not"),
        ("weren't", "were not"),
        ("hasn't", "has not"),
        ("haven't", "have not"),
    ] {
        s = s.replace(from, to);
    }
    s
}

fn sentence(lex: &Lexicon, raw: &str) -> Result<String, ProseError> {
    let trimmed = raw.trim();
    let terminal = trimmed
        .chars()
        .last()
        .filter(|c| matches!(c, '.' | '?' | '!'));
    let words: Vec<String> = expand_contractions(trimmed)
        .split_whitespace()
        .map(|w| {
            w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
                .to_string()
        })
        .filter(|w| !w.is_empty())
        .collect();
    let mut p = Parser {
        lex,
        words,
        i: 0,
        sentence: trimmed.to_string(),
    };
    let question_aux = p.eat(&["does", "do", "did", "is", "are", "was", "were"]);
    let question = question_aux.is_some();

    // Subject.
    let subject_pronoun = p.peek().and_then(pronoun);
    let (subject, person, agree) = if let Some((person, word)) = subject_pronoun {
        p.i += 1;
        let agree = match person {
            Person::ThirdFem => (Gender::Feminine, Number::Sg),
            Person::FirstPlural | Person::ThirdPlural => (Gender::Masculine, Number::Pl),
            _ => (Gender::Masculine, Number::Sg),
        };
        (vec![word.to_string()], person, agree)
    } else {
        let np = p
            .noun_phrase(true)?
            .ok_or_else(|| p.unknown_or("no subject found"))?;
        let person = Person::third(np.gender());
        let agree = (np.gender(), np.phrase.number.unwrap_or(Number::Sg));
        (np.render(lex), person, agree)
    };

    // Auxiliaries and negation.
    let mut aux = question_aux;
    if aux.is_none() {
        aux = p.eat(&[
            "does", "do", "did", "is", "are", "was", "were", "has", "have", "had",
        ]);
    }
    let negate = p.eat(&["not"]).is_some();
    let lead = if question { "hal" } else { "fa" };
    let mut out = vec![lead.to_string()];

    match aux.as_deref() {
        Some("is" | "are" | "was" | "were") => {
            // Verbless sentence: no present copula. The null copula sits in
            // second position, so a negation wraps nothing at all.
            if negate {
                out.extend(["mā".to_string(), negator().to_string()]);
            }
            let mut predicate = p
                .noun_phrase(false)?
                .ok_or_else(|| p.unknown_or("no predicate"))?;
            if predicate.phrase.nouns.is_empty() {
                // Predicate adjectives agree with the subject, Norwegian style.
                predicate.phrase.agree = Some(agree.0);
                predicate.phrase.number = Some(agree.1);
            }
            out.extend(subject);
            out.extend(predicate.render(lex));
        }
        Some("has" | "have" | "had") => {
            if negate {
                out.extend(["mā".to_string(), negator().to_string()]);
            }
            out.push("ʿinda".into());
            out.extend(subject);
            if let Some(obj) = p.noun_phrase(false)? {
                out.extend(obj.render(lex));
            }
        }
        _ => {
            let word = p
                .peek()
                .map(str::to_string)
                .ok_or_else(|| p.unsupported("no verb found"))?;
            let (verb, mut tense) = p
                .verb_tense(&word)
                .ok_or_else(|| p.unknown_or("expected a verb"))?;
            p.i += 1;
            if aux.as_deref() == Some("did") {
                tense = Tense::Past;
            }
            let mut form = match tense {
                Tense::Present => verb.present(person),
                Tense::Past => verb.past(person),
            };
            // An object pronoun is suffixed to the verb (Arabic) and also
            // written as a word in object position (Norwegian).
            let pronoun_object = match p
                .peek()
                .and_then(|w| classify_word(lex, w, Context::NounPhrase))
            {
                Some((Tok::Pron { suffix, object }, _)) => {
                    p.i += 1;
                    form = format!("{form}-{suffix}");
                    Some(object)
                }
                _ => None,
            };
            if negate {
                out.extend(["mā".to_string(), form, negator().to_string()]);
            } else {
                out.push(form);
            }
            out.extend(subject);
            if let Some(object) = pronoun_object {
                out.push(object);
            } else if let Some(obj) = p.noun_phrase(false)? {
                out.extend(obj.render(lex));
            }
        }
    }
    out.extend(p.tail()?);
    let end = if question { Some('?') } else { terminal };
    // Norwegian and ALA-LC romanization agree: a sentence starts with a
    // capital letter.
    let joined = out.join(" ");
    let mut chars = joined.chars();
    let mut s: String = match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => joined,
    };
    if let Some(c) = end {
        s.push(c);
    }
    Ok(s)
}

/// Translate one or more simple English sentences.
pub fn translate(lex: &Lexicon, text: &str) -> Result<String, ProseError> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        current.push(c);
        if matches!(c, '.' | '?' | '!') {
            sentences.push(std::mem::take(&mut current));
        }
    }
    if !current.trim().is_empty() {
        sentences.push(current);
    }
    let mut out = Vec::new();
    for s in sentences.iter().filter(|s| !s.trim().is_empty()) {
        out.push(sentence(lex, s)?);
    }
    Ok(out.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> String {
        translate(Lexicon::core(), s).unwrap()
    }

    #[test]
    fn verb_first_after_fa() {
        assert_eq!(
            t("The user writes a file."),
            "Fa yusakrivur ad-den-brukaren fil."
        );
    }

    #[test]
    fn negation_is_a_circumfix() {
        assert_eq!(
            t("The user does not write a file."),
            "Fa mā yusakrivur ikkje ad-den-brukaren fil."
        );
    }

    #[test]
    fn neuter_subjects_take_feminine_agreement() {
        assert_eq!(t("The house reads."), "Fa talsuwur ad-det-huset.");
    }

    #[test]
    fn past_tense() {
        assert_eq!(
            t("The user wrote a file."),
            "Fa sakrava ad-den-brukaren fil."
        );
    }

    #[test]
    fn questions_start_with_hal() {
        assert_eq!(
            t("Does the user write a file?"),
            "Hal yusakrivur ad-den-brukaren fil?"
        );
        assert_eq!(t("Is the file empty?"), "Hal ad-den-fila tom?");
    }

    #[test]
    fn verbless_sentences_have_no_copula() {
        assert_eq!(t("The user is valid."), "Fa ad-den-brukaren gyldig.");
        assert_eq!(
            t("The user is not valid."),
            "Fa mā ikkje ad-den-brukaren gyldig."
        );
        assert_eq!(t("The house is big."), "Fa ad-det-huset stort.");
        assert_eq!(t("The users are valid."), "Fa ad-dei-brukarane gyldige.");
    }

    #[test]
    fn counting_follows_arabic() {
        assert_eq!(
            t("The user writes 2 files."),
            "Fa yusakrivur ad-den-brukaren filān."
        );
        assert_eq!(
            t("The user writes 22 files."),
            "Fa yusakrivur ad-den-brukaren to wa tjue fil."
        );
        assert_eq!(
            t("The user writes 3 files."),
            "Fa yusakrivur ad-den-brukaren tre filer."
        );
    }

    #[test]
    fn pronouns_keep_both_languages_happy() {
        assert_eq!(t("We read the file."), "Fa nalsuwur vi ad-den-fila.");
    }

    #[test]
    fn object_pronouns_are_marked_twice() {
        assert_eq!(
            t("The user reads me."),
            "Fa yalsuwur-nī ad-den-brukaren meg."
        );
        assert_eq!(t("We help them."), "Fa nuhajlipur-hum vi dei.");
    }

    #[test]
    fn prepositions() {
        assert_eq!(
            t("The user writes the name to the file."),
            "Fa yusakrivur ad-den-brukaren ad-det-namnet ilā ad-den-fila."
        );
    }

    #[test]
    fn unsupported_shapes_are_refused_not_guessed() {
        for s in ["The user writes and reads a file.", "Read the file."] {
            assert!(
                matches!(
                    translate(Lexicon::core(), s),
                    Err(ProseError::Unsupported { .. })
                ),
                "{s}"
            );
        }
    }

    #[test]
    fn unknown_words_fail_loudly() {
        assert!(matches!(
            translate(Lexicon::core(), "The wombat writes."),
            Err(ProseError::Unknown { .. })
        ));
    }
}
