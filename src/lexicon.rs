//! The lexicon (`lexicon/core.toml`, embedded at build time) and its checks.

use std::collections::HashMap;
use std::sync::OnceLock;

use serde::Serialize;
use toml::Value;

use crate::english::{candidates, Inflection};
use crate::root::{extract, Root};
use crate::text::{Gender, Standard};
use crate::verb::{Form, Person, Verb, QUADRILITERAL_ORDER, TRILITERAL_ORDER};

pub const CORE_TOML: &str = include_str!("../lexicon/core.toml");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Pos {
    Verb,
    Noun,
    Adj,
    Adv,
    Part,
    Det,
    Num,
}

impl Pos {
    pub fn parse(s: &str) -> Option<Pos> {
        Some(match s {
            "verb" => Pos::Verb,
            "noun" => Pos::Noun,
            "adj" => Pos::Adj,
            "adv" => Pos::Adv,
            "part" => Pos::Part,
            "det" => Pos::Det,
            "num" => Pos::Num,
            _ => return None,
        })
    }
}

/// One lexicon entry. Norwegian fields are empty for Arabic particles.
#[derive(Clone, Debug, Serialize)]
pub struct Entry {
    pub key: String,
    pub english: String,
    pub pos: Pos,
    pub bm: String,
    pub nn: String,
    pub ar: String,
    pub bm_gender: Option<Gender>,
    pub nn_gender: Option<Gender>,
    pub bm_forms: Vec<String>,
    pub nn_forms: Vec<String>,
    pub value: Option<u64>,
    pub form: u8,
    pub alt: Vec<String>,
    pub en_pl: Vec<String>,
    pub en_pp: Vec<String>,
    pub verified: bool,
    pub proposed: bool,
}

impl Entry {
    /// The standard selected by the parity rule.
    pub fn standard(&self) -> Standard {
        Standard::for_lemma(&self.bm)
    }

    /// The Norwegian lemma in the selected standard.
    pub fn lemma(&self) -> &str {
        match self.standard() {
            Standard::Bokmal => &self.bm,
            Standard::Nynorsk => &self.nn,
        }
    }

    pub fn gender(&self) -> Option<Gender> {
        match self.standard() {
            Standard::Bokmal => self.bm_gender.or(self.nn_gender),
            Standard::Nynorsk => self.nn_gender.or(self.bm_gender),
        }
    }

    pub fn forms(&self) -> &[String] {
        match self.standard() {
            Standard::Bokmal => &self.bm_forms,
            Standard::Nynorsk => &self.nn_forms,
        }
    }

    pub fn root(&self) -> Root {
        extract(self.lemma())
    }

    /// The verb this entry conjugates as. Errors if `form` does not exist
    /// for the root's length.
    pub fn verb(&self) -> Result<Verb, String> {
        let root = self.root();
        let form = Form::from_number(self.form, &root).ok_or_else(|| {
            format!(
                "{}: form {} does not exist for a {}-radical root",
                self.key,
                self.form,
                root.radicals.len()
            )
        })?;
        Ok(Verb::new(root, form))
    }

    fn from_toml(key: &str, table: &toml::Table, proposed: bool) -> Result<Entry, String> {
        let s = |k: &str| {
            table
                .get(k)
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string()
        };
        let list = |k: &str| -> Vec<String> {
            table
                .get(k)
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default()
        };
        let pos_str = s("pos");
        let pos = Pos::parse(&pos_str).ok_or_else(|| format!("{key}: unknown pos '{pos_str}'"))?;
        let gender = |k: &str| -> Result<Option<Gender>, String> {
            match table.get(k).and_then(Value::as_str) {
                None => Ok(None),
                Some(g) => Gender::parse(g)
                    .map(Some)
                    .ok_or_else(|| format!("{key}: bad {k} '{g}'")),
            }
        };
        let shared = gender("gender")?;
        let entry = Entry {
            key: key.to_string(),
            english: key.split('#').next().unwrap_or(key).to_string(),
            pos,
            bm: s("bm"),
            nn: s("nn"),
            ar: s("ar"),
            bm_gender: gender("bm_gender")?.or(shared),
            nn_gender: gender("nn_gender")?.or(shared),
            bm_forms: list("bm_forms"),
            nn_forms: list("nn_forms"),
            value: table
                .get("value")
                .and_then(Value::as_integer)
                .map(|v| v as u64),
            form: table
                .get("form")
                .and_then(Value::as_integer)
                .map(|v| v as u8)
                .unwrap_or(1),
            alt: list("alt"),
            en_pl: list("en_pl"),
            en_pp: list("en_pp"),
            verified: table
                .get("verified")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            proposed,
        };
        entry.validate()?;
        Ok(entry)
    }

    fn validate(&self) -> Result<(), String> {
        let k = &self.key;
        match self.pos {
            Pos::Part | Pos::Det => {
                if self.ar.is_empty() {
                    return Err(format!("{k}: Arabic particle needs `ar`"));
                }
            }
            _ => {
                if self.bm.is_empty() || self.nn.is_empty() {
                    return Err(format!("{k}: needs both `bm` and `nn`"));
                }
            }
        }
        if self.pos == Pos::Noun && self.gender().is_none() {
            return Err(format!("{k}: noun needs a gender"));
        }
        for forms in [&self.bm_forms, &self.nn_forms] {
            if !forms.is_empty() && forms.len() != 3 {
                return Err(format!("{k}: forms must have exactly three items"));
            }
        }
        if self.pos == Pos::Num && self.value.is_none() {
            return Err(format!("{k}: numeral needs a `value`"));
        }
        Ok(())
    }
}

/// A word looked up in the lexicon, with what its English form told us.
#[derive(Clone, Debug)]
pub struct Sense<'a> {
    pub entry: &'a Entry,
    pub inflection: Inflection,
}

#[derive(Clone, Debug, Default)]
pub struct Lexicon {
    entries: Vec<Entry>,
    by_english: HashMap<String, Vec<usize>>,
    irregular: HashMap<String, Vec<(usize, Inflection)>>,
}

static CORE: OnceLock<Lexicon> = OnceLock::new();

impl Lexicon {
    /// The embedded core lexicon.
    pub fn core() -> &'static Lexicon {
        CORE.get_or_init(|| Lexicon::parse(CORE_TOML).expect("embedded lexicon is valid"))
    }

    pub fn parse(src: &str) -> Result<Lexicon, String> {
        let table: toml::Table = src.parse().map_err(|e| format!("lexicon: {e}"))?;
        let mut lex = Lexicon::default();
        for (key, value) in &table {
            let t = value
                .as_table()
                .ok_or_else(|| format!("{key}: expected a table"))?;
            lex.push(Entry::from_toml(key, t, false)?);
        }
        Ok(lex)
    }

    /// A copy of this lexicon with `[proposed]` entries from a glossary added.
    pub fn with_proposed(&self, proposed: &toml::Table) -> Result<Lexicon, String> {
        let mut lex = self.clone();
        for (key, value) in proposed {
            let t = value
                .as_table()
                .ok_or_else(|| format!("proposed {key}: expected a table"))?;
            lex.push(Entry::from_toml(key, t, true)?);
        }
        Ok(lex)
    }

    fn push(&mut self, entry: Entry) {
        let i = self.entries.len();
        let mut names = vec![entry.english.clone()];
        names.extend(entry.alt.iter().cloned());
        for name in names {
            self.by_english.entry(name).or_default().push(i);
        }
        for pl in &entry.en_pl {
            self.irregular
                .entry(pl.clone())
                .or_default()
                .push((i, Inflection::Plural));
        }
        for pp in &entry.en_pp {
            self.irregular
                .entry(pp.clone())
                .or_default()
                .push((i, Inflection::PastParticiple));
        }
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Every sense an English word could have, most literal first.
    pub fn senses<'a>(&'a self, word: &str) -> Vec<Sense<'a>> {
        let w = word.to_lowercase();
        let mut out: Vec<Sense<'a>> = Vec::new();
        let add = |i: usize, inflection: Inflection, out: &mut Vec<Sense<'a>>| {
            let entry = &self.entries[i];
            if !out
                .iter()
                .any(|s| std::ptr::eq(s.entry, entry) && s.inflection == inflection)
            {
                out.push(Sense { entry, inflection });
            }
        };
        if let Some(list) = self.irregular.get(&w) {
            for &(i, infl) in list {
                add(i, infl, &mut out);
            }
        }
        for (lemma, infl) in candidates(&w) {
            if let Some(list) = self.by_english.get(&lemma) {
                for &i in list {
                    add(i, infl, &mut out);
                }
            }
        }
        out
    }

    /// Entries whose Bokmål or Nynorsk lemma is `lemma`.
    pub fn by_lemma(&self, lemma: &str) -> Vec<&Entry> {
        let l = lemma.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.bm.to_lowercase() == l || e.nn.to_lowercase() == l)
            .collect()
    }

    pub fn by_key(&self, key: &str) -> Option<&Entry> {
        self.entries.iter().find(|e| e.key == key)
    }

    /// `nbma lexicon check`.
    pub fn check(&self) -> CheckReport {
        let mut report = CheckReport {
            entries: self.entries.len(),
            unverified: self.entries.iter().filter(|e| !e.verified).count(),
            ..CheckReport::default()
        };
        let mut seen_root: HashMap<(Vec<String>, u8), &Entry> = HashMap::new();
        let mut seen_past: HashMap<String, &Entry> = HashMap::new();
        let mut taken: HashMap<Vec<String>, Vec<u8>> = HashMap::new();
        for e in self.entries.iter().filter(|e| e.pos == Pos::Verb) {
            let verb = match e.verb() {
                Ok(v) => v,
                Err(msg) => {
                    report.errors.push(msg);
                    continue;
                }
            };
            let radicals = verb.root.radicals.clone();
            let past = verb.past(Person::ThirdMasc);
            let mut clash = None;
            if let Some(prev) = seen_root.get(&(radicals.clone(), e.form)) {
                if prev.lemma() != e.lemma() {
                    clash = Some(format!(
                        "root collision: {} ({}) and {} ({}) share root {} in form {}",
                        prev.key,
                        prev.lemma(),
                        e.key,
                        e.lemma(),
                        verb.root.display(),
                        e.form
                    ));
                }
            } else if let Some(prev) = seen_past.get(&past) {
                if prev.lemma() != e.lemma() {
                    clash = Some(format!(
                        "surface collision: {} ({}) and {} ({}) both conjugate as {}",
                        prev.key,
                        prev.lemma(),
                        e.key,
                        e.lemma(),
                        past
                    ));
                }
            }
            let used = taken.entry(radicals.clone()).or_default();
            if let Some(msg) = clash {
                let order = if verb.root.is_quadriliteral() {
                    QUADRILITERAL_ORDER
                } else {
                    TRILITERAL_ORDER
                };
                let free = order.iter().find(|n| !used.contains(n)).copied();
                report.errors.push(match free {
                    Some(n) => format!("{msg}; set `form = {n}` on [{}]", e.key),
                    None => format!("{msg}; no free form left, choose another lemma"),
                });
            }
            used.push(e.form);
            seen_root.entry((radicals, e.form)).or_insert(e);
            seen_past.entry(past).or_insert(e);
        }
        report
    }
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct CheckReport {
    pub entries: usize,
    pub unverified: usize,
    pub errors: Vec<String>,
}

impl CheckReport {
    pub fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn core_lexicon_loads_and_is_large_enough() {
        let lex = Lexicon::core();
        assert!(lex.entries().len() >= 300);
    }

    #[test]
    fn core_lexicon_passes_its_own_check() {
        let report = Lexicon::core().check();
        assert!(report.ok(), "{:#?}", report.errors);
    }

    #[test]
    fn parity_selects_lemma() {
        let lex = Lexicon::core();
        let user = lex.by_key("user").unwrap();
        assert_eq!(user.lemma(), "brukar");
        let house = lex.by_key("house").unwrap();
        assert_eq!(house.lemma(), "hus");
    }

    #[test]
    fn senses_lemmatize_english() {
        let lex = Lexicon::core();
        let senses = lex.senses("users");
        assert!(senses
            .iter()
            .any(|s| s.entry.key == "user" && s.inflection == Inflection::Plural));
        let senses = lex.senses("written");
        assert!(senses
            .iter()
            .any(|s| s.entry.key == "write" && s.inflection == Inflection::PastParticiple));
        assert!(lex
            .senses("children")
            .iter()
            .any(|s| s.entry.key == "child"));
        assert!(lex.senses("err").iter().any(|s| s.entry.key == "error"));
    }

    #[test]
    fn collisions_are_build_errors() {
        let src = r#"
[read]
pos = "verb"
bm = "lese"
nn = "lese"
[loose]
pos = "verb"
bm = "løse"
nn = "løyse"
"#;
        let lex = Lexicon::parse(src).unwrap();
        let report = lex.check();
        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(report.errors[0].contains("set `form = 2` on [loose]"));
        let fixed =
            Lexicon::parse(&src.replace("nn = \"løyse\"", "nn = \"løyse\"\nform = 2")).unwrap();
        assert!(fixed.check().ok());
    }

    #[test]
    fn invalid_entries_are_rejected() {
        assert!(Lexicon::parse("[x]\npos = \"noun\"\nbm = \"a\"\nnn = \"a\"").is_err());
        assert!(Lexicon::parse("[x]\npos = \"blah\"").is_err());
        assert!(Lexicon::parse("[x]\npos = \"part\"").is_err());
    }
}
