//! `ordbok.nbma.toml`: the record of every translation in a target repo
//! (SPEC §5.3). Reruns read it so names stay stable; `untranslate` reverses
//! it.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use serde::Serialize;
use toml::Value;

use crate::fold::{split_identifier, to_code};
use crate::ident::Translation;

pub const FILE_NAME: &str = "ordbok.nbma.toml";

/// Ordinals appended when two originals would translate to the same name.
const ORDINALS: &[&str] = &[
    "thānī", "thālith", "rābiʿ", "khāmis", "sādis", "sābiʿ", "thāmin", "tāsiʿ", "ʿāshir",
];

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct IdentRecord {
    pub to: String,
    pub role: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct TextBlock {
    pub from: String,
    pub to: String,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct Glossary {
    pub identifiers: Vec<(String, IdentRecord)>,
    pub proposed: toml::Table,
    pub text: Vec<TextBlock>,
    pub files: Vec<(String, String)>,
}

fn str_field(t: &toml::Table, k: &str, ctx: &str) -> Result<String, String> {
    t.get(k)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("{ctx}: missing string `{k}`"))
}

impl Glossary {
    /// Load a glossary; a missing file is an empty glossary.
    pub fn load(path: &Path) -> Result<Glossary, String> {
        if !path.exists() {
            return Ok(Glossary::default());
        }
        let src = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        Glossary::parse(&src).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn parse(src: &str) -> Result<Glossary, String> {
        let table: toml::Table = src.parse().map_err(|e| format!("{e}"))?;
        let mut g = Glossary::default();
        if let Some(ids) = table.get("identifiers").and_then(Value::as_table) {
            for (from, v) in ids {
                let t = v
                    .as_table()
                    .ok_or_else(|| format!("identifiers.{from}: expected a table"))?;
                let to = str_field(t, "to", &format!("identifiers.{from}"))?;
                let role = t
                    .get("role")
                    .and_then(Value::as_str)
                    .unwrap_or("var")
                    .to_string();
                g.identifiers.push((from.clone(), IdentRecord { to, role }));
            }
        }
        if let Some(p) = table.get("proposed").and_then(Value::as_table) {
            g.proposed = p.clone();
        }
        if let Some(blocks) = table.get("text").and_then(Value::as_array) {
            for (i, b) in blocks.iter().enumerate() {
                let t = b
                    .as_table()
                    .ok_or_else(|| format!("text[{i}]: expected a table"))?;
                g.text.push(TextBlock {
                    from: str_field(t, "from", &format!("text[{i}]"))?,
                    to: str_field(t, "to", &format!("text[{i}]"))?,
                });
            }
        }
        if let Some(files) = table.get("files").and_then(Value::as_table) {
            for (from, v) in files {
                let to = v
                    .as_str()
                    .ok_or_else(|| format!("files.{from}: expected a string"))?;
                g.files.push((from.clone(), to.to_string()));
            }
        }
        Ok(g)
    }

    pub fn render(&self) -> String {
        let q = |s: &str| Value::String(s.to_string()).to_string();
        let key = |s: &str| {
            if !s.is_empty()
                && s.chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                s.to_string()
            } else {
                q(s)
            }
        };
        let mut out = String::from(
            "# ordbok.nbma.toml: written by nbma. Maps original names to Nybokmarabic.\n\
             # Reruns read it first so names stay stable. `nbma untranslate` reverses it.\n",
        );
        out.push_str("\n[identifiers]\n");
        for (from, r) in &self.identifiers {
            out.push_str(&format!(
                "{} = {{ to = {}, role = {} }}\n",
                key(from),
                q(&r.to),
                q(&r.role)
            ));
        }
        if !self.files.is_empty() {
            out.push_str("\n[files]\n");
            for (from, to) in &self.files {
                out.push_str(&format!("{} = {}\n", q(from), q(to)));
            }
        }
        for (k, v) in &self.proposed {
            let mut wrapper = toml::Table::new();
            wrapper.insert(k.clone(), v.clone());
            let mut outer = toml::Table::new();
            outer.insert("proposed".into(), Value::Table(wrapper));
            out.push('\n');
            out.push_str(&toml::to_string(&outer).unwrap_or_default());
        }
        for b in &self.text {
            out.push_str(&format!(
                "\n[[text]]\nfrom = {}\nto = {}\n",
                q(&b.from),
                q(&b.to)
            ));
        }
        out
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        std::fs::write(path, self.render()).map_err(|e| format!("{}: {e}", path.display()))
    }

    pub fn get(&self, original: &str) -> Option<&IdentRecord> {
        self.identifiers
            .iter()
            .find(|(f, _)| f == original)
            .map(|(_, r)| r)
    }

    fn owner_of(&self, translated: &str) -> Option<&str> {
        self.identifiers
            .iter()
            .find(|(_, r)| r.to == translated)
            .map(|(f, _)| f.as_str())
    }

    /// Record a translation and return the name to use. A name already
    /// taken by a different original gets an Arabic ordinal: `_thaanii`.
    pub fn record(&mut self, t: &Translation) -> String {
        if let Some(r) = self.get(&t.input) {
            return r.to.clone();
        }
        let split = split_identifier(&t.input);
        let mut name = t.output.clone();
        for ordinal in ORDINALS {
            match self.owner_of(&name) {
                Some(owner) if owner != t.input => {
                    let prose = format!("{} {ordinal}", t.prose);
                    let prefix = if t.kept_prefix.is_empty() {
                        &split.prefix
                    } else {
                        &t.kept_prefix
                    };
                    name = format!("{prefix}{}{}", to_code(&prose, t.case), split.suffix);
                }
                _ => break,
            }
        }
        let role = serde_json::to_value(t.role)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_else(|| "var".into());
        self.identifiers.push((
            t.input.clone(),
            IdentRecord {
                to: name.clone(),
                role,
            },
        ));
        name
    }

    pub fn add_text(&mut self, from: &str, to: &str) {
        if !self.text.iter().any(|b| b.from == from) {
            self.text.push(TextBlock {
                from: from.into(),
                to: to.into(),
            });
        }
    }

    /// Pre-flight check, run on the ORIGINAL repo before renaming: every
    /// translation is unique and none already occurs as a word anywhere in
    /// the repo. That is what makes `untranslate` exact.
    pub fn check(&self, repo: &Path) -> Result<Vec<String>, String> {
        let mut problems = Vec::new();
        let mut seen: HashMap<&str, &str> = HashMap::new();
        for (from, r) in &self.identifiers {
            if let Some(other) = seen.insert(r.to.as_str(), from.as_str()) {
                problems.push(format!("{other} and {from} both translate to {}", r.to));
            }
        }
        let mut seen_text: HashMap<&str, &str> = HashMap::new();
        for b in &self.text {
            if b.to.is_empty() {
                problems.push(format!(
                    "text block for {:?} has an empty translation",
                    b.from
                ));
            } else if let Some(other) = seen_text.insert(b.to.as_str(), b.from.as_str()) {
                if other != b.from {
                    problems.push(format!(
                        "{other:?} and {:?} both translate to {:?}",
                        b.from, b.to
                    ));
                }
            }
        }
        for (from, r) in &self.identifiers {
            if r.to.is_empty() {
                problems.push(format!("{from} has an empty translation"));
            }
        }
        let mut words = HashSet::new();
        let mut texts = Vec::new();
        for file in walk(repo)? {
            if let Ok(text) = std::fs::read_to_string(&file) {
                for (_, w) in identifier_words(&text) {
                    words.insert(w.to_string());
                }
                texts.push(text);
            }
        }
        for b in &self.text {
            if !b.to.is_empty() && texts.iter().any(|t| t.contains(b.to.as_str())) {
                problems.push(format!(
                    "translated text {:?} already occurs in the repo",
                    b.to
                ));
            }
        }
        for (from, r) in &self.identifiers {
            if r.to != *from && words.contains(&r.to) {
                problems.push(format!(
                    "{} (for {from}) already occurs in the repo; untranslate could not tell them apart",
                    r.to
                ));
            }
        }
        Ok(problems)
    }
}

/// Directories never touched.
const SKIP_DIRS: &[&str] = &[
    ".git",
    "target",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    ".hg",
];

/// All files under `root`, sorted, minus skipped directories and the
/// glossary itself.
pub fn walk(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("{}: {e}", dir.display()))?;
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            let ft = entry
                .file_type()
                .map_err(|e| format!("{}: {e}", path.display()))?;
            if ft.is_dir() {
                if !SKIP_DIRS.contains(&name.as_str()) {
                    stack.push(path);
                }
            } else if ft.is_file() && name != FILE_NAME {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// ASCII identifier runs with their byte offsets.
pub fn identifier_words(text: &str) -> Vec<(usize, &str)> {
    let bytes = text.as_bytes();
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if is_word(bytes[i]) {
            let start = i;
            while i < bytes.len() && is_word(bytes[i]) {
                i += 1;
            }
            out.push((start, &text[start..i]));
        } else {
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ident::{translate, Options, Role};
    use crate::lexicon::Lexicon;

    fn tr(ident: &str, role: Role) -> Translation {
        translate(Lexicon::core(), ident, role, &Options::default()).unwrap()
    }

    #[test]
    fn round_trips_through_toml() {
        let mut g = Glossary::default();
        g.record(&tr("get_user_name", Role::Fn));
        g.add_text("Return the name.", "fa …\n\"quoted\"");
        g.files.push(("users.py".into(), "brukarar.py".into()));
        let parsed = Glossary::parse(&g.render()).unwrap();
        assert_eq!(parsed.identifiers, g.identifiers);
        assert_eq!(parsed.text, g.text);
        assert_eq!(parsed.files, g.files);
    }

    #[test]
    fn records_are_stable_and_collisions_get_ordinals() {
        let mut g = Glossary::default();
        assert_eq!(
            g.record(&tr("user_name", Role::Var)),
            "namn_ad_den_brukaren"
        );
        // `navn` is neuter with a zero plural, so the plural collides.
        assert_eq!(
            g.record(&tr("user_names", Role::Var)),
            "namn_ad_den_brukaren_thaanii"
        );
        assert_eq!(
            g.record(&tr("user_name", Role::Var)),
            "namn_ad_den_brukaren"
        );
    }

    #[test]
    fn identifier_words_are_ascii_runs() {
        let words: Vec<&str> = identifier_words("a.b_c(dé, 3x)")
            .into_iter()
            .map(|(_, w)| w)
            .collect();
        assert_eq!(words, vec!["a", "b_c", "d", "3x"]);
    }
}
