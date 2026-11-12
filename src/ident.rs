//! Translating identifiers (SPEC §5.1).

use serde::Serialize;

use crate::classify::{classify_word, negator, passes_through, Context, Quant, Tok, WordNote};
use crate::fold::{split_identifier, to_code, Case};
use crate::lexicon::{Lexicon, Pos};
use crate::nominal::NounPhrase;
use crate::numbers::render_counted;
use crate::segment::read;
use crate::text::Number;
use crate::verb::Person;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Fn,
    Var,
    Bool,
    Type,
}

impl Role {
    pub fn parse(s: &str) -> Option<Role> {
        Some(match s {
            "fn" | "function" | "method" => Role::Fn,
            "var" | "variable" | "field" | "param" | "const" => Role::Var,
            "bool" | "boolean" | "predicate" => Role::Bool,
            "type" | "class" | "struct" | "enum" | "trait" => Role::Type,
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Options {
    pub case: Option<Case>,
    pub number: Option<Number>,
    /// A framework prefix kept verbatim (`test_` for unittest and pytest).
    pub keep_prefix: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Translation {
    pub input: String,
    pub output: String,
    pub prose: String,
    pub role: Role,
    pub case: Case,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub kept_prefix: String,
    pub words: Vec<WordNote>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum IdentError {
    Empty,
    /// Dunder names (`__init__`) belong to the language runtime.
    Reserved {
        name: String,
    },
    Unknown {
        words: Vec<String>,
    },
    /// Nothing would be left: only articles, or a dangling `of`.
    Untranslatable {
        reason: String,
    },
}

impl std::fmt::Display for IdentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentError::Empty => write!(f, "empty identifier"),
            IdentError::Reserved { name } => {
                write!(f, "{name} is a runtime name and is never translated")
            }
            IdentError::Unknown { words } => write!(
                f,
                "not in the lexicon: {}. Propose a lemma, abbreviation or acronym with `nbma propose`.",
                words.join(", ")
            ),
            IdentError::Untranslatable { reason } => write!(f, "cannot translate: {reason}"),
        }
    }
}

/// Builds the prose words of an identifier from its tokens.
struct Builder {
    role: Role,
    out: Vec<String>,
    np: NounPhrase,
    of: Option<NounPhrase>,
    definite: Option<bool>,
    count: Option<u64>,
    dual: bool,
    negate: bool,
    quant: Option<Quant>,
    after_particle: bool,
    after_has: bool,
    /// Index in `out` of the last verb, which takes object suffixes.
    verb: Option<usize>,
}

impl Builder {
    fn new(role: Role) -> Builder {
        Builder {
            role,
            out: Vec::new(),
            np: NounPhrase::default(),
            of: None,
            definite: None,
            count: None,
            dual: false,
            negate: false,
            quant: None,
            after_particle: false,
            after_has: false,
            verb: None,
        }
    }

    fn default_definite(&self) -> bool {
        if let Some(d) = self.definite {
            return d;
        }
        if self.after_has {
            return false;
        }
        match self.quant {
            Some(Quant::All | Quant::Some) => return true,
            Some(Quant::Each | Quant::Any) => return false,
            None => {}
        }
        self.after_particle || self.role == Role::Fn
    }

    /// Render the pending noun phrase, if any.
    fn flush(&mut self, number_override: Option<Number>) {
        if self.np.is_empty() {
            return;
        }
        let mut np = std::mem::take(&mut self.np);
        if let Some(of) = self.of.take() {
            let mut nouns = np.nouns;
            nouns.extend(of.nouns);
            let mut adjectives = of.adjectives;
            adjectives.extend(np.adjectives);
            np = NounPhrase {
                nouns,
                adjectives,
                number: of.number,
                definite: false,
                agree: None,
            };
        }
        np.definite = self.default_definite();
        match self.quant {
            Some(Quant::All | Quant::Some) => np.number = Some(Number::Pl),
            Some(Quant::Each | Quant::Any) => np.number = Some(Number::Sg),
            None => {}
        }
        if self.dual {
            np.number = Some(Number::Dual);
        }
        if let Some(n) = number_override {
            np.number = Some(n);
        }
        let predicate_negation = self.negate && np.nouns.is_empty();
        if predicate_negation {
            // A verbless predicate: the circumfix wraps the null copula.
            self.out.push("mā".into());
            self.out.push(negator().into());
            self.negate = false;
        }
        let count = self
            .count
            .filter(|_| !self.dual && number_override.is_none());
        self.out.extend(render_counted(Lexicon::core(), np, count));
        self.definite = None;
        self.count = None;
        self.dual = false;
        self.quant = None;
        self.after_particle = false;
        self.after_has = false;
    }

    fn push_word(&mut self, word: String) {
        self.flush(None);
        self.out.push(word);
    }
}

/// Translate one identifier.
pub fn translate(
    lex: &Lexicon,
    ident: &str,
    role: Role,
    opts: &Options,
) -> Result<Translation, IdentError> {
    if let Some(prefix) = opts.keep_prefix.as_deref().filter(|p| !p.is_empty()) {
        if let Some(rest) = ident.strip_prefix(prefix) {
            let inner = Options {
                keep_prefix: None,
                ..opts.clone()
            };
            let mut t = translate(lex, rest, role, &inner)?;
            t.input = ident.to_string();
            t.output = format!("{prefix}{}", t.output);
            t.kept_prefix = prefix.to_string();
            return Ok(t);
        }
    }
    let split = split_identifier(ident);
    if split.words.is_empty() {
        return Err(IdentError::Empty);
    }
    if split.prefix.len() >= 2 && split.suffix.len() >= 2 {
        return Err(IdentError::Reserved {
            name: ident.to_string(),
        });
    }
    let mut toks = Vec::new();
    let mut notes = Vec::new();
    let mut unknown = Vec::new();
    let mut ctx = if role == Role::Fn {
        Context::VerbFirst
    } else {
        Context::NounPhrase
    };
    if role == Role::Bool {
        ctx = Context::Predicate;
    }
    // Abbreviations and compounds are read as the words they stand for.
    let readings = read(lex, &split.words);
    let words: Vec<String> = readings.iter().map(|r| r.word.clone()).collect();
    for (i, word) in words.iter().enumerate() {
        let next = words.get(i + 1);
        let from = readings[i].from.as_ref();
        // An article with nothing to modify is just a letter (`a_b`, `is_a`).
        if matches!(word.as_str(), "a" | "an" | "the") && next.is_none_or(|n| passes_through(n)) {
            toks.push(Tok::Pass(word.clone()));
            continue;
        }
        if word == "of" && (i == 0 || next.is_none()) {
            return Err(IdentError::Untranslatable {
                reason: format!("`of` in {ident} has nothing on one side"),
            });
        }
        let found = classify_word(lex, word, ctx);
        match found {
            Some((tok, mut note)) => {
                if let Some((original, via)) = from {
                    note.from = Some(original.clone());
                    note.via = Some(*via);
                }
                ctx = next_context(role, &tok, ctx);
                toks.push(tok);
                notes.push(note);
            }
            None if passes_through(word) => toks.push(Tok::Pass(word.clone())),
            None => {
                // Name the word the user wrote, once.
                let original = from.map_or(word, |(o, _)| o);
                if !unknown.contains(original) {
                    unknown.push(original.clone());
                }
            }
        }
    }
    if !unknown.is_empty() {
        return Err(IdentError::Unknown { words: unknown });
    }
    let prose = build(role, toks, opts.number);
    if prose.split_whitespace().all(|w| w == "hal") {
        return Err(IdentError::Untranslatable {
            reason: format!("{ident} has no word to translate"),
        });
    }
    let case = opts.case.unwrap_or(match role {
        Role::Type => Case::Pascal,
        _ => split.case,
    });
    let output = format!("{}{}{}", split.prefix, to_code(&prose, case), split.suffix);
    Ok(Translation {
        input: ident.to_string(),
        output,
        prose,
        role,
        case,
        kept_prefix: String::new(),
        words: notes,
    })
}

fn next_context(role: Role, tok: &Tok<'_>, ctx: Context) -> Context {
    match tok {
        Tok::Copula | Tok::Not if role == Role::Bool => Context::Predicate,
        // `parse_and_save`: a conjunction in a function name starts a new verb.
        Tok::Part(w) if (w == "wa" || w == "aw") && role == Role::Fn => Context::VerbFirst,
        Tok::Part(w) if w == "wa" || w == "aw" => ctx,
        _ => Context::NounPhrase,
    }
}

fn build(role: Role, toks: Vec<Tok<'_>>, number: Option<Number>) -> String {
    let mut b = Builder::new(role);
    if role == Role::Bool {
        b.out.push("hal".into());
    }
    for tok in toks {
        match tok {
            Tok::Verb(_, verb, _) => {
                b.flush(None);
                let form = match role {
                    Role::Fn => verb.imperative(),
                    _ => verb.present(Person::ThirdMasc),
                };
                if std::mem::take(&mut b.negate) {
                    b.out.push("mā".into());
                    b.verb = Some(b.out.len());
                    b.out.extend([form, negator().to_string()]);
                } else {
                    b.verb = Some(b.out.len());
                    b.out.push(form);
                }
            }
            // Arabic attaches the object to the verb; Norwegian wants a
            // separate word. Both get one.
            Tok::Pron { suffix, object } => {
                b.flush(None);
                if let Some(i) = b.verb {
                    b.out[i] = format!("{}-{suffix}", b.out[i]);
                }
                b.out.push(object);
            }
            Tok::Noun(noun, num) => {
                b.np.nouns.push(noun);
                b.np.number = Some(num);
            }
            Tok::Adj(adj) => b.np.adjectives.push(adj),
            Tok::Of => {
                let np = std::mem::take(&mut b.np);
                b.of = Some(np);
            }
            Tok::Part(ar) => {
                b.push_word(ar);
                b.after_particle = true;
            }
            Tok::Quant(ar, q) => {
                b.push_word(ar);
                b.quant = Some(q);
            }
            Tok::Num(n) => {
                b.flush(None);
                b.count = Some(n);
            }
            Tok::Has => {
                b.push_word("ʿinda".into());
                b.after_has = true;
            }
            Tok::Def => {
                b.flush(None);
                b.definite = Some(true);
            }
            Tok::Indef => {
                b.flush(None);
                b.definite = Some(false);
            }
            Tok::Dual => b.dual = true,
            Tok::Not => b.negate = true,
            Tok::Copula => {}
            Tok::Adv(w) | Tok::Pass(w) => b.push_word(w),
        }
    }
    let last_number = if role == Role::Var || role == Role::Type {
        number
    } else {
        None
    };
    b.flush(last_number);
    if b.negate {
        b.out.push("mā".into());
        b.out.push(negator().into());
    }
    b.out.join(" ")
}

/// True if any word of `ident` is a lexicon verb in its base form, which is
/// how agents tell a function name from a variable.
pub fn looks_like_function(lex: &Lexicon, ident: &str) -> bool {
    split_identifier(ident)
        .words
        .first()
        .is_some_and(|w| lex.senses(w).iter().any(|s| s.entry.pos == Pos::Verb))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(ident: &str, role: Role) -> String {
        translate(Lexicon::core(), ident, role, &Options::default())
            .unwrap()
            .output
    }

    #[test]
    fn the_briefs_example() {
        assert_eq!(t("get_user_name", Role::Fn), "uhnut_namn_ad_den_brukaren");
        assert_eq!(t("user", Role::Var), "brukar");
    }

    #[test]
    fn booleans_take_hal_and_drop_the_copula() {
        assert_eq!(t("is_valid", Role::Bool), "hal_gyldig");
        assert_eq!(t("is_not_empty", Role::Bool), "hal_maa_ikkje_tom");
        assert_eq!(t("has_children", Role::Bool), "hal_3inda_barn");
    }

    #[test]
    fn collections_are_plural_and_pairs_dual() {
        assert_eq!(t("users", Role::Var), "brukarar");
        assert_eq!(t("user_pair", Role::Var), "brukaraan");
        let dual = translate(
            Lexicon::core(),
            "users",
            Role::Var,
            &Options {
                number: Some(Number::Dual),
                ..Options::default()
            },
        )
        .unwrap();
        assert_eq!(dual.output, "brukaraan");
    }

    #[test]
    fn types_are_pascal_and_agents_are_participles() {
        assert_eq!(t("Parser", Role::Type), "Taalik");
        assert_eq!(t("User", Role::Type), "Brukar");
    }

    #[test]
    fn of_builds_idafa() {
        assert_eq!(t("name_of_user", Role::Var), t("user_name", Role::Var));
    }

    #[test]
    fn prepositions_are_arabic_and_digit_initial_gets_the_vocative() {
        assert_eq!(t("on_click", Role::Fn), "yaa_3alaa_ad_det_klikket");
    }

    #[test]
    fn unknown_words_are_reported() {
        let err = translate(
            Lexicon::core(),
            "frobnicate_widget",
            Role::Fn,
            &Options::default(),
        )
        .unwrap_err();
        assert_eq!(
            err,
            IdentError::Unknown {
                words: vec!["frobnicate".into(), "widget".into()]
            }
        );
    }

    #[test]
    fn identifiers_never_translate_to_nothing() {
        assert_eq!(t("a_b", Role::Var), "a_b");
        assert_eq!(t("is_a", Role::Bool), "hal_a");
        assert_eq!(t("a", Role::Var), "a");
        for bad in ["of", "user_of", "_of_", "of_user"] {
            assert!(
                matches!(
                    translate(Lexicon::core(), bad, Role::Var, &Options::default()),
                    Err(IdentError::Untranslatable { .. })
                ),
                "{bad}"
            );
        }
    }

    #[test]
    fn object_pronouns_are_marked_twice() {
        assert_eq!(t("READ_ME", Role::Fn), "ULSUW_NII_MEG");
        assert_eq!(t("help_us", Role::Fn), "hajlip_naa_oss");
        assert_eq!(t("tell_them", Role::Fn), "fartil_hum_dei");
    }

    #[test]
    fn dunders_are_left_alone() {
        assert!(matches!(
            translate(Lexicon::core(), "__init__", Role::Fn, &Options::default()),
            Err(IdentError::Reserved { .. })
        ));
    }

    #[test]
    fn framework_prefixes_are_kept() {
        let opts = Options {
            keep_prefix: Some("test_".into()),
            ..Options::default()
        };
        let t = translate(Lexicon::core(), "test_get_user", Role::Fn, &opts).unwrap();
        assert_eq!(t.output, "test_uhnut_ad_den_brukaren");
    }

    #[test]
    fn camel_case_is_preserved() {
        assert_eq!(t("getUserName", Role::Fn), "uhnutNamnAdDenBrukaren");
    }
}
