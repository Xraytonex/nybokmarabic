//! The word-level parser: verbs, noun phrases and prepositional tails. The
//! clause rules that decide word order live in `clause.rs`.

use crate::classify::{classify_word, Context, Quant, Tok};
use crate::english::Inflection;
use crate::lexicon::{Lexicon, Pos, Sense};
use crate::nominal::NounPhrase;
use crate::numbers::render_counted;
use crate::text::{Gender, Number, Standard};
use crate::verb::Verb;

use super::ProseError;

pub(super) const AUXILIARIES: &[&str] = &[
    "does", "do", "did", "is", "are", "was", "were", "has", "have", "had", "not",
];

/// Conjunctions that open a subordinate clause, with their Arabic.
pub(super) const SUBORDINATORS: &[(&str, &str)] = &[
    ("if", "idhā"),
    ("unless", "illā idhā"),
    ("when", "ʿindamā"),
    ("while", "baynamā"),
];

/// Words that open a relative clause after a noun.
pub(super) const RELATIVES: &[&str] = &["that", "which", "who"];

/// `this`/`these` point near, `that`/`those` far.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Demonstrative {
    Near,
    Far,
}

impl Demonstrative {
    fn parse(word: &str) -> Option<Demonstrative> {
        match word {
            "this" | "these" => Some(Demonstrative::Near),
            "that" | "those" => Some(Demonstrative::Far),
            _ => None,
        }
    }

    /// The Arabic demonstrative, which precedes the definite noun as in
    /// Arabic (`hādhā l-kitāb`): the noun keeps its triple article.
    fn word(self, gender: Gender, number: Number) -> &'static str {
        use Demonstrative::*;
        match (self, gender.verb_agreement(), number) {
            (Near, Gender::Masculine, Number::Sg) => "hādhā",
            (Near, _, Number::Sg) => "hādhihi",
            (Near, Gender::Masculine, Number::Dual) => "hādhāni",
            (Near, _, Number::Dual) => "hātāni",
            (Near, _, Number::Pl) => "hāʾulāʾi",
            (Far, Gender::Masculine, Number::Sg) => "dhālika",
            (Far, _, Number::Sg) => "tilka",
            (Far, Gender::Masculine, Number::Dual) => "dhānika",
            (Far, _, Number::Dual) => "tānika",
            (Far, _, Number::Pl) => "ulāʾika",
        }
    }
}

/// A comma, a subordinating conjunction or a relative pronoun: the end of
/// whatever phrase is being read.
pub(super) fn boundary(word: &str) -> bool {
    word == "," || RELATIVES.contains(&word) || SUBORDINATORS.iter().any(|(e, _)| *e == word)
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
}

/// A parsed noun phrase plus its counting, quantifier and relative clause.
#[derive(Default)]
pub(super) struct Np {
    pub phrase: NounPhrase,
    pub count: Option<u64>,
    pub quant: Option<String>,
    pub demonstrative: Option<Demonstrative>,
    /// A demonstrative or quantifier on a possessor (`the fields of this
    /// class`), and how many nouns come before that possessor in Arabic
    /// order.
    pub possessor_prefix: Option<(String, usize)>,
    pub relative: Vec<String>,
}

/// A word waiting for its possessor to be complete: a demonstrative needs
/// the possessor's gender and number.
enum Prefix {
    Demonstrative(Demonstrative),
    Quant(String),
}

/// Close one `of` part of a noun phrase, placing its pending prefix.
fn close_part(np: &mut Np, parts: &mut Vec<NounPhrase>, part: NounPhrase, prefix: Option<Prefix>) {
    if let Some(prefix) = prefix {
        let at = parts.iter().map(|p| p.nouns.len()).sum();
        let word = match prefix {
            Prefix::Demonstrative(d) => d
                .word(part.head_gender(), part.number.unwrap_or(Number::Sg))
                .to_string(),
            Prefix::Quant(w) => w,
        };
        np.possessor_prefix = Some((word, at));
    }
    parts.push(part);
}

impl Np {
    pub fn render(self, lex: &Lexicon) -> Vec<String> {
        let mut out: Vec<String> = self.quant.into_iter().collect();
        if let Some(d) = self.demonstrative {
            let number = self.phrase.number.unwrap_or(Number::Sg);
            out.push(d.word(self.phrase.head_gender(), number).to_string());
        }
        let numeral = usize::from(self.count.is_some_and(|n| n != 1 && n != 2));
        let mut phrase = render_counted(lex, self.phrase, self.count);
        if let Some((word, at)) = self.possessor_prefix {
            let at = (numeral + at).min(phrase.len());
            phrase.insert(at, word);
        }
        out.extend(phrase);
        out.extend(self.relative);
        out
    }

    pub fn gender(&self) -> Gender {
        self.phrase.head_gender()
    }

    pub fn number(&self) -> Number {
        self.phrase.number.unwrap_or(Number::Sg)
    }

    /// Definite in Arabic: an article, or an iḍāfa chain, which always ends
    /// in a definite noun (§5).
    pub fn definite(&self) -> bool {
        self.phrase.definite || self.phrase.nouns.len() > 1
    }
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

    /// Eat a subordinating conjunction and return its Arabic.
    pub fn subordinator(&mut self) -> Option<&'static str> {
        let w = self.peek()?;
        let (_, ar) = SUBORDINATORS.iter().find(|(e, _)| *e == w)?;
        self.i += 1;
        Some(ar)
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
                    && Demonstrative::parse(w).is_none() =>
            {
                ProseError::Unknown {
                    words: vec![w.to_string()],
                }
            }
            _ => self.unsupported(reason),
        }
    }

    fn verb_with(&self, word: &str, inflections: &[Inflection]) -> Option<(Sense<'a>, Verb)> {
        self.lex.senses(word).into_iter().find_map(|s| {
            if s.entry.pos == Pos::Verb && inflections.contains(&s.inflection) {
                let verb = s.entry.verb().ok()?;
                Some((s, verb))
            } else {
                None
            }
        })
    }

    fn found(sense: &Sense<'_>, verb: Verb, tense: Tense) -> Found {
        Found {
            verb,
            tense,
            standard: sense.entry.standard(),
        }
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

    /// Parse a noun phrase. With `stop_at_verb`, a word that can be a verb
    /// ends the phrase once it has a noun (`the user writes`). A relative
    /// clause after the phrase belongs to it.
    pub fn noun_phrase(&mut self, stop_at_verb: bool) -> Result<Option<Np>, ProseError> {
        let mut np = Np::default();
        // Finished parts before each `of`, head first; `np.phrase` is the
        // part being read.
        let mut parts: Vec<NounPhrase> = Vec::new();
        let mut prefix: Option<Prefix> = None;
        let mut any = false;
        while let Some(w) = self.peek().map(str::to_string) {
            // `this file`, `that user`: only at the very start, since `that`
            // after a noun opens a relative clause.
            let fresh = np.phrase.nouns.is_empty() && np.phrase.adjectives.is_empty();
            let possessor_free = prefix.is_none() && np.possessor_prefix.is_none();
            if let Some(d) = Demonstrative::parse(&w) {
                if parts.is_empty() && !any {
                    np.demonstrative = Some(d);
                } else if !parts.is_empty() && fresh && possessor_free {
                    prefix = Some(Prefix::Demonstrative(d));
                } else {
                    // After a noun, `that` opens a relative clause.
                    break;
                }
                if w == "these" || w == "those" {
                    np.phrase.number = Some(Number::Pl);
                }
                np.phrase.definite = true;
                any = true;
                self.i += 1;
                continue;
            }
            if AUXILIARIES.contains(&w.as_str()) || boundary(&w) {
                break;
            }
            if stop_at_verb && !np.phrase.nouns.is_empty() && self.verb_sense(&w) {
                break;
            }
            // A verb-only word ends the phrase (`the user writes`), except
            // straight after a determiner or adjective, where it can only be
            // a noun (`the debug level`).
            let open = any && np.phrase.nouns.is_empty();
            if self.finite_only(&w) && !open {
                break;
            }
            let Some((tok, _)) = classify_word(self.lex, &w, Context::NounPhrase) else {
                if self.lex.senses(&w).is_empty() {
                    return Err(ProseError::Unknown { words: vec![w] });
                }
                break;
            };
            // A determiner after a noun starts another phrase: `update the
            // file` is a verb and its object, never one noun phrase.
            let determiner = matches!(tok, Tok::Def | Tok::Indef | Tok::Num(_) | Tok::Quant(..));
            if determiner && !np.phrase.nouns.is_empty() {
                break;
            }
            // A number on a possessor (`the name of 3 users`) would count the
            // head instead.
            if matches!(tok, Tok::Num(_)) && !parts.is_empty() {
                return Err(self.unsupported("a number after `of` is not supported"));
            }
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
                    if parts.is_empty() {
                        np.quant = Some(ar);
                    } else if fresh && possessor_free {
                        prefix = Some(Prefix::Quant(ar));
                    } else {
                        break;
                    }
                    np.phrase.definite = matches!(q, Quant::All | Quant::Some);
                    np.phrase.number = Some(if np.phrase.definite {
                        Number::Pl
                    } else {
                        Number::Sg
                    });
                }
                Tok::Of if !np.phrase.nouns.is_empty() => {
                    let part = std::mem::take(&mut np.phrase);
                    close_part(&mut np, &mut parts, part, prefix.take());
                }
                _ => break,
            }
            any = true;
            self.i += 1;
        }
        if !parts.is_empty() {
            if np.phrase.nouns.is_empty() {
                return Err(self.unknown_or("`of` needs a noun after it"));
            }
            // Merge into one iḍāfa chain: nouns in English compound order
            // (last possessor first, head last), adjectives of the head
            // first. All of them are definite, so their gender cannot show.
            let last = std::mem::take(&mut np.phrase);
            let last_at = parts.iter().map(|p| p.nouns.len()).sum::<usize>();
            let last_number = last.number.unwrap_or(Number::Sg);
            close_part(&mut np, &mut parts, last, prefix.take());
            let head = &parts[0];
            np.phrase = NounPhrase {
                nouns: parts.iter().rev().flat_map(|p| p.nouns.clone()).collect(),
                adjectives: parts.iter().flat_map(|p| p.adjectives.clone()).collect(),
                number: head.number,
                definite: head.definite,
                agree: None,
                possessor_number: (last_number != Number::Sg).then_some((last_at, last_number)),
            };
        }
        let bare = np.phrase.is_empty()
            && np.count.is_none()
            && np.quant.is_none()
            && np.demonstrative.is_none();
        if any && bare {
            // `the` and `a` with nothing after them would vanish in Arabic.
            return Err(self.unknown_or("`the` or `a` needs a noun after it"));
        }
        let relative = self.peek().is_some_and(|w| RELATIVES.contains(&w));
        if relative && !np.phrase.nouns.is_empty() {
            np.relative = self.relative(&np)?;
        }
        Ok(any.then_some(np))
    }

    /// Prepositional phrases and adverbs after the core of a clause, up to
    /// the end of the sentence or a clause boundary.
    pub fn tail(&mut self) -> Result<Vec<String>, ProseError> {
        let mut out = Vec::new();
        let mut adverbs = Vec::new();
        while let Some(w) = self.peek().map(str::to_string) {
            if boundary(&w) {
                break;
            }
            match classify_word(self.lex, &w, Context::NounPhrase) {
                Some((Tok::Part(ar), _)) => {
                    if self.joins_verbs() {
                        break;
                    }
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
