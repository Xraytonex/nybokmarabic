//! Identifier words the lexicon does not list as such (SPEC §16):
//! abbreviations (`fh`, `kwargs`) and closed compounds (`filename`,
//! `rmtree`). Each is read as the English words it stands for, and the
//! grammar then treats those like any other words.

use std::collections::HashMap;

use serde::Serialize;

use crate::classify::{classify_word, is_function_word, Context};
use crate::english::{plural, Inflection};
use crate::lexicon::{Lexicon, Pos};

/// How a word was read.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Via {
    Abbreviation,
    Compound,
}

/// A word as the translator reads it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub word: String,
    /// The identifier word it was read from, and how.
    pub from: Option<(String, Via)>,
}

/// A compound has at most this many pieces (`copy` `file` `obj`).
const MAX_PIECES: usize = 3;

/// Two-letter words that may be a piece of a compound (`isdir`, `onerror`,
/// `tolist`, `backup`). Two-letter abbreviations, acronyms and nouns
/// (`rm`, `io`, `id`) may be pieces too; nothing shorter may.
const SHORT_PIECES: &[&str] = &["in", "is", "no", "on", "to", "up"];

/// Words that are never a piece: they glue onto too many real English words
/// (`together` is not `to get her`).
const NEVER_PIECES: &[&str] = &[
    "the", "an", "am", "are", "was", "were", "be", "been", "being", "her", "him", "them", "you",
];

/// True if the grammar can read `word` without help, in some position.
pub fn known(lex: &Lexicon, word: &str) -> bool {
    [Context::VerbFirst, Context::NounPhrase, Context::Predicate]
        .into_iter()
        .any(|ctx| classify_word(lex, word, ctx).is_some())
}

/// Read every word: known words as they are, the rest expanded where they
/// can be. Words that cannot be read are kept, so the caller can name them.
pub fn read(lex: &Lexicon, words: &[String]) -> Vec<Reading> {
    words
        .iter()
        .flat_map(|w| {
            let expanded = (!known(lex, w)).then(|| expand(lex, w)).flatten();
            match expanded {
                Some((parts, via)) => parts
                    .into_iter()
                    .map(|p| Reading {
                        word: p,
                        from: Some((w.clone(), via)),
                    })
                    .collect(),
                None => vec![Reading {
                    word: w.clone(),
                    from: None,
                }],
            }
        })
        .collect()
}

/// The English words an unknown word stands for: an abbreviation first,
/// then a compound of known words.
pub fn expand(lex: &Lexicon, word: &str) -> Option<(Vec<String>, Via)> {
    let w = word.to_lowercase();
    if let Some(words) = abbreviation(lex, &w) {
        return Some((words, Via::Abbreviation));
    }
    compound(lex, &w).map(|words| (words, Via::Compound))
}

/// An abbreviation, or the plural of one (`kwargs` → keyword arguments).
fn abbreviation(lex: &Lexicon, w: &str) -> Option<Vec<String>> {
    if let Some(words) = lex.expansion(w) {
        return Some(words.to_vec());
    }
    let mut words = lex.expansion(w.strip_suffix('s')?)?.to_vec();
    let last = words.last_mut()?;
    *last = plural(last);
    Some(words)
}

/// Split `w` into the fewest known pieces; ties go to the split whose
/// shortest piece is longest, then to the longer first piece. Abbreviated
/// pieces are expanded (`rmtree` → remove tree).
fn compound(lex: &Lexicon, w: &str) -> Option<Vec<String>> {
    if w.len() < 4 || !w.chars().all(|c| c.is_ascii_lowercase()) {
        return None;
    }
    let mut cache = HashMap::new();
    let mut best: Option<Vec<&str>> = None;
    let mut pieces = Vec::new();
    search(lex, w, &mut pieces, &mut best, &mut cache);
    let mut best = best?;
    particle_last(lex, &mut best);
    Some(
        best.into_iter()
            .flat_map(|p| abbreviation(lex, p).unwrap_or_else(|| vec![p.to_string()]))
            .collect(),
    )
}

fn search<'w>(
    lex: &Lexicon,
    rest: &'w str,
    pieces: &mut Vec<&'w str>,
    best: &mut Option<Vec<&'w str>>,
    cache: &mut HashMap<&'w str, bool>,
) {
    if rest.is_empty() {
        if pieces.len() >= 2 && best.as_ref().is_none_or(|b| better(pieces, b)) {
            *best = Some(pieces.clone());
        }
        return;
    }
    if pieces.len() == MAX_PIECES {
        return;
    }
    for end in 2..=rest.len() {
        let (piece, tail) = rest.split_at(end);
        let ok = *cache.entry(piece).or_insert_with(|| piece_ok(lex, piece));
        if ok {
            pieces.push(piece);
            search(lex, tail, pieces, best, cache);
            pieces.pop();
        }
    }
}

/// English puts a verb's particle first in a closed compound (`upload`,
/// `download`); Norwegian says it after the verb (`laste opp`, `laste ned`).
fn particle_last(lex: &Lexicon, pieces: &mut [&str]) {
    if let [first, second] = pieces {
        let senses = lex.senses(first);
        let adverb = !senses.is_empty() && senses.iter().all(|s| s.entry.pos == Pos::Adv);
        let verb = lex
            .senses(second)
            .iter()
            .any(|s| s.entry.pos == Pos::Verb && s.inflection == Inflection::Base);
        if adverb && verb {
            pieces.swap(0, 1);
        }
    }
}

fn better(a: &[&str], b: &[&str]) -> bool {
    let key = |p: &[&str]| {
        (
            std::cmp::Reverse(p.len()),
            p.iter().map(|s| s.len()).min(),
            p.first().map(|s| s.len()),
        )
    };
    key(a) > key(b)
}

fn piece_ok(lex: &Lexicon, piece: &str) -> bool {
    if NEVER_PIECES.contains(&piece) {
        return false;
    }
    match piece.len() {
        0 | 1 => false,
        2 => {
            SHORT_PIECES.contains(&piece)
                || lex.expansion(piece).is_some()
                || lex.acronym(piece).is_some()
                || lex
                    .senses(piece)
                    .iter()
                    .any(|s| s.entry.pos == Pos::Noun && s.inflection == Inflection::Base)
        }
        // A short piece must be a word as written or a plain plural: `ser`
        // is not "someone who sees", and `ted` is not a past participle.
        3 | 4 => {
            lex.senses(piece)
                .iter()
                .any(|s| matches!(s.inflection, Inflection::Base | Inflection::Plural))
                || is_function_word(piece)
                || lex.acronym(piece).is_some()
                || lex.expansion(piece).is_some()
        }
        _ => known(lex, piece) || abbreviation(lex, piece).is_some(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exp(word: &str) -> Option<(Vec<String>, Via)> {
        expand(Lexicon::core(), word)
    }

    fn words(word: &str) -> Vec<String> {
        exp(word).unwrap().0
    }

    #[test]
    fn abbreviations_expand_and_pluralise() {
        assert_eq!(words("fh"), ["file", "handle"]);
        assert_eq!(exp("fh").unwrap().1, Via::Abbreviation);
        assert_eq!(words("kwargs"), ["keyword", "arguments"]);
        assert_eq!(words("fds"), ["file", "descriptors"]);
        assert_eq!(words("nargs"), ["number", "of", "arguments"]);
    }

    #[test]
    fn compounds_split_into_known_words() {
        assert_eq!(words("filename"), ["file", "name"]);
        assert_eq!(exp("filename").unwrap().1, Via::Compound);
        assert_eq!(words("readlines"), ["read", "lines"]);
        assert_eq!(words("rmtree"), ["remove", "tree"]);
        assert_eq!(words("copyfileobj"), ["copy", "file", "obj"]);
        assert_eq!(words("isdir"), ["is", "dir"]);
        assert_eq!(words("onerror"), ["on", "error"]);
        assert_eq!(words("httpserver"), ["http", "server"]);
    }

    #[test]
    fn a_leading_particle_follows_its_verb() {
        assert_eq!(words("upload"), ["load", "up"]);
        assert_eq!(words("download"), ["load", "down"]);
        assert_eq!(words("backup"), ["back", "up"], "no verb, no swap");
    }

    #[test]
    fn fewest_pieces_win() {
        // `whitespace` could be white+space or a three-way split; two wins.
        assert_eq!(words("whitespace"), ["white", "space"]);
    }

    #[test]
    fn some_words_are_never_split() {
        assert_eq!(exp("together"), None);
        assert_eq!(exp("frobnicate"), None);
        assert_eq!(exp("utf8"), None, "digits pass through instead");
        assert_eq!(exp("x"), None);
    }

    #[test]
    fn known_words_are_read_as_they_are() {
        let lex = Lexicon::core();
        let r = read(lex, &["get".into(), "filename".into(), "zzz".into()]);
        let got: Vec<(&str, Option<&str>)> = r
            .iter()
            .map(|r| (r.word.as_str(), r.from.as_ref().map(|f| f.0.as_str())))
            .collect();
        assert_eq!(
            got,
            [
                ("get", None),
                ("file", Some("filename")),
                ("name", Some("filename")),
                ("zzz", None)
            ]
        );
    }
}
