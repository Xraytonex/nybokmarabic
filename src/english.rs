//! A tiny English lemmatizer: enough morphology to map identifier words
//! (`users`, `parsed`, `loading`) back to lexicon keys.

use serde::Serialize;

/// What the English surface form said about the word.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Inflection {
    Base,
    Plural,
    PastParticiple,
    Gerund,
    ThirdPerson,
    /// `parser`, `reader`: someone who does the verb.
    Agent,
}

/// Candidate (lemma, inflection) pairs for an English word, most specific
/// first. Irregular forms are handled by the lexicon (`en_pl`, `en_pp`).
pub fn candidates(word: &str) -> Vec<(String, Inflection)> {
    let w = word.to_lowercase();
    let mut out = vec![(w.clone(), Inflection::Base)];
    let mut push = |s: String, i: Inflection| {
        if !s.is_empty() && !out.iter().any(|(o, oi)| *o == s && *oi == i) {
            out.push((s, i));
        }
    };
    if let Some(stem) = w.strip_suffix("ies") {
        push(format!("{stem}y"), Inflection::Plural);
        push(format!("{stem}y"), Inflection::ThirdPerson);
    }
    for suffix in ["sses", "xes", "ches", "shes", "zes", "ses"] {
        if w.ends_with(suffix) {
            push(w[..w.len() - 2].to_string(), Inflection::Plural);
            push(w[..w.len() - 2].to_string(), Inflection::ThirdPerson);
        }
    }
    if let Some(stem) = w.strip_suffix('s') {
        if !w.ends_with("ss") {
            push(stem.to_string(), Inflection::Plural);
            push(stem.to_string(), Inflection::ThirdPerson);
        }
    }
    if let Some(stem) = w.strip_suffix("ied") {
        push(format!("{stem}y"), Inflection::PastParticiple);
    }
    if let Some(stem) = w.strip_suffix("ed") {
        push(stem.to_string(), Inflection::PastParticiple);
        push(format!("{stem}e"), Inflection::PastParticiple);
        if let Some(single) = undouble(stem) {
            push(single, Inflection::PastParticiple);
        }
    }
    if let Some(stem) = w.strip_suffix("ing") {
        push(stem.to_string(), Inflection::Gerund);
        push(format!("{stem}e"), Inflection::Gerund);
        if let Some(single) = undouble(stem) {
            push(single, Inflection::Gerund);
        }
    }
    for (suffix, replacement) in [("er", ""), ("er", "e"), ("r", ""), ("or", "e"), ("or", "")] {
        if let Some(stem) = w.strip_suffix(suffix) {
            if stem.len() >= 2 {
                push(format!("{stem}{replacement}"), Inflection::Agent);
                if replacement.is_empty() {
                    if let Some(single) = undouble(stem) {
                        push(single, Inflection::Agent);
                    }
                }
            }
        }
    }
    out
}

/// `stopp` → `stop`, `runn` → `run`.
fn undouble(stem: &str) -> Option<String> {
    let chars: Vec<char> = stem.chars().collect();
    let n = chars.len();
    if n >= 3 && chars[n - 1] == chars[n - 2] && !"aeiou".contains(chars[n - 1]) {
        Some(chars[..n - 1].iter().collect())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn has(word: &str, lemma: &str, infl: Inflection) -> bool {
        candidates(word).contains(&(lemma.to_string(), infl))
    }

    #[test]
    fn plurals() {
        assert!(has("users", "user", Inflection::Plural));
        assert!(has("entries", "entry", Inflection::Plural));
        assert!(has("boxes", "box", Inflection::Plural));
        assert!(has("classes", "class", Inflection::Plural));
        assert!(!has("class", "clas", Inflection::Plural));
    }

    #[test]
    fn participles_and_gerunds() {
        assert!(has("parsed", "parse", Inflection::PastParticiple));
        assert!(has("loaded", "load", Inflection::PastParticiple));
        assert!(has("stopped", "stop", Inflection::PastParticiple));
        assert!(has("loading", "load", Inflection::Gerund));
        assert!(has("parsing", "parse", Inflection::Gerund));
        assert!(has("running", "run", Inflection::Gerund));
    }

    #[test]
    fn agents() {
        assert!(has("parser", "parse", Inflection::Agent));
        assert!(has("reader", "read", Inflection::Agent));
        assert!(has("runner", "run", Inflection::Agent));
        assert!(has("validator", "validate", Inflection::Agent));
    }
}
