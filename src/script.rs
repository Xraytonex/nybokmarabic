//! Phase 2 (SPEC §9): prose mode rendered in Arabic script, right to left.
//! Off by default; applying it to identifiers needs `--i-understand-bidi`.
//!
//! Arabic writes three short vowels as diacritics. Norwegian has nine vowel
//! letters, so the six Arabic lacks are borrowed from orthographies that
//! already solved the problem, mostly Uyghur (see SPEC.md for the table).

const FATHA: char = '\u{064E}';
const KASRA: char = '\u{0650}';
const DAMMA: char = '\u{064F}';
const SHADDA: char = '\u{0651}';
const ZWNJ: char = '\u{200C}';

/// Articles as prose writes them after assimilation.
const ARTICLES: &[&str] = &[
    "al", "ad", "as", "at", "ar", "az", "an", "ash", "ath", "adh", "aṣ", "aḍ", "aṭ", "aẓ",
];

pub const BIDI_WARNING: &str =
    "warning: Arabic script in identifiers mixes right-to-left text into \
source code. Bidirectional text is how Trojan Source attacks hide code (CVE-2021-42574; \
homoglyphs: CVE-2021-42694). Expect linters, diff viewers, grep and colleagues to break.";

fn consonant(token: &str) -> Option<&'static str> {
    Some(match token {
        "sh" | "skj" | "sj" => "ش",
        "kh" | "kj" => "خ",
        "th" => "ث",
        "dh" => "ذ",
        "gh" => "غ",
        "b" => "ب",
        "t" => "ت",
        "d" => "د",
        "r" => "ر",
        "z" => "ز",
        "s" => "س",
        "ṣ" => "ص",
        "ḍ" => "ض",
        "ṭ" => "ط",
        "ẓ" => "ظ",
        "ʿ" => "ع",
        "ʾ" => "ء",
        "ḥ" => "ح",
        "f" => "ف",
        "q" => "ق",
        "k" | "c" => "ك",
        "l" => "ل",
        "m" => "م",
        "n" => "ن",
        "h" => "ه",
        "w" => "و",
        "j" | "y" => "ي",
        "g" => "گ",
        "p" => "پ",
        "v" => "ڤ",
        "x" => "كس",
        _ => return None,
    })
}

/// (medial form, word-initial form) for vowels.
fn vowel(c: char) -> Option<(String, String)> {
    let pair = |m: &str, i: &str| Some((m.to_string(), i.to_string()));
    match c {
        'a' => pair(&FATHA.to_string(), &format!("أ{FATHA}")),
        'i' => pair(&KASRA.to_string(), &format!("إ{KASRA}")),
        'u' => pair(&DAMMA.to_string(), &format!("أ{DAMMA}")),
        'ā' => pair("ا", "آ"),
        'ī' => pair("ي", "إي"),
        'ū' => pair("و", "أو"),
        'e' | 'é' | 'è' | 'ê' => pair("ې", "ئې"),
        'o' | 'ó' | 'ò' | 'ô' => pair("و", "ئو"),
        'y' => pair("ۈ", "ئۈ"),
        'æ' => pair("ە", "ئە"),
        'ø' => pair("ۆ", "ئۆ"),
        'å' => pair("ۏ", "ئۏ"),
        _ => None,
    }
}

fn is_arabic_vowel(c: char) -> bool {
    matches!(c, 'a' | 'i' | 'u' | 'ā' | 'ī' | 'ū')
}

/// One morpheme (no hyphens). `shadda_first` doubles the first consonant
/// (after an assimilated article).
fn morpheme(m: &str, shadda_first: bool) -> String {
    let chars: Vec<char> = m.to_lowercase().chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let mut first = true;
    let mut prev_consonant: Option<String> = None;
    while i < chars.len() {
        let c = chars[i];
        // y is a consonant next to an Arabic vowel (yaskurur), else a vowel.
        let y_consonant = c == 'y'
            && (chars.get(i + 1).is_some_and(|&n| is_arabic_vowel(n))
                || (i > 0 && is_arabic_vowel(chars[i - 1])));
        if c != 'y' || !y_consonant {
            if let Some((medial, initial)) = vowel(c) {
                out.push_str(if first { &initial } else { &medial });
                first = false;
                prev_consonant = None;
                i += 1;
                continue;
            }
        }
        let rest: String = chars[i..].iter().take(3).collect();
        let token = ["skj", "sj", "kj", "sh", "kh", "th", "dh", "gh"]
            .iter()
            .find(|d| rest.starts_with(*d))
            .map(|d| d.to_string())
            .unwrap_or_else(|| c.to_string());
        let len = token.chars().count();
        match consonant(&token) {
            Some(letter) => {
                if prev_consonant.as_deref() == Some(letter) {
                    out.push(SHADDA);
                } else {
                    out.push_str(letter);
                    if first && shadda_first {
                        out.push(SHADDA);
                    }
                }
                prev_consonant = Some(letter.to_string());
            }
            None => {
                out.push_str(&digit_or_punct(c));
                prev_consonant = None;
            }
        }
        first = false;
        i += len;
    }
    out
}

fn digit_or_punct(c: char) -> String {
    match c {
        '0'..='9' => char::from_u32('٠' as u32 + (c as u32 - '0' as u32))
            .map(String::from)
            .unwrap_or_default(),
        '?' => "؟".into(),
        ',' => "،".into(),
        ';' => "؛".into(),
        other => other.to_string(),
    }
}

/// One space-separated word: articles join their noun (`ال` plus a shadda on
/// a sun letter), other hyphens become a zero-width non-joiner.
fn word(w: &str, separator: char) -> String {
    let parts: Vec<&str> = w.split('-').collect();
    let mut out = String::new();
    let mut shadda_next = false;
    for (i, part) in parts.iter().enumerate() {
        let is_article = i + 1 < parts.len() && ARTICLES.contains(&part.to_lowercase().as_str());
        if is_article {
            out.push_str("ال");
            shadda_next = part.to_lowercase() != "al";
            continue;
        }
        if i > 0 && !out.ends_with("ال") {
            out.push(separator);
        }
        out.push_str(&morpheme(part, shadda_next));
        shadda_next = false;
    }
    out
}

/// Prose mode in Arabic script, for comments and docs.
pub fn to_arabic(prose: &str) -> String {
    prose
        .split(' ')
        .map(|w| word(w, ZWNJ))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Arabic script for an identifier: words joined with `_`. Only reachable
/// with `--i-understand-bidi`.
pub fn identifier_to_arabic(prose: &str) -> String {
    prose
        .split(' ')
        .map(|w| word(w, '_'))
        .collect::<Vec<_>>()
        .join("_")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn article_is_written_al_with_shadda_on_sun_letters() {
        assert_eq!(
            to_arabic("ad-den-brukaren"),
            "الدّېن\u{200C}بْرُكَرېن".replace('\u{0652}', "")
        );
    }

    #[test]
    fn norwegian_vowels_have_letters() {
        assert_eq!(to_arabic("æ"), "ئە");
        assert_eq!(to_arabic("ø"), "ئۆ");
        assert_eq!(to_arabic("å"), "ئۏ");
        assert_eq!(to_arabic("sær"), "سەر");
    }

    #[test]
    fn doubled_consonants_take_shadda() {
        assert_eq!(to_arabic("nøkkel"), "نۆكّېل");
        // kj is one consonant (خ), as in root extraction.
        assert_eq!(to_arabic("ikkje"), "إِكخې");
    }

    #[test]
    fn arabic_words_come_out_as_arabic() {
        assert_eq!(to_arabic("hal"), "هَل");
        assert_eq!(to_arabic("fī"), "في");
        assert_eq!(to_arabic("ʿalā"), "عَلا");
    }
}
