//! Shorthand in identifiers (SPEC §16): abbreviations that stand for English
//! words (`fh` is "file handle") and acronyms kept as written (`HTTP`).
//!
//! The embedded table is `lexicon/shorthand.toml`; a glossary can add more
//! under `[proposed]` with `pos = "abbrev"` or `pos = "acronym"`.

use std::collections::BTreeMap;

use toml::Value;

pub const SHORTHAND_TOML: &str = include_str!("../lexicon/shorthand.toml");

#[derive(Clone, Debug, Default)]
pub struct Shorthand {
    /// Lower-case word → the English words it stands for.
    abbreviations: BTreeMap<String, Vec<String>>,
    /// Lower-case word → the acronym as written (`http` → `HTTP`).
    acronyms: BTreeMap<String, String>,
}

/// Identifier words are lower-case ASCII after splitting; shorthand keys must
/// be too, or they could never match.
fn valid_word(word: &str) -> bool {
    !word.is_empty() && word.chars().all(|c| c.is_ascii_lowercase())
}

impl Shorthand {
    /// The embedded table.
    pub fn core() -> Result<Shorthand, String> {
        Shorthand::parse(SHORTHAND_TOML)
    }

    pub fn parse(src: &str) -> Result<Shorthand, String> {
        let table: toml::Table = src.parse().map_err(|e| format!("shorthand: {e}"))?;
        let mut out = Shorthand::default();
        if let Some(abbrevs) = table.get("abbreviations") {
            let abbrevs = abbrevs
                .as_table()
                .ok_or("shorthand: [abbreviations] must be a table")?;
            for (word, expansion) in abbrevs {
                let expansion = expansion
                    .as_str()
                    .ok_or_else(|| format!("shorthand: {word} must expand to a string"))?;
                out.add_abbreviation(word, expansion)?;
            }
        }
        if let Some(acronyms) = table.get("acronyms") {
            let words = acronyms
                .get("words")
                .and_then(Value::as_array)
                .ok_or("shorthand: [acronyms] needs a `words` list")?;
            for w in words {
                let w = w.as_str().ok_or("shorthand: acronyms must be strings")?;
                out.add_acronym(w)?;
            }
        }
        Ok(out)
    }

    pub fn add_abbreviation(&mut self, word: &str, expansion: &str) -> Result<(), String> {
        let words: Vec<String> = expansion
            .split_whitespace()
            .map(str::to_lowercase)
            .collect();
        if !valid_word(word) {
            return Err(format!(
                "abbreviation {word:?}: must be lower-case letters a-z"
            ));
        }
        if words.is_empty() || !words.iter().all(|w| valid_word(w)) {
            return Err(format!(
                "abbreviation {word}: expansion {expansion:?} must be lower-case English words"
            ));
        }
        if self.acronyms.contains_key(word) {
            return Err(format!("{word} is both an abbreviation and an acronym"));
        }
        self.abbreviations.insert(word.to_string(), words);
        Ok(())
    }

    pub fn add_acronym(&mut self, written: &str) -> Result<(), String> {
        let key = written.to_lowercase();
        if !valid_word(&key) || written.chars().count() < 2 {
            return Err(format!(
                "acronym {written:?}: must be two or more letters a-z"
            ));
        }
        if self.abbreviations.contains_key(&key) {
            return Err(format!("{key} is both an abbreviation and an acronym"));
        }
        self.acronyms.insert(key, written.to_string());
        Ok(())
    }

    /// The English words `word` stands for.
    pub fn expansion(&self, word: &str) -> Option<&[String]> {
        self.abbreviations.get(word).map(Vec::as_slice)
    }

    /// The acronym as written, if `word` is one.
    pub fn acronym(&self, word: &str) -> Option<&str> {
        self.acronyms.get(word).map(String::as_str)
    }

    pub fn abbreviations(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        self.abbreviations.iter()
    }

    pub fn acronyms(&self) -> impl Iterator<Item = &String> {
        self.acronyms.keys()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedded_table_parses() {
        let s = Shorthand::core().unwrap();
        assert_eq!(s.expansion("fh").unwrap(), ["file", "handle"]);
        assert_eq!(s.acronym("http"), Some("HTTP"));
        assert_eq!(s.acronym("gzip"), Some("gzip"));
        assert_eq!(s.acronym("fh"), None);
    }

    #[test]
    fn keys_must_be_plain_lower_case_words() {
        let mut s = Shorthand::default();
        assert!(s.add_abbreviation("Fh", "file handle").is_err());
        assert!(s.add_abbreviation("fh", "").is_err());
        assert!(s.add_abbreviation("fh", "file-handle").is_err());
        s.add_abbreviation("fh", "File Handle").unwrap();
        assert_eq!(s.expansion("fh").unwrap(), ["file", "handle"]);
        assert!(s.add_acronym("X").is_err());
        assert!(s.add_acronym("C3PO").is_err());
    }

    #[test]
    fn a_word_is_never_both() {
        let mut s = Shorthand::default();
        s.add_acronym("CRC").unwrap();
        assert!(s.add_abbreviation("crc", "check").is_err());
        s.add_abbreviation("fh", "file handle").unwrap();
        assert!(s.add_acronym("FH").is_err());
    }
}
