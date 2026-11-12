//! Noun phrases: articles, demonstratives, quantifiers, possessors with
//! `of` and `'s`, possessive pronouns, and what follows a noun (§5, §17.5,
//! §18.7–18.9).

use crate::classify::{classify_word, Context, Quant, Tok};
use crate::lexicon::Lexicon;
use crate::nominal::{Noun, NounPhrase};
use crate::numbers::render_counted;
use crate::text::{Gender, Number};

use super::parser::{boundary, Parser, AUXILIARIES, GRAMMAR_WORDS, RELATIVES};
use super::ProseError;

/// Possessive pronouns and the Arabic suffix each becomes. `her` is left
/// out: it is also the object pronoun.
const POSSESSIVES: &[(&str, &str)] = &[
    ("my", "ī"),
    ("your", "ka"),
    ("his", "hu"),
    ("its", "hā"),
    ("our", "nā"),
    ("their", "hum"),
];

/// `this`/`these` point near, `that`/`those` far.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Demonstrative {
    Near,
    Far,
}

impl Demonstrative {
    pub fn parse(word: &str) -> Option<Demonstrative> {
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
    /// What follows the noun: a relative clause or a participle.
    pub relative: Vec<String>,
    /// A gerund phrase, already rendered (§18.2).
    pub verbal: Option<Vec<String>>,
    /// `no file`: the clause the phrase is in is negated (§18.7).
    pub negated: bool,
}

impl Np {
    /// A gerund phrase as a noun phrase: the verbal noun decides agreement.
    pub fn verbal(verbal_noun: &str, words: Vec<String>) -> Np {
        Np {
            phrase: NounPhrase {
                nouns: vec![Noun::derived(verbal_noun)],
                number: Some(Number::Sg),
                ..NounPhrase::default()
            },
            verbal: Some(words),
            ..Np::default()
        }
    }

    pub fn render(self, lex: &Lexicon) -> Vec<String> {
        if let Some(mut words) = self.verbal {
            words.extend(self.relative);
            return words;
        }
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

/// A word waiting for its possessor to be complete: a demonstrative needs
/// the possessor's gender and number.
enum Prefix {
    Demonstrative(Demonstrative),
    Quant(String),
}

/// What a word did to the phrase being read.
enum Step {
    /// It belongs to the phrase.
    Took,
    /// The phrase ends before it.
    Stop,
    /// Not handled here.
    Next,
}

/// A noun phrase being read.
#[derive(Default)]
struct Builder {
    np: Np,
    /// Finished parts before each `of`, head first; `np.phrase` is the part
    /// being read.
    parts: Vec<NounPhrase>,
    /// A demonstrative or quantifier for the part being read.
    prefix: Option<Prefix>,
    /// Possessors marked with `'s` in the part being read, outermost first,
    /// and a demonstrative that belongs to the first of them.
    owners: Vec<NounPhrase>,
    owner_prefix: Option<Prefix>,
    /// Whether the whole phrase is definite: the article of `the user's
    /// name` belongs to the possessor.
    head_definite: bool,
    any: bool,
}

impl Builder {
    fn has_noun(&self) -> bool {
        !self.np.phrase.nouns.is_empty()
    }

    /// Where a determiner can stand: the very start, or the start of a
    /// possessor after `of`.
    fn at_start(&self) -> bool {
        if self.parts.is_empty() {
            return !self.any;
        }
        self.np.phrase.is_empty() && self.prefix.is_none() && self.np.possessor_prefix.is_none()
    }

    fn close_part(&mut self, part: NounPhrase, prefix: Option<Prefix>) {
        if let Some(prefix) = prefix {
            let at = self.parts.iter().map(|p| p.nouns.len()).sum();
            let word = match prefix {
                Prefix::Demonstrative(d) => d
                    .word(part.head_gender(), part.number.unwrap_or(Number::Sg))
                    .to_string(),
                Prefix::Quant(w) => w,
            };
            self.np.possessor_prefix = Some((word, at));
        }
        self.parts.push(part);
    }

    /// Close the part being read: its head, then its `'s` possessors from
    /// the innermost out, exactly as if each were an `of` (§18.8).
    fn close(&mut self) {
        let head = std::mem::take(&mut self.np.phrase);
        if self.parts.is_empty() {
            self.head_definite = self.owners.first().map_or(head.definite, |o| o.definite);
        }
        let prefix = self.prefix.take();
        self.close_part(head, prefix);
        let owners = std::mem::take(&mut self.owners);
        for (k, owner) in owners.into_iter().enumerate().rev() {
            let p = if k == 0 {
                self.owner_prefix.take()
            } else {
                None
            };
            self.close_part(owner, p);
        }
    }

    /// Merge the parts, head first and the innermost possessor last, into
    /// one iḍāfa chain: nouns in English compound order (last possessor
    /// first, head last), adjectives of the head first. All of them are
    /// definite, so their gender cannot show.
    fn finish(mut self) -> Np {
        if self.parts.is_empty() && self.owners.is_empty() {
            return self.np;
        }
        self.close();
        let parts = &self.parts;
        if let Some((last, before)) = parts.split_last() {
            let last_at = before.iter().map(|p| p.nouns.len()).sum::<usize>();
            let last_number = last.number.unwrap_or(Number::Sg);
            self.np.phrase = NounPhrase {
                nouns: parts.iter().rev().flat_map(|p| p.nouns.clone()).collect(),
                adjectives: parts.iter().flat_map(|p| p.adjectives.clone()).collect(),
                number: parts[0].number,
                definite: self.head_definite,
                agree: None,
                possessor_number: (last_number != Number::Sg).then_some((last_at, last_number)),
                possessive: last.possessive.clone(),
            };
        }
        self.np
    }

    /// `the` and `a` with nothing after them would vanish in Arabic.
    fn bare(&self) -> bool {
        self.any
            && self.np.phrase.is_empty()
            && self.np.count.is_none()
            && self.np.quant.is_none()
            && self.np.demonstrative.is_none()
    }
}

impl Parser<'_> {
    /// Parse a noun phrase. With `stop_at_verb`, a word that can be a verb
    /// ends the phrase once it has a noun (`the user writes`). A participle
    /// or relative clause after the phrase belongs to it.
    pub fn noun_phrase(&mut self, stop_at_verb: bool) -> Result<Option<Np>, ProseError> {
        if let Some(found) = self.peek().and_then(|w| self.gerund(w)) {
            let verbal_noun = found.verb.verbal_noun();
            let words = self.gerund_phrase(found, None)?;
            return Ok(Some(Np::verbal(&verbal_noun, words)));
        }
        let mut b = Builder::default();
        while let Some(w) = self.peek().map(str::to_string) {
            let step = match self.marker(&mut b, &w)? {
                Step::Next if self.ends_phrase(&b, &w, stop_at_verb) => Step::Stop,
                Step::Next => self.content(&mut b, &w)?,
                step => step,
            };
            match step {
                Step::Took => {
                    b.any = true;
                    self.i += 1;
                }
                _ => break,
            }
        }
        if (!b.parts.is_empty() || !b.owners.is_empty()) && !b.has_noun() {
            let what = if b.owners.is_empty() { "`of`" } else { "`'s`" };
            return Err(self.unknown_or(&format!("{what} needs a noun after it")));
        }
        if b.bare() {
            return Err(self.unknown_or("`the` or `a` needs a noun after it"));
        }
        let any = b.any;
        let mut np = b.finish();
        if !np.phrase.nouns.is_empty() {
            if let Some(words) = self.modifier(&np, stop_at_verb)? {
                np.relative = words;
            }
            let relative = self.peek().is_some_and(|w| RELATIVES.contains(&w));
            if relative && np.relative.is_empty() {
                np.relative = self.relative(&np)?;
            }
        }
        Ok(any.then_some(np))
    }

    /// `'s`, a possessive pronoun, `no` or a demonstrative.
    fn marker(&self, b: &mut Builder, w: &str) -> Result<Step, ProseError> {
        if w == "'s" {
            if !b.has_noun() || b.np.count.is_some() {
                return Err(self.unsupported("`'s` needs a noun before it"));
            }
            // `this user's name`: the demonstrative belongs to the
            // possessor. `all the user's names`: a quantifier belongs to the
            // head.
            if b.owners.is_empty() {
                if b.parts.is_empty() {
                    b.owner_prefix = b.np.demonstrative.take().map(Prefix::Demonstrative);
                } else if matches!(b.prefix, Some(Prefix::Demonstrative(_))) {
                    b.owner_prefix = b.prefix.take();
                }
            }
            let owner = std::mem::take(&mut b.np.phrase);
            b.owners.push(owner);
            return Ok(Step::Took);
        }
        // `its name`: the suffix replaces the article (§18.8).
        if let (Some((_, suffix)), true) = (POSSESSIVES.iter().find(|(e, _)| *e == w), b.at_start())
        {
            b.np.phrase.possessive = Some(suffix.to_string());
            b.np.phrase.definite = true;
            return Ok(Step::Took);
        }
        // `no file` is `any file` in a negated clause (§18.7).
        if w == "no" && !b.any {
            b.np.quant = Some(self.any());
            b.np.negated = true;
            b.np.phrase.number = Some(Number::Sg);
            return Ok(Step::Took);
        }
        let Some(d) = Demonstrative::parse(w) else {
            return Ok(Step::Next);
        };
        if !b.at_start() {
            // After a noun, `that` opens a relative clause.
            return Ok(Step::Stop);
        }
        if b.parts.is_empty() {
            b.np.demonstrative = Some(d);
        } else {
            b.prefix = Some(Prefix::Demonstrative(d));
        }
        if w == "these" || w == "those" {
            b.np.phrase.number = Some(Number::Pl);
        }
        b.np.phrase.definite = true;
        Ok(Step::Took)
    }

    /// Words that end the phrase before they are classified.
    fn ends_phrase(&self, b: &Builder, w: &str, stop_at_verb: bool) -> bool {
        if AUXILIARIES.contains(&w) || boundary(w) {
            return true;
        }
        if b.has_noun() && stop_at_verb && self.verb_sense(w) {
            return true;
        }
        // `paths matching a pattern`: an `-ing` word after a noun is a
        // participle describing it (§18.1).
        if b.has_noun() && self.gerund(w).is_some() {
            return true;
        }
        // A verb-only word ends the phrase (`the user writes`), except
        // straight after a determiner or adjective, where it can only be a
        // noun (`the debug level`).
        let open = b.any && !b.has_noun();
        self.finite_only(w) && !open
    }

    /// An article, number, adjective, noun, quantifier or `of`.
    fn content(&self, b: &mut Builder, w: &str) -> Result<Step, ProseError> {
        let Some((tok, _)) = classify_word(self.lex, w, Context::NounPhrase) else {
            if self.lex.senses(w).is_empty() && !GRAMMAR_WORDS.contains(&w) {
                return Err(ProseError::Unknown {
                    words: vec![w.to_string()],
                });
            }
            return Ok(Step::Stop);
        };
        // A determiner after a noun starts another phrase: `update the
        // file` is a verb and its object, never one noun phrase.
        let determiner = matches!(tok, Tok::Def | Tok::Indef | Tok::Num(_) | Tok::Quant(..));
        if determiner && !b.np.phrase.is_empty() {
            return Ok(Step::Stop);
        }
        // A number on a possessor (`the name of 3 users`) would count the
        // head instead.
        if matches!(tok, Tok::Num(_)) && (!b.parts.is_empty() || !b.owners.is_empty()) {
            return Err(self.unsupported("a number after `of` is not supported"));
        }
        let phrase = &mut b.np.phrase;
        match tok {
            Tok::Def => phrase.definite = true,
            Tok::Indef => phrase.definite = false,
            Tok::Num(n) => b.np.count = Some(n),
            Tok::Dual => phrase.number = Some(Number::Dual),
            // English adjectives precede the noun; one after it is a
            // predicate and belongs to the caller.
            Tok::Adj(_) if !phrase.nouns.is_empty() => return Ok(Step::Stop),
            Tok::Adj(a) => phrase.adjectives.push(a),
            Tok::Noun(n, num) => {
                phrase.nouns.push(n);
                if phrase.number != Some(Number::Dual) {
                    phrase.number = Some(num);
                }
            }
            Tok::Quant(ar, q) => {
                if b.parts.is_empty() {
                    b.np.quant = Some(ar);
                } else if b.at_start() {
                    b.prefix = Some(Prefix::Quant(ar));
                } else {
                    return Ok(Step::Stop);
                }
                let phrase = &mut b.np.phrase;
                phrase.definite = matches!(q, Quant::All | Quant::Some);
                phrase.number = Some(if phrase.definite {
                    Number::Pl
                } else {
                    Number::Sg
                });
            }
            Tok::Of if !phrase.nouns.is_empty() => b.close(),
            _ => return Ok(Step::Stop),
        }
        Ok(Step::Took)
    }

    /// The Arabic of `any`, which `no` becomes.
    fn any(&self) -> String {
        match classify_word(self.lex, "any", Context::NounPhrase) {
            Some((Tok::Quant(ar, _), _)) => ar,
            _ => "ayy".to_string(),
        }
    }

    /// The noun phrase after a preposition. `with no name` has no clause
    /// for `no` to negate, so it is refused: say `without` (§18.7).
    pub fn prepositional_object(&mut self, stop_at_verb: bool) -> Result<Np, ProseError> {
        let np = self
            .noun_phrase(stop_at_verb)?
            .ok_or_else(|| self.unknown_or("a preposition needs a noun after it"))?;
        if np.negated {
            return Err(self.unsupported("`no` after a preposition: say `without`"));
        }
        Ok(np)
    }

    /// The rest of a list after its first noun phrase: `, b and c`, `and c`,
    /// `, b, or c`. Arabic repeats the conjunction before every item and
    /// needs no commas (`a wa b wa c`), so it does (§18.9). Anything that is
    /// not a list leaves the parser where it was. `no` inside a list is
    /// refused: it would negate the whole clause.
    pub fn list(&mut self, stop_at_verb: bool) -> Result<Option<Vec<String>>, ProseError> {
        let start = self.i;
        let mut items: Vec<Np> = Vec::new();
        let conjunction = loop {
            let comma = self.eat(&[","]).is_some();
            let last = self.eat(&["and", "or"]);
            let item = if comma || last.is_some() {
                self.noun_phrase(stop_at_verb)
            } else {
                Ok(None)
            };
            match item {
                Ok(Some(np)) => items.push(np),
                _ => {
                    self.i = start;
                    return Ok(None);
                }
            }
            if let Some(c) = last {
                break if c == "and" { "wa" } else { "aw" };
            }
        };
        if items.iter().any(|np| np.negated) {
            return Err(self.unsupported("`no` inside a list would negate the whole clause"));
        }
        Ok(Some(
            items
                .into_iter()
                .flat_map(|np| std::iter::once(conjunction.to_string()).chain(np.render(self.lex)))
                .collect(),
        ))
    }
}
