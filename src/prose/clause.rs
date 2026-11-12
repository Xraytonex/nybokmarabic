//! Clauses and their word order: main clauses, questions, commands,
//! subordinate and relative clauses (SPEC §9–11, §17, §18.5, §18.9).

use crate::classify::{classify_word, negator, Context, Tok};
use crate::text::{Gender, Number};
use crate::verb::Person;

use super::noun::Np;
use super::parser::{boundary, Parser, AUXILIARIES, RELATIVES};
use super::ProseError;

/// The subject of a clause, rendered, with what the verb agrees with.
pub(super) struct Subject {
    words: Vec<String>,
    /// Set for pronouns, which fix the person outright.
    person: Option<Person>,
    gender: Gender,
    number: Number,
    /// `no user …` negates the clause (§18.7).
    negated: bool,
}

/// The Arabic mood a subordinate clause forces on its verb.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mood {
    /// The tense English gave.
    Indicative,
    /// The perfect, whatever the English tense (`idhā`, `baʿda an`).
    Perfect,
    /// The subjunctive (`qabla an`).
    Subjunctive,
}

/// The person the verb takes and the mood the clause forces.
#[derive(Clone, Copy)]
pub(super) struct Agreement {
    pub person: Person,
    pub mood: Mood,
    pub gender: Gender,
    pub number: Number,
}

impl Agreement {
    /// Verb before subject: Arabic agrees in gender only (§9).
    fn main(s: &Subject) -> Agreement {
        Agreement {
            person: s.person.unwrap_or_else(|| Person::third(s.gender)),
            mood: Mood::Indicative,
            gender: s.gender,
            number: s.number,
        }
    }

    /// Subject before verb: Arabic agrees in number too (§17.2).
    fn full(gender: Gender, number: Number, person: Option<Person>, mood: Mood) -> Agreement {
        let person = person.unwrap_or(match number {
            Number::Sg => Person::third(gender),
            _ => Person::ThirdPlural,
        });
        Agreement {
            person,
            mood,
            gender,
            number,
        }
    }

    fn subordinate(s: &Subject, mood: Mood) -> Agreement {
        Agreement::full(s.gender, s.number, s.person, mood)
    }
}

/// A verb with what follows it.
pub(super) struct VerbGroup {
    pub form: String,
    pub object: Vec<String>,
    /// Adverbs, which go after everything else.
    pub adverbs: Vec<String>,
    /// The negator and form a negated clause uses instead of `mā` and
    /// `form`: `lan` for the future, no `qad` in the perfect.
    pub negative: Option<(&'static str, String)>,
    /// After a modal, `object` starts with `an` and the second verb, which
    /// is the one that takes an object suffix.
    pub modal: bool,
}

impl VerbGroup {
    /// `the file that the user reads`: the verb takes the noun back as a
    /// pronoun suffix (§17.3), on the second verb after a modal. None if the
    /// clause already has an object of its own.
    fn resumed(mut self, suffix: &str) -> Option<VerbGroup> {
        if self.modal {
            if self.object.len() != 2 {
                return None;
            }
            self.object[1] = format!("{}-{suffix}", self.object[1]);
            return Some(self);
        }
        if !self.object.is_empty() {
            return None;
        }
        self.form = format!("{}-{suffix}", self.form);
        self.negative = self.negative.map(|(neg, f)| (neg, format!("{f}-{suffix}")));
        Some(self)
    }
}

pub(super) enum Predicate {
    Verb(VerbGroup),
    Nominal(Vec<String>),
    Possessive(Vec<String>),
}

impl Predicate {
    pub fn verb(form: String, object: Vec<String>) -> Predicate {
        Predicate::Verb(VerbGroup {
            form,
            object,
            adverbs: Vec::new(),
            negative: None,
            modal: false,
        })
    }

    /// The perfect: `qad` and the past, which a negation drops (§18.4).
    pub fn perfect(form: String, object: Vec<String>) -> Predicate {
        Predicate::Verb(VerbGroup {
            form: format!("qad {form}"),
            object,
            adverbs: Vec::new(),
            negative: Some(("mā", form)),
            modal: false,
        })
    }

    /// A modal, then `an` and the second verb in `words` (§18.3).
    pub fn modal(form: String, words: Vec<String>) -> Predicate {
        Predicate::Verb(VerbGroup {
            form,
            object: words,
            adverbs: Vec::new(),
            negative: None,
            modal: true,
        })
    }
}

/// Main clauses put the verb before the subject (after `fa` or `hal`);
/// subordinate clauses put the subject first (§17.2).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Order {
    Main,
    Subordinate,
}

fn not() -> [String; 2] {
    ["mā".to_string(), negator().to_string()]
}

/// A verb with its negation: the circumfix around it in a main clause, both
/// halves before it in a subordinate one (§17.2).
fn verb_words(
    order: Order,
    negate: bool,
    form: String,
    negative: Option<(&str, String)>,
) -> Vec<String> {
    if !negate {
        return vec![form];
    }
    let (neg, form) = negative.unwrap_or(("mā", form));
    match order {
        Order::Main => vec![neg.to_string(), form, negator().to_string()],
        Order::Subordinate => vec![neg.to_string(), negator().to_string(), form],
    }
}

fn render(order: Order, subject: Vec<String>, negate: bool, pred: Predicate) -> Vec<String> {
    let mut out = Vec::new();
    match (order, pred) {
        (_, Predicate::Verb(v)) => {
            let verb = verb_words(order, negate, v.form, v.negative);
            if order == Order::Main {
                out.extend(verb);
                out.extend(subject);
            } else {
                out.extend(subject);
                out.extend(verb);
            }
            out.extend(v.object);
            out.extend(v.adverbs);
        }
        // A verbless clause: the null copula sits where the verb would, so a
        // negation wraps nothing at all (§11).
        (Order::Main, Predicate::Nominal(p)) => {
            if negate {
                out.extend(not());
            }
            out.extend(subject);
            out.extend(p);
        }
        (Order::Subordinate, Predicate::Nominal(p)) => {
            out.extend(subject);
            if negate {
                out.extend(not());
            }
            out.extend(p);
        }
        (_, Predicate::Possessive(object)) => {
            if negate {
                out.extend(not());
            }
            out.push("ʿinda".into());
            out.extend(subject);
            out.extend(object);
        }
    }
    out
}

/// Norwegian pronoun after the verb (Norwegian needs a subject) and the
/// Arabic person on the verb (Arabic needs agreement). Both are satisfied.
pub(super) fn pronoun(word: &str) -> Option<(Person, &'static str)> {
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

/// `alladhī` and its forms after a definite noun; Arabic forbids a relative
/// pronoun after an indefinite one, so Norwegian `som` fills the gap.
fn relative_marker(definite: bool, gender: Gender, number: Number) -> &'static str {
    if !definite {
        return "som";
    }
    match (gender.verb_agreement(), number) {
        (Gender::Masculine, Number::Sg) => "alladhī",
        (_, Number::Sg) => "allatī",
        (Gender::Masculine, Number::Dual) => "alladhāni",
        (_, Number::Dual) => "allatāni",
        (Gender::Masculine, Number::Pl) => "alladhīna",
        (_, Number::Pl) => "allātī",
    }
}

/// The object suffix that refers back to the noun a relative clause
/// describes (`allatī … yusakrivur-hā`, "which … writes it").
fn resumptive(gender: Gender, number: Number) -> &'static str {
    match (gender.verb_agreement(), number) {
        (Gender::Masculine, Number::Sg) => "hu",
        (_, Number::Sg) => "hā",
        (_, Number::Dual) => "humā",
        (Gender::Masculine, Number::Pl) => "hum",
        (_, Number::Pl) => "hunna",
    }
}

/// Prepositions that can also open a clause, the Arabic conjunction they
/// become and the mood it forces: `qabla an` takes the subjunctive and
/// `baʿda an` the perfect (§18.5).
const CLAUSE_PREPOSITIONS: &[(&str, &str, Mood)] = &[
    ("before", "qabla an", Mood::Subjunctive),
    ("after", "baʿda an", Mood::Perfect),
    ("until", "ḥattā", Mood::Indicative),
];

fn end_with_comma(out: &mut [String]) {
    if let Some(last) = out.last_mut() {
        last.push(',');
    }
}

impl Parser<'_> {
    /// A sentence: an optional fronted clause or prepositional phrase and
    /// its comma, the main clause, and any trailing subordinate clauses or
    /// participles. Returns the words and whether it is a question.
    pub fn sentence(&mut self) -> Result<(Vec<String>, bool), ProseError> {
        let mut out = self.fronted()?.unwrap_or_default();
        // `So …`: `fa` already means "so" (§18.9).
        self.so();
        let (main, question) = self.main_clause()?;
        out.extend(main);
        loop {
            let comma = self.eat(&[","]).is_some();
            // `…, so the file is empty`: a second main clause, whose `fa`
            // is the `so`.
            if comma && self.so() {
                end_with_comma(&mut out);
                let (next, _) = self.main_clause()?;
                out.extend(next);
                continue;
            }
            if let Some((en, ar)) = self.subordinator() {
                if comma {
                    end_with_comma(&mut out);
                }
                out.extend(self.subordinate(en, ar)?);
                continue;
            }
            if !comma {
                break;
            }
            if let Some(words) = self.circumstantial()? {
                end_with_comma(&mut out);
                out.extend(words);
                continue;
            }
            // `, with arguments,`, `, to be called when …`: a prepositional
            // phrase or an infinitive set off by commas.
            let phrase = self.infinitive_at(0).is_some()
                || self
                    .peek()
                    .is_some_and(|w| self.preposition_at(w).is_some());
            if phrase {
                let words = self.tail()?;
                if !words.is_empty() {
                    end_with_comma(&mut out);
                    out.extend(words);
                    continue;
                }
            }
            if self.peek().is_some_and(|w| RELATIVES.contains(&w)) {
                return Err(self.unsupported("leave out the comma before that, which or who"));
            }
            return Err(self.unsupported(
                "a comma here must start an if, when, while or because clause, or an -ing \
                 or past participle",
            ));
        }
        if self.joins_verbs() {
            return Err(self.unsupported(
                "`and`, `or` and `but` join nouns or commands, not statements; write two sentences",
            ));
        }
        if self.peek().is_some() {
            return Err(self.unknown_or("unexpected word"));
        }
        Ok((out, question))
    }

    /// A subordinate clause or a prepositional phrase before the main clause,
    /// with its comma. Norwegian puts the verb second after either, and `fa`
    /// is already there to keep it second (§9, §18.9).
    fn fronted(&mut self) -> Result<Option<Vec<String>>, ProseError> {
        let start = self.i;
        let mut out = if let Some((en, ar)) = self.subordinator() {
            let clause = self.subordinate(en, ar)?;
            if self.peek() != Some(",") {
                return Err(self
                    .unsupported("put a comma after a clause that starts with if, when or while"));
            }
            clause
        } else if self
            .peek()
            .is_some_and(|w| self.preposition_at(w).is_some())
        {
            match self.tail() {
                Ok(words) if !words.is_empty() && self.peek() == Some(",") => words,
                Err(e @ ProseError::Unknown { .. }) => return Err(e),
                _ => {
                    self.i = start;
                    return Ok(None);
                }
            }
        } else {
            return Ok(None);
        };
        self.i += 1;
        end_with_comma(&mut out);
        // `If …, then …`: `fa` is the `then`.
        self.eat(&["then"]);
        Ok(Some(out))
    }

    /// Eat `so` (not `so that`), which `fa` translates.
    fn so(&mut self) -> bool {
        let so = self.peek() == Some("so") && self.peek_at(1) != Some("that");
        if so {
            self.i += 1;
        }
        so
    }

    /// `if`/`when`/`while`/`because` + subject first, or `when`/`while`
    /// with an `-ing` word.
    fn subordinate(&mut self, en: &str, ar: &str) -> Result<Vec<String>, ProseError> {
        // `when instantiating a logger`: `ʿinda`, "at", and a verbal noun
        // (§18.2).
        if let Some(found) = self.peek().and_then(|w| self.gerund(w)) {
            let preposition = match en {
                "when" => "ʿinda",
                "while" => "khilāla",
                _ => return Err(self.unsupported("only when and while take an -ing word")),
            };
            let mut out = vec![preposition.to_string()];
            out.extend(self.gerund_phrase(found, None)?);
            out.extend(self.tail()?);
            return Ok(out);
        }
        // Arabic follows `idhā` with the perfect, whatever the English tense.
        let mood = if ar.ends_with("idhā") {
            Mood::Perfect
        } else {
            Mood::Indicative
        };
        let mut out = vec![ar.to_string()];
        out.extend(self.subordinate_clause(mood)?);
        Ok(out)
    }

    /// The clause after a conjunction: subject first, Norwegian style
    /// (§17.2).
    pub fn subordinate_clause(&mut self, mood: Mood) -> Result<Vec<String>, ProseError> {
        if let Some((negate, words)) = self.existential()? {
            let mut out = if negate { not().to_vec() } else { Vec::new() };
            out.extend(words);
            out.extend(self.tail()?);
            return Ok(out);
        }
        let subject = self.subject()?;
        let agr = Agreement::subordinate(&subject, mood);
        let (negate, pred) = self.predicate(None, agr)?;
        let negate = self.negation(negate, subject.negated)?;
        let mut out = render(Order::Subordinate, subject.words, negate, pred);
        out.extend(self.tail()?);
        Ok(out)
    }

    /// One negation per clause. `does not read no file` has two, which
    /// English and Arabic would read differently (§18.7).
    pub fn negation(&self, a: bool, b: bool) -> Result<bool, ProseError> {
        if a && b {
            return Err(self.unsupported("two negations in one clause"));
        }
        Ok(a || b)
    }

    /// `before`, `after` or `until` followed by a whole clause rather than a
    /// noun (§18.5). Anything else leaves the parser where it was, for the
    /// preposition to take.
    pub fn clause_after_preposition(
        &mut self,
        word: &str,
    ) -> Result<Option<Vec<String>>, ProseError> {
        let Some((_, ar, mood)) = CLAUSE_PREPOSITIONS.iter().find(|(e, ..)| *e == word) else {
            return Ok(None);
        };
        if self.peek_at(1).is_some_and(|w| self.gerund(w).is_some()) {
            return Ok(None);
        }
        let start = self.i;
        self.i += 1;
        match self.subordinate_clause(*mood) {
            Ok(words) if self.peek().is_none_or(boundary) => {
                Ok(Some(std::iter::once(ar.to_string()).chain(words).collect()))
            }
            // A word missing from the lexicon is missing either way.
            Err(e @ ProseError::Unknown { .. }) => Err(e),
            _ => {
                self.i = start;
                Ok(None)
            }
        }
    }

    fn main_clause(&mut self) -> Result<(Vec<String>, bool), ProseError> {
        let questions = ["does", "do", "did", "is", "are", "was", "were"];
        if self.peek_at(1) != Some("not") {
            if let Some(aux) = self.eat(&questions) {
                let subject = self.subject()?;
                let agr = Agreement::main(&subject);
                let (negate, pred) = self.predicate(Some(aux), agr)?;
                let negate = self.negation(negate, subject.negated)?;
                let mut out = vec!["hal".to_string()];
                out.extend(render(Order::Main, subject.words, negate, pred));
                out.extend(self.tail()?);
                return Ok((out, true));
            }
        }
        // A bare verb first is a command (`Set the level.`), even when the
        // verb could also be a noun; if that reading fails, try a subject
        // (`Test cases run.`). Otherwise the other way round, so that
        // `Logs record events.` keeps its subject.
        let start = self.i;
        let command = match self.peek() {
            Some("do") => self.peek_at(1) == Some("not"),
            Some(w) => self.base_verb(w).is_some(),
            None => false,
        };
        let first = if command {
            self.subjectless().or_else(|e| {
                self.i = start;
                self.declarative().map_err(|_| e)
            })
        } else {
            self.declarative().or_else(|e| {
                self.i = start;
                if self.starts_with_verb() {
                    self.subjectless()
                } else {
                    Err(e)
                }
            })
        };
        // Last, a heading with no verb at all (`Exception for empty
        // headers.`), which stays a noun phrase without `fa`.
        first
            .or_else(|e| {
                self.i = start;
                self.fragment().map_err(|_| e)
            })
            .map(|out| (out, false))
    }

    fn fragment(&mut self) -> Result<Vec<String>, ProseError> {
        let mut out = match self.noun_phrase(false)? {
            Some(np) if !np.negated => np.render(self.lex),
            Some(_) => return Err(self.unsupported("`no` needs a verb to negate")),
            None => Vec::new(),
        };
        out.extend(self.tail()?);
        let ended = self.peek().is_none_or(boundary);
        if out.is_empty() || !ended {
            return Err(self.unsupported("not a noun phrase"));
        }
        Ok(out)
    }

    fn declarative(&mut self) -> Result<Vec<String>, ProseError> {
        let mut out = vec!["fa".to_string()];
        if let Some((negate, words)) = self.existential()? {
            if negate {
                out.extend(not());
            }
            out.extend(words);
            out.extend(self.tail()?);
            return Ok(out);
        }
        let subject = self.subject()?;
        let agr = Agreement::main(&subject);
        let (negate, pred) = self.predicate(None, agr)?;
        let negate = self.negation(negate, subject.negated)?;
        out.extend(render(Order::Main, subject.words, negate, pred));
        out.extend(self.tail()?);
        Ok(out)
    }

    fn starts_with_verb(&self) -> bool {
        match self.peek() {
            Some("do" | "does") => self.peek_at(1) == Some("not"),
            Some(w) => {
                self.base_verb(w).is_some()
                    || self.third_person_verb(w).is_some()
                    || self.participle(w).is_some()
            }
            None => false,
        }
    }

    /// A sentence without a subject: a command (`Return the user.`), a
    /// prohibition (`Do not return the user.`) or a docstring's description
    /// (`Returns the user.`, `Does not return the user.`).
    fn subjectless(&mut self) -> Result<Vec<String>, ProseError> {
        let aux = if self.peek_at(1) == Some("not") {
            self.eat(&["do", "does"])
        } else {
            None
        };
        let negate = aux.is_some() && self.eat(&["not"]).is_some();
        let word = self
            .peek()
            .map(str::to_string)
            .ok_or_else(|| self.unsupported("no verb found"))?;
        let describes = match aux.as_deref() {
            Some(aux) => aux == "does",
            None => self.base_verb(&word).is_none(),
        };
        // `Raised when …`: an exception's docstring, a passive about `it`.
        let passive = aux.is_none() && describes && self.third_person_verb(&word).is_none();
        if passive {
            let found = self
                .participle(&word)
                .ok_or_else(|| self.unknown_or("expected a verb"))?;
            self.i += 1;
            let agr = Agreement::full(
                Gender::Feminine,
                Number::Sg,
                Some(Person::ThirdFem),
                Mood::Indicative,
            );
            let object = self.agent()?;
            let form = found.verb.passive_present(agr.person, found.standard);
            let mut out = vec!["fa".to_string()];
            out.extend(render(
                Order::Main,
                Vec::new(),
                false,
                Predicate::verb(form, object),
            ));
            out.extend(self.tail()?);
            return Ok(out);
        }
        let mut out = vec!["fa".to_string()];
        out.extend(self.command(&word, describes, negate)?);
        // `Read and return a line.`: commands (or descriptions) joined by
        // `and`/`or`, each with its own verb.
        loop {
            let save = self.i;
            self.eat(&[","]);
            let Some(conj) = self.eat(&["and", "or"]) else {
                self.i = save;
                break;
            };
            let next = self.peek().map(str::to_string);
            let same_kind = next.as_deref().is_some_and(|w| {
                if describes {
                    self.third_person_verb(w).is_some()
                } else {
                    self.base_verb(w).is_some()
                }
            });
            if !same_kind {
                self.i = save;
                break;
            }
            // A negated `or` is `and not` in Arabic: lā … wa-lā ….
            let conj = if conj == "and" || negate { "wa" } else { "aw" };
            out.push(conj.to_string());
            out.extend(self.command(&next.unwrap_or_default(), describes, negate)?);
        }
        Ok(out)
    }

    /// One verb of a command or description, its object and its tail.
    fn command(
        &mut self,
        word: &str,
        describes: bool,
        negate: bool,
    ) -> Result<Vec<String>, ProseError> {
        let found = if describes {
            self.verb_tense(word)
        } else {
            self.base_verb(word)
        }
        .ok_or_else(|| self.unknown_or("expected a verb"))?;
        self.i += 1;
        let object = self.object()?;
        let negate = self.negation(negate, object.negated)?;
        let form = if describes {
            // The docstring's subject is the function, `it`: dropped, as
            // Arabic drops subject pronouns and Norwegian docstrings do too.
            found.verb.present(Person::ThirdFem)
        } else if negate {
            // Egyptian Arabic negates a command like a statement: ma- on
            // the second-person present, -š after it (§17.1).
            found.verb.present(Person::Second)
        } else {
            found.verb.imperative()
        };
        let mut out = render(
            Order::Main,
            Vec::new(),
            negate,
            Predicate::verb(object.attach(&form), object.words),
        );
        out.extend(self.tail()?);
        Ok(out)
    }

    fn subject(&mut self) -> Result<Subject, ProseError> {
        if let Some((person, word)) = self.peek().and_then(pronoun) {
            self.i += 1;
            let (gender, number) = match person {
                Person::ThirdFem => (Gender::Feminine, Number::Sg),
                Person::FirstPlural | Person::ThirdPlural => (Gender::Masculine, Number::Pl),
                _ => (Gender::Masculine, Number::Sg),
            };
            return Ok(Subject {
                words: vec![word.to_string()],
                person: Some(person),
                gender,
                number,
                negated: false,
            });
        }
        let np: Np = self
            .noun_phrase(true)?
            .ok_or_else(|| self.unknown_or("no subject found"))?;
        let (gender, mut number, negated) = (np.gender(), np.number(), np.negated);
        let mut words = np.render(self.lex);
        // Before the verb, a prepositional phrase or an infinitive can only
        // belong to the subject (`a committee with three members has …`,
        // `the class to be used is …`).
        loop {
            if let Some(infinitive) = self.infinitive()? {
                words.extend(infinitive);
                continue;
            }
            let Some(ar) = self.preposition() else {
                break;
            };
            let pp = self.prepositional_object(true)?;
            words.push(ar);
            words.extend(pp.render(self.lex));
        }
        // `Nouns, verbs and prepositions are …`: a plural subject (§18.9).
        let joins_nouns = self.peek() == Some(",")
            || (matches!(self.peek(), Some("and" | "or")) && !self.joins_verbs());
        if joins_nouns {
            if let Some(more) = self.list(true)? {
                words.extend(more);
                number = Number::Pl;
            }
        }
        Ok(Subject {
            gender,
            number,
            person: None,
            words,
            negated,
        })
    }

    /// A relative clause after `antecedent`, starting at `that`/`which`/
    /// `who` (§17.3).
    pub fn relative(&mut self, antecedent: &Np) -> Result<Vec<String>, ProseError> {
        self.i += 1;
        let (gender, number) = (antecedent.gender(), antecedent.number());
        let mut out = vec![relative_marker(antecedent.definite(), gender, number).to_string()];
        let refuse = |p: &Self| {
            p.unsupported(
                "a relative clause with `is` or `has` needs a pronoun; put the adjective \
                 before the noun instead",
            )
        };
        // The noun is the subject (`the user that writes the file`) or the
        // object (`the file that the user writes`), which the verb takes
        // back as a pronoun suffix.
        let (subject, negate, pred) = if self.relative_verb_first() {
            let agr = Agreement::full(gender, number, None, Mood::Indicative);
            let (negate, pred) = self.predicate(None, agr)?;
            (Vec::new(), negate, pred)
        } else {
            let subject = self.subject()?;
            let agr = Agreement::subordinate(&subject, Mood::Indicative);
            let (negate, pred) = self.predicate(None, agr)?;
            let negate = self.negation(negate, subject.negated)?;
            (subject.words, negate, pred)
        };
        let Predicate::Verb(mut verb) = pred else {
            return Err(refuse(self));
        };
        if !subject.is_empty() {
            verb = verb.resumed(resumptive(gender, number)).ok_or_else(|| {
                self.unsupported("the noun is already the object of this relative clause")
            })?;
        }
        out.extend(render(
            Order::Subordinate,
            subject,
            negate,
            Predicate::Verb(verb),
        ));
        Ok(out)
    }

    fn relative_verb_first(&self) -> bool {
        // `that always writes`: an adverb before the verb.
        let adverb = matches!(
            self.peek()
                .and_then(|w| classify_word(self.lex, w, Context::NounPhrase)),
            Some((Tok::Adv(_), _))
        );
        match self.peek_at(usize::from(adverb)) {
            Some(w) if AUXILIARIES.contains(&w) => true,
            Some(w) if boundary(w) || pronoun(w).is_some() => false,
            Some(w) => self.verb_tense(w).is_some(),
            None => false,
        }
    }
}
