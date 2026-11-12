//! Verbs used as nouns and adjectives (SPEC §18.1–18.2): gerunds and
//! infinitives become Arabic verbal nouns, and participles after a noun or a
//! comma become Arabic participles.

use crate::classify::{classify_word, Context, Tok};
use crate::nominal::Adjective;

use super::noun::Np;
use super::parser::{determiner, Found, Parser, AUXILIARIES};
use super::ProseError;

impl Parser<'_> {
    /// `reading the file`: the verbal noun in construct with its object
    /// (`qirāʾatu l-malaffi`). The construct takes no article on the verbal
    /// noun, which is how an indefinite noun renders anyway. `clitic` is a
    /// one-letter Arabic preposition written onto it (`li-`, `bi-`).
    pub fn gerund_phrase(
        &mut self,
        found: Found,
        clitic: Option<&str>,
    ) -> Result<Vec<String>, ProseError> {
        self.i += 1;
        let noun = found.verb.verbal_noun();
        let noun = match clitic {
            Some(c) => format!("{c}-{noun}"),
            None => noun,
        };
        let object = self.plain_object()?;
        Ok(std::iter::once(object.attach(&noun))
            .chain(object.words)
            .collect())
    }

    /// Whether `to` (or `in order to`) at `n` starts an infinitive, and how
    /// many words the marker takes. A word that is also a noun (`to file`)
    /// is read as a verb only when an object follows (`to file the report`).
    pub fn infinitive_at(&self, n: usize) -> Option<usize> {
        let skip = match (self.peek_at(n)?, self.peek_at(n + 1), self.peek_at(n + 2)) {
            ("to", ..) => 1,
            ("in", Some("order"), Some("to")) => 3,
            _ => return None,
        };
        let verb = self.peek_at(n + skip)?;
        if verb == "be" {
            let participle = self.peek_at(n + skip + 1)?;
            return self.participle(participle).map(|_| skip);
        }
        self.base_verb(verb)?;
        let noun_too = self.other_reading(verb);
        let object_follows = self.peek_at(n + skip + 1).is_some_and(|w| {
            determiner(w)
                || w.chars().all(|c| c.is_ascii_digit())
                || matches!(
                    classify_word(self.lex, w, Context::NounPhrase),
                    Some((Tok::Pron { .. }, _))
                )
        });
        (!noun_too || object_follows).then_some(skip)
    }

    /// `to read the file`, `in order to read it`: Arabic has no infinitive
    /// and says `li-`, "for", with the verbal noun, where Norwegian says
    /// `for å`. `to be read` takes the same verbal noun: Arabic verbal nouns
    /// have no voice (§18.2).
    pub fn infinitive(&mut self) -> Result<Option<Vec<String>>, ProseError> {
        let Some(skip) = self.infinitive_at(0) else {
            return Ok(None);
        };
        self.i += skip;
        if self.eat(&["be"]).is_some() {
            let word = self.peek().unwrap_or_default().to_string();
            let found = self
                .participle(&word)
                .ok_or_else(|| self.unsupported("`to be` needs a past participle"))?;
            self.i += 1;
            let mut out = vec![format!("li-{}", found.verb.verbal_noun())];
            out.extend(self.agent()?);
            return Ok(Some(out));
        }
        let word = self.peek().unwrap_or_default().to_string();
        let found = self
            .base_verb(&word)
            .ok_or_else(|| self.unknown_or("expected a verb after `to`"))?;
        self.gerund_phrase(found, Some("li")).map(Some)
    }

    /// `the file deleted by the user`, `paths matching a pattern`: a
    /// participle after a noun is an adjective in Arabic, where adjectives
    /// follow the noun anyway, definite when the noun is. Its agent or
    /// object and its prepositional phrases follow it (§18.1).
    pub fn modifier(
        &mut self,
        np: &Np,
        stop_at_verb: bool,
    ) -> Result<Option<Vec<String>>, ProseError> {
        let Some(w) = self.peek().map(str::to_string) else {
            return Ok(None);
        };
        let agree = |word: &str| {
            let a = Adjective::participle(word);
            if np.definite() {
                a.definite(np.number())
            } else {
                a.indefinite(np.gender(), np.number())
            }
        };
        let start = self.i;
        let mut words = if let Some(found) = self.participle(&w) {
            // Before the verb, `the user deleted` is a subject and its past
            // tense; only `by` makes the participle part of the subject.
            if stop_at_verb && self.peek_at(1) != Some("by") {
                return Ok(None);
            }
            self.i += 1;
            let mut out = vec![agree(&found.verb.passive_participle())];
            out.extend(self.agent()?);
            out
        } else if let Some(found) = self.gerund(&w) {
            self.i += 1;
            let object = self.plain_object()?;
            std::iter::once(object.attach(&agree(&found.verb.active_participle())))
                .chain(object.words)
                .collect()
        } else {
            return Ok(None);
        };
        while let Some(ar) = self.preposition() {
            let pp = self.prepositional_object(stop_at_verb)?;
            words.push(ar);
            words.extend(pp.render(self.lex));
        }
        // A subject with a participle still needs its verb (`Was the file
        // deleted by the user?` has none after it).
        let verb_follows = self
            .peek()
            .is_some_and(|w| AUXILIARIES.contains(&w) || self.verb_sense(w));
        if stop_at_verb && !verb_follows {
            self.i = start;
            return Ok(None);
        }
        Ok(Some(words))
    }

    /// `, decrementing the counter`, `, formatted by the handler`: after a
    /// comma, a participle describes the whole clause. Arabic says this with
    /// the circumstantial participle (ḥāl), indefinite (§18.9).
    pub fn circumstantial(&mut self) -> Result<Option<Vec<String>>, ProseError> {
        let start = self.i;
        // `, possibly deleting it`: adverbs go last, as everywhere.
        let mut adverbs = Vec::new();
        while let Some((Tok::Adv(a), _)) = self
            .peek()
            .and_then(|w| classify_word(self.lex, w, Context::NounPhrase))
        {
            adverbs.push(a);
            self.i += 1;
        }
        let Some(w) = self.peek().map(str::to_string) else {
            self.i = start;
            return Ok(None);
        };
        let mut out = if let Some(found) = self.gerund(&w) {
            self.i += 1;
            let object = self.plain_object()?;
            std::iter::once(object.attach(&found.verb.active_participle()))
                .chain(object.words)
                .collect()
        } else if let Some(found) = self.participle(&w) {
            self.i += 1;
            let mut out = vec![found.verb.passive_participle()];
            out.extend(self.agent()?);
            out
        } else {
            self.i = start;
            return Ok(None);
        };
        out.extend(self.tail()?);
        out.extend(adverbs);
        Ok(Some(out))
    }
}
