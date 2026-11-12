//! The word-level parser: verbs and prepositional tails. Noun phrases live
//! in `noun.rs`, the clause rules that decide word order in `clause.rs` and
//! `predicate.rs`, and verbs used as nouns and adjectives in `verbal.rs`.

use crate::classify::{classify_word, Context, Tok};
use crate::english::Inflection;
use crate::lexicon::{Lexicon, Pos, Sense};
use crate::text::Standard;
use crate::verb::Verb;

use super::noun::Demonstrative;
use super::ProseError;

/// Words that end a noun phrase because a verb group starts there.
pub(super) const AUXILIARIES: &[&str] = &[
    "does", "do", "did", "is", "are", "was", "were", "has", "have", "had", "not", "will", "shall",
    "may", "might",
];

/// Conjunctions that open a subordinate clause, with their Arabic.
pub(super) const SUBORDINATORS: &[(&str, &str)] = &[
    ("if", "idhā"),
    ("unless", "illā idhā"),
    ("when", "ʿindamā"),
    ("while", "baynamā"),
    ("because", "li-anna"),
];

/// Words the grammar reads itself, which are never missing from the
/// lexicon even where a sentence cannot use them.
pub(super) const GRAMMAR_WORDS: &[&str] = &[
    "'s", "i", "you", "he", "she", "it", "we", "they", "my", "your", "his", "its", "our", "their",
    "there", "so", "whether", "then",
];

/// Words that open a relative clause after a noun.
pub(super) const RELATIVES: &[&str] = &["that", "which", "who"];

/// A comma, a subordinating conjunction or a relative pronoun: the end of
/// whatever phrase is being read.
pub(super) fn boundary(word: &str) -> bool {
    word == "," || RELATIVES.contains(&word) || SUBORDINATORS.iter().any(|(e, _)| *e == word)
}

/// The words that can start a noun phrase without being nouns themselves.
pub(super) fn determiner(word: &str) -> bool {
    matches!(
        word,
        "the"
            | "a"
            | "an"
            | "this"
            | "these"
            | "those"
            | "all"
            | "each"
            | "every"
            | "some"
            | "any"
            | "no"
    )
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Tense {
    Present,
    Past,
}

/// A verb found in the sentence, with what its English form said.
pub(super) struct Found {
    pub verb: Verb,
    pub tense: Tense,
    /// The standard of the Norwegian lemma, for the s-passive ending.
    pub standard: Standard,
    /// The English lexicon word, which tells the modals apart.
    pub english: String,
}

pub(super) struct Parser<'a> {
    pub lex: &'a Lexicon,
    words: Vec<String>,
    pub i: usize,
    sentence: String,
}

impl<'a> Parser<'a> {
    pub fn new(lex: &'a Lexicon, words: Vec<String>, sentence: &str) -> Self {
        Parser {
            lex,
            words,
            i: 0,
            sentence: sentence.to_string(),
        }
    }

    pub fn peek(&self) -> Option<&str> {
        self.peek_at(0)
    }

    pub fn peek_at(&self, n: usize) -> Option<&str> {
        self.words.get(self.i + n).map(String::as_str)
    }

    pub fn eat(&mut self, options: &[&str]) -> Option<String> {
        let w = self.peek()?.to_string();
        if options.contains(&w.as_str()) {
            self.i += 1;
            return Some(w);
        }
        None
    }

    /// Eat a subordinating conjunction and return its English and Arabic.
    pub fn subordinator(&mut self) -> Option<(&'static str, &'static str)> {
        let w = self.peek()?;
        let found = SUBORDINATORS.iter().find(|(e, _)| *e == w)?;
        self.i += 1;
        Some(*found)
    }

    pub fn unsupported(&self, reason: &str) -> ProseError {
        ProseError::Unsupported {
            sentence: self.sentence.clone(),
            reason: reason.to_string(),
        }
    }

    /// An unknown-word error if the current word is not in the lexicon,
    /// otherwise an unsupported-shape error with `reason`.
    pub fn unknown_or(&self, reason: &str) -> ProseError {
        match self.peek() {
            Some(w)
                if self.lex.senses(w).is_empty()
                    && classify_word(self.lex, w, Context::NounPhrase).is_none()
                    && !AUXILIARIES.contains(&w)
                    && !boundary(w)
                    && !GRAMMAR_WORDS.contains(&w)
                    && Demonstrative::parse(w).is_none() =>
            {
                ProseError::Unknown {
                    words: vec![w.to_string()],
                }
            }
            _ => self.unsupported(reason),
        }
    }

    pub fn verb_with(&self, word: &str, inflections: &[Inflection]) -> Option<(Sense<'a>, Verb)> {
        self.lex.senses(word).into_iter().find_map(|s| {
            if s.entry.pos == Pos::Verb && inflections.contains(&s.inflection) {
                let verb = s.entry.verb().ok()?;
                Some((s, verb))
            } else {
                None
            }
        })
    }

    pub fn found(sense: &Sense<'_>, verb: Verb, tense: Tense) -> Found {
        Found {
            verb,
            tense,
            standard: sense.entry.standard(),
            english: sense.entry.english.clone(),
        }
    }

    /// The Arabic of a preposition (not a conjunction) at `word`.
    pub fn preposition_at(&self, word: &str) -> Option<String> {
        match classify_word(self.lex, word, Context::NounPhrase) {
            Some((Tok::Part(ar), _)) if !matches!(ar.as_str(), "wa" | "aw" | "lākin") => Some(ar),
            _ => None,
        }
    }

    /// Eat a preposition and return its Arabic.
    pub fn preposition(&mut self) -> Option<String> {
        if self.peek() == Some("to") && self.infinitive_at(0).is_some() {
            return None;
        }
        let ar = self.preposition_at(self.peek()?)?;
        self.i += 1;
        Some(ar)
    }

    /// `and`/`or`/`but` followed by a verb: a second clause, not a second
    /// noun.
    pub fn joins_verbs(&self) -> bool {
        matches!(self.peek(), Some("and" | "or" | "but"))
            && self
                .peek_at(1)
                .is_some_and(|w| AUXILIARIES.contains(&w) || self.verb_tense(w).is_some())
    }

    /// A word that can be a finite verb.
    pub fn verb_sense(&self, word: &str) -> bool {
        self.verb_with(
            word,
            &[
                Inflection::Base,
                Inflection::ThirdPerson,
                Inflection::PastParticiple,
            ],
        )
        .is_some()
    }

    /// A word that can only be a finite verb (`reads`), never a noun.
    pub fn finite_only(&self, word: &str) -> bool {
        let senses = self.lex.senses(word);
        senses.iter().any(|s| {
            s.entry.pos == Pos::Verb
                && matches!(s.inflection, Inflection::Base | Inflection::ThirdPerson)
        }) && senses.iter().all(|s| s.entry.pos == Pos::Verb)
    }

    /// A finite verb and its tense. An ambiguous form (`read`) is present.
    pub fn verb_tense(&self, word: &str) -> Option<Found> {
        if let Some((s, v)) = self.verb_with(word, &[Inflection::Base, Inflection::ThirdPerson]) {
            return Some(Self::found(&s, v, Tense::Present));
        }
        let (s, v) = self.verb_with(word, &[Inflection::PastParticiple])?;
        Some(Self::found(&s, v, Tense::Past))
    }

    /// A verb in its base form, as a command starts (`Return …`).
    pub fn base_verb(&self, word: &str) -> Option<Found> {
        let (s, v) = self.verb_with(word, &[Inflection::Base])?;
        Some(Self::found(&s, v, Tense::Present))
    }

    /// A third-person verb, as a docstring starts (`Returns …`).
    pub fn third_person_verb(&self, word: &str) -> Option<Found> {
        let (s, v) = self.verb_with(word, &[Inflection::ThirdPerson])?;
        Some(Self::found(&s, v, Tense::Present))
    }

    /// A past participle that is not also an adjective: `deleted` makes a
    /// passive, `open` stays an adjective.
    pub fn participle(&self, word: &str) -> Option<Found> {
        let adjective = self
            .lex
            .senses(word)
            .iter()
            .any(|s| s.entry.pos == Pos::Adj);
        if adjective {
            return None;
        }
        let (s, v) = self.verb_with(word, &[Inflection::PastParticiple])?;
        Some(Self::found(&s, v, Tense::Past))
    }

    /// A word that is only ever a verb's `-ing` form (`matching`), not a
    /// noun or adjective of its own (`string`, `missing`).
    pub fn gerund(&self, word: &str) -> Option<Found> {
        if self.other_reading(word) {
            return None;
        }
        let (s, v) = self.verb_with(word, &[Inflection::Gerund])?;
        Some(Self::found(&s, v, Tense::Present))
    }

    /// Whether `word` is a noun or adjective in its own right (`use`,
    /// `string`), not only a form of a verb.
    pub fn other_reading(&self, word: &str) -> bool {
        self.lex.senses(word).iter().any(|s| {
            s.entry.pos != Pos::Verb
                && matches!(s.inflection, Inflection::Base | Inflection::Plural)
        })
    }

    /// Prepositional phrases and adverbs after the core of a clause, up to
    /// the end of the sentence or a clause boundary.
    pub fn tail(&mut self) -> Result<Vec<String>, ProseError> {
        let mut out = Vec::new();
        let mut adverbs = Vec::new();
        while let Some(w) = self.peek().map(str::to_string) {
            if w == "," {
                match self.list(false)? {
                    Some(words) => {
                        out.extend(words);
                        continue;
                    }
                    None => break,
                }
            }
            if boundary(&w) {
                break;
            }
            if let Some(words) = self.infinitive()? {
                out.extend(words);
                continue;
            }
            match classify_word(self.lex, &w, Context::NounPhrase) {
                Some((Tok::Part(ar), _)) => {
                    if self.joins_verbs() {
                        break;
                    }
                    // `by checking …`: the instrumental `bi-` with a verbal
                    // noun (§18.2); `by` on its own is `ḥasaba`.
                    let by_gerund = self.peek_at(1).and_then(|n| self.gerund(n));
                    if let (true, Some(found)) = (w == "by", by_gerund) {
                        self.i += 1;
                        out.extend(self.gerund_phrase(found, Some("bi"))?);
                        continue;
                    }
                    if let Some(words) = self.clause_after_preposition(&w)? {
                        out.extend(words);
                        continue;
                    }
                    self.i += 1;
                    out.push(ar);
                    // `about grammar and about politics`: the conjunction
                    // joins two prepositional phrases.
                    let joins = out
                        .last()
                        .is_some_and(|c| matches!(c.as_str(), "wa" | "aw"))
                        && self
                            .peek()
                            .is_some_and(|w| self.preposition_at(w).is_some());
                    if joins {
                        continue;
                    }
                    let np = self.prepositional_object(false)?;
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
