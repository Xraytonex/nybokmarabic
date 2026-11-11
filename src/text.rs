//! Letters, vowels, the parity rule (SPEC §4.2) and the small enums every
//! other module shares.

use serde::Serialize;

/// Vowel letters, Norwegian and Arabic transliteration alike. Accented
/// Norwegian vowels (é, ò, ô …) count as vowels too.
const VOWELS: &[char] = &[
    'a', 'e', 'i', 'o', 'u', 'y', 'æ', 'ø', 'å', 'é', 'è', 'ê', 'ó', 'ò', 'ô', 'à', 'ā', 'ī', 'ū',
];

pub fn is_vowel(c: char) -> bool {
    c.to_lowercase().any(|l| VOWELS.contains(&l))
}

/// Number of letters in a word. Hyphens, spaces and digits are not letters;
/// æ, ø and å are one letter each.
pub fn letter_count(word: &str) -> usize {
    word.chars().filter(|c| c.is_alphabetic()).count()
}

/// Number of vowel groups, a cheap syllable count used by the regular
/// inflection fallback for words that are not in the dictionary.
pub fn syllables(word: &str) -> usize {
    let mut count = 0;
    let mut in_vowel = false;
    for c in word.chars() {
        let v = is_vowel(c);
        if v && !in_vowel {
            count += 1;
        }
        in_vowel = v;
    }
    count
}

/// The two written standards of Norwegian.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Standard {
    #[serde(rename = "bm")]
    Bokmal,
    #[serde(rename = "nn")]
    Nynorsk,
}

impl Standard {
    /// The parity rule: count the letters of the Bokmål lemma. Odd selects
    /// Bokmål, even selects Nynorsk.
    pub fn for_lemma(bokmal_lemma: &str) -> Self {
        if letter_count(bokmal_lemma) % 2 == 1 {
            Standard::Bokmal
        } else {
            Standard::Nynorsk
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Standard::Bokmal => "bm",
            Standard::Nynorsk => "nn",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Gender {
    #[serde(rename = "m")]
    Masculine,
    #[serde(rename = "f")]
    Feminine,
    #[serde(rename = "n")]
    Neuter,
}

impl Gender {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "m" => Some(Gender::Masculine),
            "f" => Some(Gender::Feminine),
            "n" => Some(Gender::Neuter),
            _ => None,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Gender::Masculine => "m",
            Gender::Feminine => "f",
            Gender::Neuter => "n",
        }
    }

    /// Arabic verb agreement has two genders. Neuter nouns take feminine
    /// agreement, by analogy with Arabic non-human plurals (SPEC §4.7).
    pub fn verb_agreement(self) -> Gender {
        match self {
            Gender::Masculine => Gender::Masculine,
            _ => Gender::Feminine,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Number {
    Sg,
    Pl,
    Dual,
}

impl Number {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "sg" | "singular" => Some(Number::Sg),
            "pl" | "plural" => Some(Number::Pl),
            "dual" | "du" => Some(Number::Dual),
            _ => None,
        }
    }
}

/// Drop one trailing letter from `word` if it is `letter`.
pub fn strip_final(word: &str, letter: char) -> &str {
    word.strip_suffix(letter).unwrap_or(word)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parity_examples_from_the_brief() {
        assert_eq!(Standard::for_lemma("bruker"), Standard::Nynorsk);
        assert_eq!(Standard::for_lemma("navn"), Standard::Nynorsk);
        assert_eq!(Standard::for_lemma("ikke"), Standard::Nynorsk);
        assert_eq!(Standard::for_lemma("hus"), Standard::Bokmal);
    }

    #[test]
    fn norwegian_letters_count_once() {
        assert_eq!(letter_count("nøkkel"), 6);
        assert_eq!(letter_count("e-post"), 5);
        assert_eq!(letter_count("åtte"), 4);
    }

    #[test]
    fn syllables_counts_vowel_groups() {
        assert_eq!(syllables("hus"), 1);
        assert_eq!(syllables("eple"), 2);
        assert_eq!(syllables("brukeren"), 3);
    }

    #[test]
    fn neuter_agrees_feminine() {
        assert_eq!(Gender::Neuter.verb_agreement(), Gender::Feminine);
        assert_eq!(Gender::Masculine.verb_agreement(), Gender::Masculine);
    }
}
