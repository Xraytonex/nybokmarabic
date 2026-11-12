//! Predicates: verbs and their objects, the verbless copula, possession and
//! passives (SPEC §9–11, §17.4), modals, the perfect and the progressive
//! (§18.3–18.4), `that`/`whether` clauses (§18.6) and `there is` (§18.7).

use crate::classify::{classify_word, Context, Tok};
use crate::english::Inflection;
use crate::root::extract;
use crate::text::{Number, Standard};
use crate::verb::{Form, Person, Verb};

use super::clause::{pronoun, Agreement, Mood, Predicate, VerbGroup};
use super::parser::{determiner, Found, Parser, Tense};
use super::ProseError;

/// English modals that the lexicon has as Norwegian verbs (`kunne`,
/// `skulle`, `måtte`). The verb after them takes `an` and the subjunctive.
const MODAL_VERBS: &[&str] = &["can", "should", "must"];

/// Particles that stay next to the verb in English and Norwegian alike
/// (`set up the file`, `sett opp fila`).
const PARTICLES: &[&str] = &["up", "down", "out", "back", "away", "forward"];

/// The object of a verb, verbal noun or participle.
pub(super) struct Object {
    /// An Arabic object pronoun, suffixed to the verb.
    pub suffix: Option<String>,
    pub words: Vec<String>,
    /// The object was `no …`, which negates the clause (§18.7).
    pub negated: bool,
}

impl Object {
    fn none() -> Object {
        Object {
            suffix: None,
            words: Vec::new(),
            negated: false,
        }
    }

    pub fn attach(&self, form: &str) -> String {
        match &self.suffix {
            Some(s) => format!("{form}-{s}"),
            None => form.to_string(),
        }
    }
}

/// What follows a modal.
enum Infinitive {
    Active(Found),
    Passive(Found),
    /// `be` with something other than a passive participle.
    Be,
}

/// The verb after a modal in both the moods a modal can ask for, and what
/// follows it.
struct AfterModal {
    object: Object,
    indicative: String,
    subjunctive: String,
}

/// Norwegian `være`/`vere`, chosen by the parity rule, as an Arabic verb.
/// Arabic drops the copula in the present (§11) but needs it after a modal,
/// where it says `an yakūna` (§18.3).
fn copula() -> Option<Verb> {
    let lemma = match Standard::for_lemma("være") {
        Standard::Bokmal => "være",
        Standard::Nynorsk => "vere",
    };
    let root = extract(lemma);
    let form = Form::from_number(1, &root)?;
    Some(Verb::new(root, form))
}

impl Parser<'_> {
    /// The object of a verb. An object pronoun is suffixed to the verb
    /// (Arabic) and also written as a word (Norwegian). A particle (`set up
    /// the file`) stays next to the verb, as in Norwegian. `that …` and
    /// `whether …` are clauses (§18.6).
    pub fn object(&mut self) -> Result<Object, ProseError> {
        let mut out = Object::none();
        let particle = self.peek().is_some_and(|w| PARTICLES.contains(&w))
            && self.peek_at(1).is_some_and(|w| self.starts_object(w));
        if particle {
            if let Some((Tok::Adv(a), _)) = classify_word(
                self.lex,
                self.peek().unwrap_or_default(),
                Context::NounPhrase,
            ) {
                out.words.push(a);
            }
            self.i += 1;
        }
        if let Some(clause) = self.complement()? {
            out.words.extend(clause);
            return Ok(out);
        }
        let pronoun = self
            .peek()
            .and_then(|w| classify_word(self.lex, w, Context::NounPhrase));
        if let Some((Tok::Pron { suffix, object }, _)) = pronoun {
            self.i += 1;
            out.suffix = Some(suffix);
            out.words.push(object);
            return Ok(out);
        }
        if let Some(np) = self.noun_phrase(false)? {
            out.negated = np.negated;
            out.words.extend(np.render(self.lex));
        }
        Ok(out)
    }

    /// An object that cannot negate anything: of a verbal noun or a
    /// participle, where `no` would have no clause to negate.
    pub fn plain_object(&mut self) -> Result<Object, ProseError> {
        let object = self.object()?;
        if object.negated {
            return Err(self.unsupported("`no` works only on a subject or an object of a verb"));
        }
        Ok(object)
    }

    /// A word that can start the object after a particle.
    fn starts_object(&self, word: &str) -> bool {
        determiner(word)
            || matches!(
                classify_word(self.lex, word, Context::NounPhrase),
                Some((
                    Tok::Noun(..) | Tok::Adj(_) | Tok::Pron { .. } | Tok::Num(_),
                    _
                ))
            )
    }

    /// `that …` or `whether …` after a verb: Arabic `anna` or `hal`, and a
    /// clause with the subject first, as after `if` (§18.6).
    fn complement(&mut self) -> Result<Option<Vec<String>>, ProseError> {
        let ar = match self.peek() {
            Some("whether") => "hal",
            Some("that") if self.complement_that() => "anna",
            _ => return Ok(None),
        };
        self.i += 1;
        let mut out = vec![ar.to_string()];
        out.extend(self.subordinate_clause(Mood::Indicative)?);
        Ok(Some(out))
    }

    /// `that` before a clause rather than before a noun: followed by a
    /// determiner, a subject pronoun, `there` or a plural noun (`check that
    /// files exist`); `that file` is a demonstrative.
    fn complement_that(&self) -> bool {
        self.peek_at(1).is_some_and(|w| {
            (determiner(w) && w != "that")
                || pronoun(w).is_some()
                || w == "there"
                || matches!(
                    classify_word(self.lex, w, Context::NounPhrase),
                    Some((Tok::Noun(_, Number::Pl), _))
                )
        })
    }

    /// `by` + the agent of a passive verb, in Modern Standard Arabic's
    /// `min qibali` (§17.4).
    pub fn agent(&mut self) -> Result<Vec<String>, ProseError> {
        if self.eat(&["by"]).is_none() {
            return Ok(Vec::new());
        }
        let agent = self
            .noun_phrase(false)?
            .ok_or_else(|| self.unknown_or("`by` needs a noun after it"))?;
        if agent.negated {
            return Err(self.unsupported("`by no …` is not supported; negate the verb instead"));
        }
        let mut out = vec!["min qibali".to_string()];
        out.extend(agent.render(self.lex));
        Ok(out)
    }

    /// `there is a file`: Arabic `hunāka`, "there", and no verb, so
    /// Norwegian's dummy `det er` has nothing to do (§18.7). Returns whether
    /// it is negated and its words.
    pub fn existential(&mut self) -> Result<Option<(bool, Vec<String>)>, ProseError> {
        let be = matches!(self.peek_at(1), Some("is" | "are" | "was" | "were"));
        if self.peek() != Some("there") || !be {
            return Ok(None);
        }
        self.i += 2;
        let negate = self.eat(&["not"]).is_some();
        let np = self
            .noun_phrase(false)?
            .ok_or_else(|| self.unknown_or("`there is` needs a noun after it"))?;
        let negate = negate || np.negated;
        let mut out = vec!["hunāka".to_string()];
        out.extend(np.render(self.lex));
        Ok(Some((negate, out)))
    }

    fn adverbs(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        while let Some((Tok::Adv(a), _)) = self
            .peek()
            .and_then(|w| classify_word(self.lex, w, Context::NounPhrase))
        {
            out.push(a);
            self.i += 1;
        }
        out
    }

    /// Auxiliary, negation and predicate. `aux` is set when a question has
    /// already read it.
    pub fn predicate(
        &mut self,
        aux: Option<String>,
        agr: Agreement,
    ) -> Result<(bool, Predicate), ProseError> {
        // An adverb before the verb (`always has`) goes with the other
        // adverbs, after everything else.
        let mut adverbs = self.adverbs();
        let aux = aux.or_else(|| {
            self.eat(&[
                "does", "do", "did", "is", "are", "was", "were", "has", "have", "had", "will",
                "shall", "may", "might",
            ])
        });
        let mut negate = self.eat(&["not"]).is_some();
        adverbs.extend(self.adverbs());
        let (negated, mut pred) = match aux.as_deref() {
            Some(be @ ("is" | "are" | "was" | "were")) => self.be(be, agr)?,
            Some("has" | "have" | "had") => self.have(agr)?,
            Some(m @ ("will" | "shall" | "may" | "might")) => self.future(m, negate, agr)?,
            _ => self.verb(aux.as_deref() == Some("did"), &mut negate, agr)?,
        };
        match &mut pred {
            Predicate::Verb(verb) => verb.adverbs.extend(adverbs),
            Predicate::Nominal(words) | Predicate::Possessive(words) => words.extend(adverbs),
        }
        Ok((self.negation(negate, negated)?, pred))
    }

    /// The finite form of `found` for this clause.
    fn finite(&self, found: &Found, past: bool, agr: Agreement) -> String {
        match (past, found.tense, agr.mood) {
            (_, _, Mood::Perfect) | (true, _, _) | (_, Tense::Past, _) => {
                found.verb.past(agr.person)
            }
            (false, Tense::Present, Mood::Subjunctive) => {
                // `qabla an`: the Arabic subjunctive in the mood slot, the
                // Norwegian present `-r` in the tense slot (§18.5).
                format!("{}r", found.verb.subjunctive(agr.person))
            }
            (false, Tense::Present, Mood::Indicative) => found.verb.present(agr.person),
        }
    }

    /// After `is`: the progressive, a passive, a prepositional phrase or a
    /// verbless predicate.
    fn be(&mut self, be: &str, agr: Agreement) -> Result<(bool, Predicate), ProseError> {
        let past = matches!(be, "was" | "were");
        // `is reading`: both languages say the plain present (§18.4).
        if let Some(found) = self.peek().and_then(|w| self.gerund(w)) {
            self.i += 1;
            let form = self.finite(&found, past, agr);
            let object = self.object()?;
            return Ok((
                object.negated,
                Predicate::verb(object.attach(&form), object.words),
            ));
        }
        if let Some(p) = self.passive(be, agr)? {
            return Ok((false, p));
        }
        let (negated, words) = self.nominal(agr)?;
        Ok((negated, Predicate::Nominal(words)))
    }

    /// What `is` leaves behind in a verbless clause: an adjective or noun
    /// phrase, or a prepositional phrase, which is a predicate of its own in
    /// Arabic (`is in a broken state`) and which the tail reads.
    fn nominal(&mut self, agr: Agreement) -> Result<(bool, Vec<String>), ProseError> {
        // `is to read`: an obligation, not a purpose.
        if self.infinitive_at(0).is_some() {
            return Err(self.unsupported("`is to` with a verb is not supported; say `must`"));
        }
        if self
            .peek()
            .is_some_and(|w| self.preposition_at(w).is_some())
        {
            return Ok((false, Vec::new()));
        }
        let mut predicate = self
            .noun_phrase(false)?
            .ok_or_else(|| self.unknown_or("no predicate"))?;
        if predicate.phrase.nouns.is_empty() {
            // Predicate adjectives agree with the subject, Norwegian style.
            predicate.phrase.agree = Some(agr.gender);
            predicate.phrase.number = Some(agr.number);
        }
        let negated = predicate.negated;
        Ok((negated, predicate.render(self.lex)))
    }

    /// After `has`: the perfect, or possession with `ʿinda`. Arabic marks the
    /// perfect with `qad` and the past tense, where Norwegian has `har`;
    /// negated, Arabic drops `qad` (§18.4).
    fn have(&mut self, agr: Agreement) -> Result<(bool, Predicate), ProseError> {
        if self.eat(&["been"]).is_some() {
            let word = self.peek().unwrap_or_default().to_string();
            let found = self
                .participle(&word)
                .ok_or_else(|| self.unknown_or("`has been` needs a past participle"))?;
            self.i += 1;
            let form = found.verb.passive_past(agr.person);
            let agent = self.agent()?;
            return Ok((false, Predicate::perfect(form, agent)));
        }
        let perfect = self.peek().and_then(|w| {
            let (s, v) = self.verb_with(w, &[Inflection::PastParticiple])?;
            (!self.other_reading(w)).then(|| Self::found(&s, v, Tense::Past))
        });
        if let Some(found) = perfect {
            self.i += 1;
            let object = self.object()?;
            let form = object.attach(&found.verb.past(agr.person));
            return Ok((object.negated, Predicate::perfect(form, object.words)));
        }
        let np = self.noun_phrase(false)?;
        let negated = np.as_ref().is_some_and(|np| np.negated);
        let words = np.map(|np| np.render(self.lex)).unwrap_or_default();
        Ok((negated, Predicate::Possessive(words)))
    }

    /// The verb after a modal: a bare infinitive, `be` and a past
    /// participle (a passive), or `be` and anything else (the copula).
    fn after_modal(&mut self, agr: Agreement, person: Person) -> Result<AfterModal, ProseError> {
        let next = if self.eat(&["be"]).is_some() {
            match self.peek().and_then(|w| self.participle(w)) {
                Some(found) => {
                    self.i += 1;
                    Infinitive::Passive(found)
                }
                None => Infinitive::Be,
            }
        } else {
            let word = self.peek().unwrap_or_default().to_string();
            let found = self
                .base_verb(&word)
                .ok_or_else(|| self.unknown_or("expected a verb after the modal"))?;
            self.i += 1;
            Infinitive::Active(found)
        };
        Ok(match next {
            Infinitive::Active(found) => AfterModal {
                object: self.object()?,
                indicative: found.verb.indicative(person),
                subjunctive: found.verb.subjunctive(person),
            },
            // The Norwegian s-passive is also its infinitive (`skal
            // skrivast`), so the passive keeps its `-s`.
            Infinitive::Passive(found) => AfterModal {
                object: Object {
                    words: self.agent()?,
                    ..Object::none()
                },
                indicative: found.verb.passive_present(person, found.standard),
                subjunctive: found.verb.passive_subjunctive(person, found.standard),
            },
            Infinitive::Be => {
                let verb = copula().ok_or_else(|| self.unsupported("no copula"))?;
                let (negated, words) = self.nominal(agr)?;
                AfterModal {
                    object: Object {
                        words,
                        negated,
                        suffix: None,
                    },
                    indicative: verb.indicative(person),
                    subjunctive: verb.subjunctive(person),
                }
            }
        })
    }

    /// `will`/`shall` are Arabic `sa-` on the indicative, and `will not` is
    /// `lan` with the subjunctive. `may`/`might` are Arabic `qad` with the
    /// indicative, "perhaps". Norwegian has an infinitive after all of them,
    /// so there is no `-r` (§18.3).
    fn future(
        &mut self,
        modal: &str,
        negate: bool,
        agr: Agreement,
    ) -> Result<(bool, Predicate), ProseError> {
        let may = matches!(modal, "may" | "might");
        if may && negate {
            return Err(self.unsupported("`may not` is ambiguous; say `can not` or `might`"));
        }
        let next = self.after_modal(agr, agr.person)?;
        let object = next.object;
        let pred = if may {
            Predicate::verb(
                format!("qad {}", object.attach(&next.indicative)),
                object.words,
            )
        } else {
            Predicate::Verb(VerbGroup {
                form: format!("sa-{}", object.attach(&next.indicative)),
                negative: Some(("lan", object.attach(&next.subjunctive))),
                object: object.words,
                adverbs: Vec::new(),
                modal: false,
            })
        };
        Ok((object.negated, pred))
    }

    /// A finite verb, its object, and for a modal (`can`, `should`, `must`)
    /// the verb after it.
    fn verb(
        &mut self,
        did: bool,
        negate: &mut bool,
        agr: Agreement,
    ) -> Result<(bool, Predicate), ProseError> {
        let word = self
            .peek()
            .map(str::to_string)
            .ok_or_else(|| self.unsupported("no verb found"))?;
        let found = self
            .verb_tense(&word)
            .ok_or_else(|| self.unknown_or("expected a verb"))?;
        self.i += 1;
        let form = self.finite(&found, did, agr);
        if !MODAL_VERBS.contains(&found.english.as_str()) {
            let object = self.object()?;
            return Ok((
                object.negated,
                Predicate::verb(object.attach(&form), object.words),
            ));
        }
        // `can not read`: the negation comes after the modal in English.
        *negate |= self.eat(&["not"]).is_some();
        // The second verb comes after the subject, so it agrees in number
        // too (`yastaṭīʿu l-mustakhdimūna an yaqraʾū`): Arabic `an` and the
        // subjunctive, Norwegian's bare infinitive with no `-r` (§18.3).
        let next = self.after_modal(agr, agr.after_subject())?;
        let object = next.object;
        let mut words = vec!["an".to_string(), object.attach(&next.subjunctive)];
        words.extend(object.words);
        Ok((object.negated, Predicate::modal(form, words)))
    }

    /// `was deleted`, `is being read`, `is read by the user`: a passive verb.
    /// `is closed` with no agent is a state and stays a participle (§17.4).
    fn passive(&mut self, be: &str, agr: Agreement) -> Result<Option<Predicate>, ProseError> {
        let skip = usize::from(self.peek() == Some("being"));
        let Some(found) = self.peek_at(skip).and_then(|w| self.participle(w)) else {
            return Ok(None);
        };
        let past = matches!(be, "was" | "were");
        let by = self.peek_at(skip + 1) == Some("by");
        if !(past || skip == 1 || by) {
            return Ok(None);
        }
        self.i += skip + 1;
        let form = if past || agr.mood == Mood::Perfect {
            found.verb.passive_past(agr.person)
        } else {
            found.verb.passive_present(agr.person, found.standard)
        };
        let object = self.agent()?;
        Ok(Some(Predicate::verb(form, object)))
    }
}

impl Agreement {
    /// The person of a verb that follows the subject, which agrees in number
    /// as well as gender.
    pub fn after_subject(&self) -> Person {
        match (self.person, self.number) {
            (Person::ThirdMasc | Person::ThirdFem, Number::Pl | Number::Dual) => {
                Person::ThirdPlural
            }
            (p, _) => p,
        }
    }
}
