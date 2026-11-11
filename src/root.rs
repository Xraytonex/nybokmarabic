//! Root extraction (SPEC §4.8): Norwegian infinitive to consonantal root.

use serde::Serialize;

use crate::text::is_vowel;

/// A consonantal root. Radicals are strings because the digraphs `sh` and
/// `kh` count as one consonant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Root {
    pub radicals: Vec<String>,
    /// How many radicals were padded in (`w`, `y`, or a hamza for a verb
    /// with no consonants at all).
    pub padded: usize,
}

impl Root {
    pub fn is_quadriliteral(&self) -> bool {
        self.radicals.len() == 4
    }

    pub fn is_weak(&self) -> bool {
        self.padded > 0
    }

    /// `S-K-R-V` style display.
    pub fn display(&self) -> String {
        self.radicals
            .iter()
            .map(|r| r.to_uppercase())
            .collect::<Vec<_>>()
            .join("-")
    }

    pub fn radical(&self, i: usize) -> &str {
        &self.radicals[i]
    }
}

/// Split a word into consonant tokens, applying the digraph rule:
/// `skj` and `sj` become `sh`, `kj` becomes `kh`. Vowels and non-letters are
/// skipped.
pub fn consonants(word: &str) -> Vec<String> {
    let chars: Vec<char> = word.to_lowercase().chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let rest: String = chars[i..].iter().take(3).collect();
        if rest.starts_with("skj") {
            out.push("sh".to_string());
            i += 3;
        } else if rest.starts_with("sj") {
            out.push("sh".to_string());
            i += 2;
        } else if rest.starts_with("kj") {
            out.push("kh".to_string());
            i += 2;
        } else {
            let c = chars[i];
            if c.is_alphabetic() && !is_vowel(c) {
                out.push(c.to_string());
            }
            i += 1;
        }
    }
    out
}

/// Extract the root of a Norwegian infinitive.
///
/// 1. Drop a final `-e` or `-a` (Nynorsk allows a-infinitives). Since vowels
///    are dropped in step 2 anyway, this step has no observable effect. It is
///    kept because it is in the regulation.
/// 2. Keep the consonants in order, digraphs counted once.
/// 3. Three consonants: triliteral. Four or more: the first four,
///    quadriliteral. Fewer than three: pad with `w`, then `y`.
/// 4. A verb with no consonants at all (Bokmål `eie`) starts with a glottal
///    stop in careful speech, so it gets a hamza: `ʾ-w-y`.
pub fn extract(infinitive: &str) -> Root {
    let lower = infinitive.trim().to_lowercase();
    let stem = lower
        .strip_suffix('e')
        .or_else(|| lower.strip_suffix('a'))
        .unwrap_or(&lower);
    let mut radicals = consonants(stem);
    radicals.truncate(4);
    let mut padded = 0;
    if radicals.is_empty() {
        radicals.push("ʾ".to_string());
        padded += 1;
    }
    for pad in ["w", "y"] {
        if radicals.len() < 3 {
            radicals.push(pad.to_string());
            padded += 1;
        }
    }
    Root { radicals, padded }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(word: &str) -> String {
        extract(word).display()
    }

    #[test]
    fn triliteral() {
        assert_eq!(r("hente"), "H-N-T");
        assert_eq!(r("lagre"), "L-G-R");
    }

    #[test]
    fn quadriliteral_keeps_first_four() {
        // The brief hand-computed S-K-R; by its own rule skrive has four.
        assert_eq!(r("skrive"), "S-K-R-V");
        assert_eq!(r("oppdatere"), "P-P-D-T");
    }

    #[test]
    fn digraphs_count_once() {
        assert_eq!(r("sjekke"), "SH-K-K");
        assert_eq!(r("skjule"), "SH-L-W");
        assert_eq!(r("søkje"), "S-KH-W");
    }

    #[test]
    fn weak_roots_pad_with_w_then_y() {
        assert_eq!(r("lese"), "L-S-W");
        assert_eq!(r("se"), "S-W-Y");
        assert!(extract("se").is_weak());
    }

    #[test]
    fn a_infinitive_gives_the_same_root() {
        assert_eq!(r("skriva"), r("skrive"));
    }

    #[test]
    fn no_consonants_gets_a_hamza() {
        assert_eq!(r("eie"), "ʾ-W-Y");
    }
}
