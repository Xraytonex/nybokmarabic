//! Numbers (SPEC §4.12): Norwegian numerals, Arabic order and Arabic noun
//! agreement.

use crate::lexicon::{Lexicon, Pos};
use crate::nominal::NounPhrase;
use crate::text::{Gender, Number};

/// The largest number `numeral` spells out.
pub const MAX: u64 = 999_999;

fn word(lex: &Lexicon, value: u64) -> String {
    lex.entries()
        .iter()
        .find(|e| e.pos == Pos::Num && e.value == Some(value))
        .map(|e| e.lemma().to_string())
        .unwrap_or_else(|| value.to_string())
}

/// 1–99: units before tens, joined with `wa`, the one order old Norwegian
/// counting and Arabic agree on.
fn below_hundred(lex: &Lexicon, n: u64) -> String {
    if n < 20 || n.is_multiple_of(10) {
        word(lex, n)
    } else {
        format!("{} wa {}", word(lex, n % 10), word(lex, n - n % 10))
    }
}

/// `hundre`, dual `hundrān` (Arabic miʾatān), or `tre hundre`.
fn scaled(lex: &Lexicon, count: u64, unit: u64) -> String {
    let unit_word = word(lex, unit);
    match count {
        1 => unit_word,
        2 => crate::nominal::dual(&unit_word),
        _ => format!("{} {}", below_thousand(lex, count), unit_word),
    }
}

fn below_thousand(lex: &Lexicon, n: u64) -> String {
    let mut parts = Vec::new();
    if n >= 100 {
        parts.push(scaled(lex, n / 100, 100));
    }
    if !n.is_multiple_of(100) || n == 0 {
        parts.push(below_hundred(lex, n % 100));
    }
    parts.join(" wa ")
}

/// Spell out `n` (0 to 999 999). Larger numbers are left as digits.
pub fn numeral(lex: &Lexicon, n: u64) -> String {
    if n > MAX {
        return n.to_string();
    }
    let mut parts = Vec::new();
    if n >= 1000 {
        parts.push(scaled(lex, n / 1000, 1000));
    }
    if !n.is_multiple_of(1000) || n == 0 {
        parts.push(below_thousand(lex, n % 1000));
    }
    parts.join(" wa ")
}

/// How a noun counted by `n` is inflected, following Arabic: 1 singular,
/// 2 dual (no numeral needed), 3–10 plural, 11 and up singular.
pub fn counted_number(n: u64) -> Number {
    let r = if n > 100 && !n.is_multiple_of(100) {
        n % 100
    } else {
        n
    };
    match r {
        2 => Number::Dual,
        3..=10 => Number::Pl,
        _ => Number::Sg,
    }
}

/// `ein`, `ei`, `eitt`: the numeral one in Nynorsk (parity: `en` has two
/// letters), agreeing with the counted noun and placed after it, as Arabic
/// places wāḥid after the noun.
pub fn one(gender: Gender) -> &'static str {
    match gender {
        Gender::Masculine => "ein",
        Gender::Feminine => "ei",
        Gender::Neuter => "eitt",
    }
}

/// Render a noun phrase counted by `count`: 1 puts `ein` after the noun,
/// 2 is the dual with no numeral, anything else puts the numeral first.
pub fn render_counted(lex: &Lexicon, mut np: NounPhrase, count: Option<u64>) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(n) = count {
        np.number = Some(counted_number(n));
        if n != 1 && n != 2 {
            out.push(numeral(lex, n));
        }
    }
    out.extend(np.render());
    if count == Some(1) {
        out.push(one(np.head_gender()).to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn n(v: u64) -> String {
        numeral(Lexicon::core(), v)
    }

    #[test]
    fn units_before_tens() {
        assert_eq!(n(22), "to wa tjue");
        assert_eq!(n(21), "ein wa tjue");
        assert_eq!(n(7), "sju");
        assert_eq!(n(40), "førti");
        assert_eq!(n(0), "null");
    }

    #[test]
    fn hundreds_and_thousands() {
        assert_eq!(n(100), "hundre");
        assert_eq!(n(200), "hundrān");
        assert_eq!(n(300), "tre hundre");
        assert_eq!(n(122), "hundre wa to wa tjue");
        assert_eq!(n(2000), "tusenān");
        assert_eq!(n(1222), "tusen wa hundrān wa to wa tjue");
        assert_eq!(n(1_000_000), "1000000");
    }

    #[test]
    fn arabic_counted_noun_agreement() {
        assert_eq!(counted_number(1), Number::Sg);
        assert_eq!(counted_number(2), Number::Dual);
        assert_eq!(counted_number(3), Number::Pl);
        assert_eq!(counted_number(10), Number::Pl);
        assert_eq!(counted_number(11), Number::Sg);
        assert_eq!(counted_number(22), Number::Sg);
        assert_eq!(counted_number(103), Number::Pl);
    }
}
