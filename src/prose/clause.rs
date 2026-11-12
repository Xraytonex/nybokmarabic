//! Clauses and their word order: main clauses, questions, commands,
//! if/when/while clauses and relative clauses (SPEC §9–11, §17).

use crate::classify::{classify_word, negator, Context, Tok};
use crate::text::{Gender, Number};
use crate::verb::Person;

use super::parser::{boundary, Np, Parser, Tense, RELATIVES};
use super::ProseError;

/// The subject of a clause, rendered, with what the verb agrees with.
pub(super) struct Subject {
    words: Vec<String>,
    /// Set for pronouns, which fix the person outright.
    person: Option<Person>,
    gender: Gender,
    number: Number,
}

/// The person the verb takes, and whether the tense is forced to the
/// Arabic perfect (after `idhā`).
#[derive(Clone, Copy)]
struct Agreement {
    person: Person,
    perfect: bool,
    gender: Gender,
    number: Number,
}

impl Agreement {
    /// Verb before subject: Arabic agrees in gender only (§9).
    fn main(s: &Subject) -> Agreement {
        Agreement {
            person: s.person.unwrap_or_else(|| Person::third(s.gender)),
            perfect: false,
            gender: s.gender,
            number: s.number,
        }
    }

    /// Subject before verb: Arabic agrees in number too (§17.2).
    fn full(gender: Gender, number: Number, person: Option<Person>, perfect: bool) -> Agreement {
        let person = person.unwrap_or(match number {
            Number::Sg => Person::third(gender),
            _ => Person::ThirdPlural,
        });
        Agreement {
            person,
            perfect,
            gender,
            number,
        }
    }

    fn subordinate(s: &Subject, perfect: bool) -> Agreement {
        Agreement::full(s.gender, s.number, s.person, perfect)
    }
}

enum Predicate {
    Verb { form: String, object: Vec<String> },
    Nominal(Vec<String>),
    Possessive(Vec<String>),
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

fn render(order: Order, subject: Vec<String>, negate: bool, pred: Predicate) -> Vec<String> {
    let mut out = Vec::new();
    match (order, pred) {
        (Order::Main, Predicate::Verb { form, object }) => {
            if negate {
                out.extend(["mā".to_string(), form, negator().to_string()]);
            } else {
                out.push(form);
            }
            out.extend(subject);
            out.extend(object);
        }
        (Order::Subordinate, Predicate::Verb { form, object }) => {
            out.extend(subject);
            if negate {
                out.extend(not());
            }
            out.push(form);
            out.extend(object);
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

impl Parser<'_> {
    /// A sentence: an optional fronted subordinate clause and its comma, the
    /// main clause, and any trailing subordinate clauses. Returns the words
    /// and whether it is a question.
    pub fn sentence(&mut self) -> Result<(Vec<String>, bool), ProseError> {
        let mut out = Vec::new();
        if let Some(ar) = self.subordinator() {
            let clause = self.subordinate(ar)?;
            if self.eat(&[","]).is_none() {
                return Err(self
                    .unsupported("put a comma after a clause that starts with if, when or while"));
            }
            out.push(ar.to_string());
            out.extend(clause);
            if let Some(last) = out.last_mut() {
                last.push(',');
            }
        }
        let (main, question) = self.main_clause()?;
        out.extend(main);
        loop {
            let comma = self.eat(&[","]).is_some();
            match self.subordinator() {
                Some(ar) => {
                    if let (true, Some(last)) = (comma, out.last_mut()) {
                        last.push(',');
                    }
                    out.push(ar.to_string());
                    out.extend(self.subordinate(ar)?);
                }
                None if comma && self.peek().is_some_and(|w| RELATIVES.contains(&w)) => {
                    return Err(self.unsupported("leave out the comma before that, which or who"))
                }
                None if comma => {
                    return Err(
                        self.unsupported("a comma here must start an if, when or while clause")
                    )
                }
                None => break,
            }
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

    /// `if`/`when`/`while` + subject first. Arabic `idhā` takes the perfect
    /// whatever the English tense.
    fn subordinate(&mut self, conjunction: &str) -> Result<Vec<String>, ProseError> {
        let subject = self.subject()?;
        let agr = Agreement::subordinate(&subject, conjunction.ends_with("idhā"));
        let (negate, pred) = self.predicate(None, agr)?;
        let mut out = render(Order::Subordinate, subject.words, negate, pred);
        out.extend(self.tail()?);
        Ok(out)
    }

    fn main_clause(&mut self) -> Result<(Vec<String>, bool), ProseError> {
        let questions = ["does", "do", "did", "is", "are", "was", "were"];
        if self.peek_at(1) != Some("not") {
            if let Some(aux) = self.eat(&questions) {
                let subject = self.subject()?;
                let agr = Agreement::main(&subject);
                let (negate, pred) = self.predicate(Some(aux), agr)?;
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
        let np = self
            .noun_phrase(false)?
            .ok_or_else(|| self.unsupported("no noun phrase"))?;
        let mut out = np.render(self.lex);
        out.extend(self.tail()?);
        let ended = self.peek().is_none_or(boundary);
        if out.is_empty() || !ended {
            return Err(self.unsupported("not a noun phrase"));
        }
        Ok(out)
    }

    fn declarative(&mut self) -> Result<Vec<String>, ProseError> {
        let subject = self.subject()?;
        let agr = Agreement::main(&subject);
        let (negate, pred) = self.predicate(None, agr)?;
        let mut out = vec!["fa".to_string()];
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
            let agr = Agreement::full(Gender::Feminine, Number::Sg, Some(Person::ThirdFem), false);
            let object = self.agent()?;
            let form = found.verb.passive_present(agr.person, found.standard);
            let mut out = vec!["fa".to_string()];
            out.extend(render(
                Order::Main,
                Vec::new(),
                false,
                Predicate::Verb { form, object },
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
        let word = word.to_string();
        let form = if describes {
            // The docstring's subject is the function, `it`: dropped, as
            // Arabic drops subject pronouns and Norwegian docstrings do too.
            let found = self
                .verb_tense(&word)
                .ok_or_else(|| self.unknown_or("expected a verb"))?;
            found.verb.present(Person::ThirdFem)
        } else {
            let found = self
                .base_verb(&word)
                .ok_or_else(|| self.unknown_or("expected a verb"))?;
            // Egyptian Arabic negates a command like a statement: ma- on
            // the second-person present, -š after it (§17.1).
            if negate {
                found.verb.present(Person::Second)
            } else {
                found.verb.imperative()
            }
        };
        self.i += 1;
        let (form, object) = self.object(form)?;
        let mut out = render(
            Order::Main,
            Vec::new(),
            negate,
            Predicate::Verb { form, object },
        );
        out.extend(self.tail()?);
        Ok(out)
    }

    /// `by` + the agent of a passive verb, in Modern Standard Arabic's
    /// `min qibali` (§17.4).
    fn agent(&mut self) -> Result<Vec<String>, ProseError> {
        if self.eat(&["by"]).is_none() {
            return Ok(Vec::new());
        }
        let agent = self
            .noun_phrase(false)?
            .ok_or_else(|| self.unknown_or("`by` needs a noun after it"))?;
        let mut out = vec!["min qibali".to_string()];
        out.extend(agent.render(self.lex));
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
            });
        }
        let np = self
            .noun_phrase(true)?
            .ok_or_else(|| self.unknown_or("no subject found"))?;
        let (gender, number) = (np.gender(), np.number());
        let mut words = np.render(self.lex);
        // Before the verb, a prepositional phrase can only belong to the
        // subject (`a committee with three members has …`).
        while let Some(ar) = self.preposition() {
            let pp = self
                .noun_phrase(true)?
                .ok_or_else(|| self.unknown_or("a preposition needs a noun after it"))?;
            words.push(ar);
            words.extend(pp.render(self.lex));
        }
        Ok(Subject {
            gender,
            number,
            person: None,
            words,
        })
    }

    /// The object of a verb. An object pronoun is suffixed to the verb
    /// (Arabic) and also written as a word (Norwegian).
    fn object(&mut self, form: String) -> Result<(String, Vec<String>), ProseError> {
        let pronoun = self
            .peek()
            .and_then(|w| classify_word(self.lex, w, Context::NounPhrase));
        if let Some((Tok::Pron { suffix, object }, _)) = pronoun {
            self.i += 1;
            return Ok((format!("{form}-{suffix}"), vec![object]));
        }
        let object = self
            .noun_phrase(false)?
            .map(|np| np.render(self.lex))
            .unwrap_or_default();
        Ok((form, object))
    }

    /// Auxiliary, negation and predicate. `aux` is set when a question has
    /// already read it.
    fn predicate(
        &mut self,
        aux: Option<String>,
        agr: Agreement,
    ) -> Result<(bool, Predicate), ProseError> {
        // An adverb before the verb (`always has`) goes with the other
        // adverbs, after everything else.
        let mut adverbs = Vec::new();
        while let Some((Tok::Adv(a), _)) = self
            .peek()
            .and_then(|w| classify_word(self.lex, w, Context::NounPhrase))
        {
            adverbs.push(a);
            self.i += 1;
        }
        let aux = aux.or_else(|| {
            self.eat(&[
                "does", "do", "did", "is", "are", "was", "were", "has", "have", "had",
            ])
        });
        let negate = self.eat(&["not"]).is_some();
        let mut pred = match aux.as_deref() {
            Some(be @ ("is" | "are" | "was" | "were")) => match self.passive(be, agr)? {
                Some(p) => p,
                None => {
                    let mut predicate = self
                        .noun_phrase(false)?
                        .ok_or_else(|| self.unknown_or("no predicate"))?;
                    if predicate.phrase.nouns.is_empty() {
                        // Predicate adjectives agree with the subject,
                        // Norwegian style.
                        predicate.phrase.agree = Some(agr.gender);
                        predicate.phrase.number = Some(agr.number);
                    }
                    Predicate::Nominal(predicate.render(self.lex))
                }
            },
            Some("has" | "have" | "had") => Predicate::Possessive(
                self.noun_phrase(false)?
                    .map(|np| np.render(self.lex))
                    .unwrap_or_default(),
            ),
            _ => {
                let word = self
                    .peek()
                    .map(str::to_string)
                    .ok_or_else(|| self.unsupported("no verb found"))?;
                let found = self
                    .verb_tense(&word)
                    .ok_or_else(|| self.unknown_or("expected a verb"))?;
                self.i += 1;
                let past = agr.perfect || aux.as_deref() == Some("did");
                let form = match (past, found.tense) {
                    (false, Tense::Present) => found.verb.present(agr.person),
                    _ => found.verb.past(agr.person),
                };
                let (form, object) = self.object(form)?;
                Predicate::Verb { form, object }
            }
        };
        match &mut pred {
            Predicate::Verb { object: words, .. }
            | Predicate::Nominal(words)
            | Predicate::Possessive(words) => words.extend(adverbs),
        }
        Ok((negate, pred))
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
        let form = if past || agr.perfect {
            found.verb.passive_past(agr.person)
        } else {
            found.verb.passive_present(agr.person, found.standard)
        };
        let object = self.agent()?;
        Ok(Some(Predicate::Verb { form, object }))
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
        if self.relative_verb_first() {
            // The noun is the subject: `the user that writes the file`.
            let agr = Agreement::full(gender, number, None, false);
            let (negate, pred) = self.predicate(None, agr)?;
            let Predicate::Verb { form, object } = pred else {
                return Err(refuse(self));
            };
            if negate {
                out.extend(not());
            }
            out.push(form);
            out.extend(object);
        } else {
            // The noun is the object: `the file that the user writes`. The
            // verb takes it back as a pronoun suffix.
            let subject = self.subject()?;
            let agr = Agreement::subordinate(&subject, false);
            let (negate, pred) = self.predicate(None, agr)?;
            let Predicate::Verb { form, object } = pred else {
                return Err(refuse(self));
            };
            out.extend(subject.words);
            if negate {
                out.extend(not());
            }
            out.push(format!("{form}-{}", resumptive(gender, number)));
            out.extend(object);
        }
        Ok(out)
    }

    fn relative_verb_first(&self) -> bool {
        match self.peek() {
            Some(
                "does" | "do" | "did" | "is" | "are" | "was" | "were" | "has" | "have" | "had"
                | "not",
            ) => true,
            Some(w) if boundary(w) || pronoun(w).is_some() => false,
            Some(w) => self.verb_tense(w).is_some(),
            None => false,
        }
    }
}
