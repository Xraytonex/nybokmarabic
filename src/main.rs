//! `nbma`: the Nybokmarabic command line. Pure, deterministic, offline.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use serde_json::{json, Value};

use nbma::fold::{to_code, Case};
use nbma::glossary::{Glossary, FILE_NAME};
use nbma::ident::{self, IdentError, Role};
use nbma::lexicon::{Entry, Lexicon, Pos};
use nbma::nominal::{Adjective, Noun};
use nbma::root::extract;
use nbma::text::{Gender, Number, Standard};
use nbma::verb::Person;
use nbma::{classify, numbers, prose, script, untranslate};

#[derive(Parser)]
#[command(
    name = "nbma",
    version,
    about = "Nybokmarabic: Bokmål, Nynorsk and Arabic, merged by regulation."
)]
struct Cli {
    /// Use this lexicon file instead of the built-in one.
    #[arg(long, global = true, value_name = "PATH")]
    lexicon: Option<PathBuf>,
    /// Print machine-readable JSON.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show the consonantal root of a Norwegian verb.
    Root { verb: String },
    /// Inflect a Norwegian lemma (or a lexicon English key).
    Inflect {
        lemma: String,
        /// Form to show (default: all). Nouns: indef def pl def-pl dual def-dual.
        /// Verbs: root past present imperative doer passive verbal-noun place.
        /// Adjectives: indef def pl dual.
        #[arg(long)]
        form: Option<String>,
        /// Person for past/present: 1sg 2sg 3sgm 3sgf 1pl 3pl.
        #[arg(long, default_value = "3sgm")]
        person: String,
        /// Wrap a verb form in the negation circumfix.
        #[arg(long)]
        negate: bool,
        /// Part of speech, for words not in the lexicon.
        #[arg(long)]
        pos: Option<String>,
        /// Gender (m, f, n): for nouns not in the lexicon, or adjective agreement.
        #[arg(long)]
        gender: Option<String>,
        /// Number for adjective agreement: sg pl dual.
        #[arg(long)]
        number: Option<String>,
    },
    /// Translate an English identifier.
    Ident {
        identifier: String,
        #[arg(long)]
        role: String,
        /// snake, camel, pascal, screaming, kebab (default: the input's own).
        #[arg(long)]
        case: Option<String>,
        /// Force the number of a variable: sg, pl, dual (pairs and 2-tuples).
        #[arg(long)]
        number: Option<String>,
        /// Glossary to read (stable names, proposed words).
        #[arg(long, value_name = "PATH")]
        glossary: Option<PathBuf>,
        /// Write the result into the glossary.
        #[arg(long, requires = "glossary")]
        record: bool,
        /// Keep a framework prefix verbatim, e.g. `test_`.
        #[arg(long, value_name = "PREFIX")]
        keep_prefix: Option<String>,
    },
    /// Translate simple English sentences (best effort).
    Prose {
        text: String,
        #[arg(long, value_name = "PATH")]
        glossary: Option<PathBuf>,
        /// Record the sentence as a text block in the glossary.
        #[arg(long, requires = "glossary")]
        record: bool,
        /// latin (default) or arabic.
        #[arg(long, default_value = "latin")]
        script: String,
    },
    /// Fold prose mode to code mode, or render it in Arabic script.
    Fold {
        text: String,
        #[arg(long, default_value = "snake")]
        case: String,
        /// latin (code mode, default) or arabic (prose in Arabic script).
        #[arg(long, default_value = "latin")]
        script: String,
        /// Render an identifier in Arabic script (needs --i-understand-bidi).
        #[arg(long)]
        ident: bool,
        #[arg(long = "i-understand-bidi")]
        i_understand_bidi: bool,
    },
    /// Spell out a number.
    Number { value: u64 },
    /// Lexicon maintenance.
    Lexicon {
        #[command(subcommand)]
        action: LexiconAction,
    },
    /// Glossary maintenance.
    Glossary {
        #[command(subcommand)]
        action: GlossaryAction,
    },
    /// Propose a Norwegian lemma for an English word missing from the lexicon.
    Propose {
        english: String,
        #[arg(long)]
        pos: String,
        #[arg(long)]
        bm: String,
        #[arg(long)]
        nn: String,
        #[arg(long)]
        gender: Option<String>,
        #[arg(long, value_name = "PATH")]
        glossary: PathBuf,
    },
    /// Reverse a translation using ordbok.nbma.toml.
    Untranslate {
        path: PathBuf,
        #[arg(long, value_name = "PATH")]
        glossary: Option<PathBuf>,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum LexiconAction {
    /// Validate the lexicon and detect root collisions.
    Check,
}

#[derive(Subcommand)]
enum GlossaryAction {
    /// Pre-flight: run on the ORIGINAL repo before renaming anything.
    Check {
        path: PathBuf,
        #[arg(long, value_name = "PATH")]
        glossary: Option<PathBuf>,
    },
}

/// A command's result: JSON for `--json`, text otherwise, and an exit code.
struct Output {
    json: Value,
    text: String,
    code: u8,
}

impl Output {
    fn ok(json: Value, text: impl Into<String>) -> Output {
        Output {
            json,
            text: text.into(),
            code: 0,
        }
    }
}

fn fail(code: u8, json: Value, message: impl Into<String>) -> Output {
    Output {
        json,
        text: message.into(),
        code,
    }
}

fn err(message: impl Into<String>) -> Output {
    let m = message.into();
    fail(1, json!({ "error": "invalid", "message": m }), m)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let out = match load_lexicon(cli.lexicon.as_deref()) {
        Ok(lex) => run(&lex, cli.command),
        Err(e) => err(e),
    };
    if cli.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&out.json).unwrap_or_default()
        );
    } else if out.code == 0 {
        println!("{}", out.text);
    } else {
        eprintln!("nbma: {}", out.text);
    }
    ExitCode::from(out.code)
}

fn load_lexicon(path: Option<&Path>) -> Result<Lexicon, String> {
    match path {
        None => Ok(Lexicon::core().clone()),
        Some(p) => {
            let src = std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))?;
            Lexicon::parse(&src)
        }
    }
}

fn with_glossary(lex: &Lexicon, path: Option<&Path>) -> Result<(Lexicon, Glossary), String> {
    let glossary = match path {
        Some(p) => Glossary::load(p)?,
        None => Glossary::default(),
    };
    let lex = lex.with_proposed(&glossary.proposed)?;
    Ok((lex, glossary))
}

fn run(lex: &Lexicon, command: Command) -> Output {
    match command {
        Command::Root { verb } => {
            let root = extract(&verb);
            let display = root.display();
            Output::ok(
                json!({ "verb": verb, "root": display, "radicals": root.radicals,
                        "quadriliteral": root.is_quadriliteral(), "weak": root.is_weak() }),
                display,
            )
        }
        Command::Inflect {
            lemma,
            form,
            person,
            negate,
            pos,
            gender,
            number,
        } => inflect(
            lex,
            &lemma,
            form.as_deref(),
            &person,
            negate,
            pos.as_deref(),
            gender.as_deref(),
            number.as_deref(),
        ),
        Command::Ident {
            identifier,
            role,
            case,
            number,
            glossary,
            record,
            keep_prefix,
        } => {
            let opts = IdentArgs {
                case,
                number,
                keep_prefix,
                glossary,
                record,
            };
            ident_cmd(lex, &identifier, &role, opts)
        }
        Command::Prose {
            text,
            glossary,
            record,
            script: s,
        } => prose_cmd(lex, &text, glossary.as_deref(), record, &s),
        Command::Fold {
            text,
            case,
            script: s,
            ident,
            i_understand_bidi,
        } => fold_cmd(&text, &case, &s, ident, i_understand_bidi),
        Command::Number { value } => {
            let words = numbers::numeral(lex, value);
            Output::ok(json!({ "value": value, "numeral": words }), words)
        }
        Command::Lexicon {
            action: LexiconAction::Check,
        } => {
            let r = lex.check();
            let text = if r.ok() {
                format!(
                    "ok: {} entries, {} unverified, no collisions",
                    r.entries, r.unverified
                )
            } else {
                r.errors.join("\n")
            };
            let json = serde_json::to_value(&r).unwrap_or(Value::Null);
            if r.ok() {
                Output::ok(json, text)
            } else {
                fail(1, json, text)
            }
        }
        Command::Glossary {
            action: GlossaryAction::Check { path, glossary },
        } => {
            let gpath = glossary.unwrap_or_else(|| path.join(FILE_NAME));
            match Glossary::load(&gpath).and_then(|g| g.check(&path)) {
                Err(e) => err(e),
                Ok(problems) if problems.is_empty() => Output::ok(
                    json!({ "ok": true, "problems": [] }),
                    "ok: every translation is unique and new",
                ),
                Ok(problems) => fail(
                    1,
                    json!({ "ok": false, "problems": problems }),
                    problems.join("\n"),
                ),
            }
        }
        Command::Propose {
            english,
            pos,
            bm,
            nn,
            gender,
            glossary,
        } => propose(lex, &english, &pos, &bm, &nn, gender.as_deref(), &glossary),
        Command::Untranslate {
            path,
            glossary,
            dry_run,
        } => {
            let gpath = glossary.unwrap_or_else(|| path.join(FILE_NAME));
            if !gpath.exists() {
                return err(format!("no glossary at {}", gpath.display()));
            }
            match Glossary::load(&gpath).and_then(|g| untranslate::untranslate(&path, &g, dry_run))
            {
                Err(e) => err(e),
                Ok(r) => {
                    let text = format!(
                        "{}{} files, {} identifiers, {} text blocks restored",
                        if dry_run { "dry run: " } else { "" },
                        r.files_changed.len(),
                        r.identifiers,
                        r.text_blocks
                    );
                    Output::ok(serde_json::to_value(&r).unwrap_or(Value::Null), text)
                }
            }
        }
    }
}

fn find_entry<'a>(lex: &'a Lexicon, lemma: &str, pos: Option<Pos>) -> Option<&'a Entry> {
    let by_lemma = lex.by_lemma(lemma);
    let pick = |list: Vec<&'a Entry>| list.into_iter().find(|e| pos.is_none_or(|p| e.pos == p));
    pick(by_lemma).or_else(|| {
        let keyed: Vec<&Entry> = lex
            .entries()
            .iter()
            .filter(|e| e.english == lemma)
            .collect();
        pick(keyed)
    })
}

/// An entry for a word that is not in the lexicon, from `--pos`/`--gender`.
fn adhoc_entry(lemma: &str, pos: Pos, gender: Option<Gender>) -> Result<Entry, String> {
    let mut t = toml::Table::new();
    let s = |v: &str| toml::Value::String(v.to_string());
    t.insert("pos".into(), s(&format!("{pos:?}").to_lowercase()));
    t.insert("bm".into(), s(lemma));
    t.insert("nn".into(), s(lemma));
    if let Some(g) = gender {
        t.insert("gender".into(), s(g.code()));
    }
    let mut outer = toml::Table::new();
    outer.insert(lemma.to_string(), toml::Value::Table(t));
    Lexicon::default()
        .with_proposed(&outer)?
        .entries()
        .first()
        .cloned()
        .ok_or_else(|| "could not build entry".into())
}

#[allow(clippy::too_many_arguments)]
fn inflect(
    lex: &Lexicon,
    lemma: &str,
    form: Option<&str>,
    person: &str,
    negate: bool,
    pos: Option<&str>,
    gender: Option<&str>,
    number: Option<&str>,
) -> Output {
    let pos = match pos.map(|p| Pos::parse(p).ok_or(p)) {
        Some(Err(p)) => return err(format!("unknown --pos {p}")),
        Some(Ok(p)) => Some(p),
        None => None,
    };
    let gender = match gender.map(|g| Gender::parse(g).ok_or(g)) {
        Some(Err(g)) => return err(format!("unknown --gender {g}")),
        Some(Ok(g)) => Some(g),
        None => None,
    };
    let Some(person) = Person::parse(person) else {
        return err(format!("unknown --person {person}"));
    };
    let number = match number.map(|n| Number::parse(n).ok_or(n)) {
        Some(Err(n)) => return err(format!("unknown --number {n}")),
        Some(Ok(n)) => n,
        None => Number::Sg,
    };
    let entry = match find_entry(lex, lemma, pos) {
        Some(e) => e.clone(),
        None => match pos {
            Some(p) => match adhoc_entry(lemma, p, gender) {
                Ok(e) => e,
                Err(e) => return err(e),
            },
            None => {
                return fail(
                    3,
                    json!({ "error": "unknown", "words": [lemma] }),
                    format!("{lemma} is not in the lexicon; pass --pos to inflect it anyway"),
                )
            }
        },
    };
    let forms: Vec<(&str, String)> = match entry.pos {
        Pos::Noun => {
            let n = Noun::from_entry(&entry);
            vec![
                ("indef", n.indefinite(Number::Sg)),
                ("def", n.definite(Number::Sg)),
                ("pl", n.indefinite(Number::Pl)),
                ("def-pl", n.definite(Number::Pl)),
                ("dual", n.indefinite(Number::Dual)),
                ("def-dual", n.definite(Number::Dual)),
            ]
        }
        Pos::Verb => {
            let v = match entry.verb() {
                Ok(v) => v,
                Err(e) => return err(e),
            };
            let neg = |f: String| {
                if negate {
                    format!("mā {f} {}", classify::negator())
                } else {
                    f
                }
            };
            vec![
                ("root", v.root.display()),
                ("past", neg(v.past(person))),
                ("present", neg(v.present(person))),
                ("imperative", v.imperative()),
                ("doer", v.active_participle()),
                ("passive", v.passive_participle()),
                ("verbal-noun", v.verbal_noun()),
                ("place", v.noun_of_place()),
            ]
        }
        Pos::Adj => {
            let a = Adjective::from_entry(&entry);
            let g = gender.unwrap_or(Gender::Masculine);
            vec![
                ("indef", a.indefinite(g, number)),
                ("def", a.definite(number)),
                ("pl", a.indefinite(g, Number::Pl)),
                ("dual", a.indefinite(g, Number::Dual)),
            ]
        }
        Pos::Num => vec![("numeral", numbers::numeral(lex, entry.value.unwrap_or(0)))],
        Pos::Adv => vec![("adverb", entry.lemma().to_string())],
        Pos::Part | Pos::Det => vec![("arabic", entry.ar.clone())],
    };
    fn aliases(f: &str) -> &str {
        match f {
            "active-participle" | "active" => "doer",
            "passive-participle" => "passive",
            "noun-of-place" => "place",
            "definite" => "def",
            "indefinite" => "indef",
            "plural" => "pl",
            other => other,
        }
    }
    let standard = (!matches!(entry.pos, Pos::Part | Pos::Det)).then(|| entry.standard());
    let base = json!({
        "lemma": if matches!(entry.pos, Pos::Part | Pos::Det) { entry.ar.clone() } else { entry.lemma().to_string() },
        "key": entry.key, "pos": entry.pos, "standard": standard, "verified": entry.verified,
    });
    match form.map(aliases) {
        Some("all") | None => {
            let map: serde_json::Map<String, Value> = forms
                .iter()
                .map(|(k, v)| (k.to_string(), json!(v)))
                .collect();
            let width = forms.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
            let text = forms
                .iter()
                .map(|(k, v)| format!("{k:width$}  {v}"))
                .collect::<Vec<_>>()
                .join("\n");
            let mut j = base;
            j["forms"] = Value::Object(map);
            Output::ok(j, text)
        }
        Some(f) => match forms.iter().find(|(k, _)| *k == f) {
            Some((k, v)) => {
                let mut j = base;
                j["form"] = json!(k);
                j["value"] = json!(v);
                Output::ok(j, v.clone())
            }
            None => err(format!(
                "no form '{f}' for a {:?}; available: {}",
                entry.pos,
                forms.iter().map(|(k, _)| *k).collect::<Vec<_>>().join(", ")
            )),
        },
    }
}

fn parse_case(case: &str) -> Result<Case, Output> {
    Case::parse(case).ok_or_else(|| err(format!("unknown case '{case}'")))
}

struct IdentArgs {
    case: Option<String>,
    number: Option<String>,
    keep_prefix: Option<String>,
    glossary: Option<PathBuf>,
    record: bool,
}

fn ident_cmd(lex: &Lexicon, identifier: &str, role: &str, args: IdentArgs) -> Output {
    let glossary_path = args.glossary.as_deref();
    let record = args.record;
    let Some(role) = Role::parse(role) else {
        return err(format!("unknown --role {role} (fn, var, bool, type)"));
    };
    let case = match args.case.as_deref().map(parse_case) {
        Some(Err(o)) => return o,
        Some(Ok(c)) => Some(c),
        None => None,
    };
    let number = match args.number.as_deref().map(|n| Number::parse(n).ok_or(n)) {
        Some(Err(n)) => return err(format!("unknown --number {n}")),
        Some(Ok(n)) => Some(n),
        None => None,
    };
    let (lex, mut glossary) = match with_glossary(lex, glossary_path) {
        Ok(x) => x,
        Err(e) => return err(e),
    };
    if let Some(r) = glossary.get(identifier) {
        return Output::ok(
            json!({ "input": identifier, "output": r.to, "role": r.role, "from_glossary": true }),
            r.to.clone(),
        );
    }
    let opts = ident::Options {
        case,
        number,
        keep_prefix: args.keep_prefix,
    };
    match ident::translate(&lex, identifier, role, &opts) {
        Ok(t) => {
            let mut name = t.output.clone();
            if record {
                name = glossary.record(&t);
                if let Some(p) = glossary_path {
                    if let Err(e) = glossary.save(p) {
                        return err(e);
                    }
                }
            }
            let mut j = serde_json::to_value(&t).unwrap_or(Value::Null);
            j["output"] = json!(name);
            j["recorded"] = json!(record);
            Output::ok(j, name)
        }
        Err(e) => {
            let code = if matches!(e, IdentError::Unknown { .. }) {
                3
            } else {
                1
            };
            fail(
                code,
                serde_json::to_value(&e).unwrap_or(Value::Null),
                e.to_string(),
            )
        }
    }
}

fn prose_cmd(
    lex: &Lexicon,
    text: &str,
    glossary_path: Option<&Path>,
    record: bool,
    s: &str,
) -> Output {
    let (lex, mut glossary) = match with_glossary(lex, glossary_path) {
        Ok(x) => x,
        Err(e) => return err(e),
    };
    match prose::translate(&lex, text) {
        Ok(out) => {
            let rendered = match s {
                "arabic" => script::to_arabic(&out),
                "latin" => out.clone(),
                other => return err(format!("unknown --script {other}")),
            };
            if record {
                glossary.add_text(text, &rendered);
                if let Some(p) = glossary_path {
                    if let Err(e) = glossary.save(p) {
                        return err(e);
                    }
                }
            }
            Output::ok(
                json!({ "input": text, "output": rendered, "latin": out, "recorded": record }),
                rendered,
            )
        }
        Err(e) => {
            let code = if matches!(e, prose::ProseError::Unknown { .. }) {
                3
            } else {
                1
            };
            fail(
                code,
                serde_json::to_value(&e).unwrap_or(Value::Null),
                e.to_string(),
            )
        }
    }
}

fn fold_cmd(text: &str, case: &str, s: &str, ident: bool, bidi: bool) -> Output {
    let case = match parse_case(case) {
        Ok(c) => c,
        Err(o) => return o,
    };
    match (s, ident) {
        ("latin", _) => {
            let out = to_code(text, case);
            Output::ok(json!({ "input": text, "output": out, "case": case }), out)
        }
        ("arabic", false) => {
            let out = script::to_arabic(text);
            Output::ok(
                json!({ "input": text, "output": out, "script": "arabic" }),
                out,
            )
        }
        ("arabic", true) if !bidi => err(format!(
            "refusing Arabic-script identifiers without --i-understand-bidi.\n{}",
            script::BIDI_WARNING
        )),
        ("arabic", true) => {
            eprintln!("{}", script::BIDI_WARNING);
            let out = script::identifier_to_arabic(text);
            Output::ok(
                json!({ "input": text, "output": out, "script": "arabic", "warning": script::BIDI_WARNING }),
                out,
            )
        }
        (other, _) => err(format!("unknown --script {other} (latin, arabic)")),
    }
}

fn propose(
    lex: &Lexicon,
    english: &str,
    pos: &str,
    bm: &str,
    nn: &str,
    gender: Option<&str>,
    path: &Path,
) -> Output {
    let mut glossary = match Glossary::load(path) {
        Ok(g) => g,
        Err(e) => return err(e),
    };
    let mut t = toml::Table::new();
    let s = |v: &str| toml::Value::String(v.to_string());
    t.insert("pos".into(), s(pos));
    t.insert("bm".into(), s(bm));
    t.insert("nn".into(), s(nn));
    if let Some(g) = gender {
        t.insert("gender".into(), s(g));
    }
    t.insert("verified".into(), toml::Value::Boolean(false));
    let mut one = toml::Table::new();
    one.insert(english.to_lowercase(), toml::Value::Table(t.clone()));
    // Validate exactly as the lexicon would, including root collisions.
    let merged = match lex
        .with_proposed(&glossary.proposed)
        .and_then(|l| l.with_proposed(&one))
    {
        Ok(l) => l,
        Err(e) => return err(e),
    };
    let report = merged.check();
    if !report.ok() {
        return fail(
            1,
            json!({ "error": "collision", "problems": report.errors }),
            report.errors.join("\n"),
        );
    }
    glossary
        .proposed
        .insert(english.to_lowercase(), toml::Value::Table(t));
    if let Err(e) = glossary.save(path) {
        return err(e);
    }
    let selected = if Standard::for_lemma(bm) == Standard::Bokmal {
        bm
    } else {
        nn
    };
    Output::ok(
        json!({ "english": english, "pos": pos, "bm": bm, "nn": nn, "selected": selected, "recorded": true }),
        format!("proposed {english} = {selected}"),
    )
}
