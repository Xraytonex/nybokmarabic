//! Orthography (SPEC §4.3): prose mode to code mode, and identifier case
//! conventions.

use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Case {
    Snake,
    Camel,
    Pascal,
    Screaming,
    Kebab,
}

impl Case {
    pub fn parse(s: &str) -> Option<Case> {
        Some(match s {
            "snake" => Case::Snake,
            "camel" => Case::Camel,
            "pascal" => Case::Pascal,
            "screaming" | "upper" | "constant" => Case::Screaming,
            "kebab" => Case::Kebab,
            _ => return None,
        })
    }

    fn join(self, parts: &[String]) -> String {
        match self {
            Case::Snake => parts.join("_").to_lowercase(),
            Case::Kebab => parts.join("-").to_lowercase(),
            Case::Screaming => parts.join("_").to_uppercase(),
            Case::Camel => parts
                .iter()
                .enumerate()
                .map(|(i, p)| if i == 0 { p.clone() } else { capitalize(p) })
                .collect(),
            Case::Pascal => parts.iter().map(|p| capitalize(p)).collect(),
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

/// An identifier taken apart: leading/trailing underscores, lowercase
/// words, and the case convention it was written in.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Split {
    pub prefix: String,
    pub words: Vec<String>,
    pub suffix: String,
    pub case: Case,
}

/// Split an identifier using its own case convention. Acronyms stay whole:
/// `parseJSONData` is `parse`, `json`, `data`.
pub fn split_identifier(ident: &str) -> Split {
    let core = ident.trim_matches('_');
    let start = ident.len() - ident.trim_start_matches('_').len();
    let prefix = ident[..start].to_string();
    let suffix = ident[start + core.len()..].to_string();
    let case = if core.contains('-') {
        Case::Kebab
    } else if core.contains('_') {
        if core.chars().any(|c| c.is_lowercase()) {
            Case::Snake
        } else {
            Case::Screaming
        }
    } else if core.chars().next().is_some_and(char::is_uppercase) {
        if core.chars().all(|c| !c.is_lowercase()) && core.len() > 1 {
            Case::Screaming
        } else {
            Case::Pascal
        }
    } else if core.chars().any(char::is_uppercase) {
        Case::Camel
    } else {
        Case::Snake
    };
    let mut words = Vec::new();
    for chunk in core.split(['_', '-']).filter(|s| !s.is_empty()) {
        words.extend(split_camel(chunk));
    }
    Split {
        prefix,
        words: words.into_iter().map(|w| w.to_lowercase()).collect(),
        suffix,
        case,
    }
}

fn split_camel(chunk: &str) -> Vec<String> {
    let chars: Vec<char> = chunk.chars().collect();
    let mut out = Vec::new();
    let mut cur = String::new();
    for i in 0..chars.len() {
        let c = chars[i];
        let prev = if i > 0 { Some(chars[i - 1]) } else { None };
        let next = chars.get(i + 1).copied();
        let boundary = match prev {
            None => false,
            Some(p) => {
                (c.is_uppercase() && p.is_lowercase())
                    || (c.is_uppercase()
                        && p.is_uppercase()
                        && next.is_some_and(char::is_lowercase))
                    || (c.is_ascii_digit() != p.is_ascii_digit() && !c.is_ascii_digit())
            }
        };
        if boundary && !cur.is_empty() {
            out.push(std::mem::take(&mut cur));
        }
        cur.push(c);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Fold one prose morpheme to ASCII code mode.
pub fn fold_morpheme(prose: &str) -> String {
    let mut out = String::new();
    for c in prose.chars() {
        let folded: &str = match c {
            'ā' | 'å' => "aa",
            'Ā' | 'Å' => "Aa",
            'ī' => "ii",
            'Ī' => "Ii",
            'ū' => "uu",
            'Ū' => "Uu",
            'æ' => "ae",
            'Æ' => "Ae",
            'ø' => "oe",
            'Ø' => "Oe",
            'ʿ' => "3",
            'ʾ' => "2",
            'ḥ' | 'Ḥ' => "7",
            'ṣ' | 'Ṣ' => "9",
            'ṭ' | 'Ṭ' => "6",
            'ḍ' => "d",
            'Ḍ' => "D",
            'ẓ' => "z",
            'Ẓ' => "Z",
            'é' | 'è' | 'ê' => "e",
            'ó' | 'ò' | 'ô' => "o",
            'à' => "a",
            _ if c.is_ascii_alphanumeric() => {
                out.push(c);
                continue;
            }
            _ => continue,
        };
        out.push_str(folded);
    }
    out
}

/// Words no identifier may be, in the languages translated most often.
/// A translation that lands on one is addressed with the vocative.
const RESERVED: &[&str] = &[
    "and",
    "as",
    "assert",
    "async",
    "await",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "def",
    "default",
    "del",
    "do",
    "elif",
    "else",
    "enum",
    "except",
    "export",
    "extends",
    "false",
    "finally",
    "fn",
    "for",
    "from",
    "func",
    "function",
    "global",
    "go",
    "if",
    "impl",
    "import",
    "in",
    "interface",
    "is",
    "lambda",
    "let",
    "loop",
    "match",
    "mod",
    "mut",
    "new",
    "nil",
    "none",
    "nonlocal",
    "not",
    "null",
    "or",
    "pass",
    "pub",
    "raise",
    "return",
    "self",
    "static",
    "struct",
    "super",
    "switch",
    "this",
    "throw",
    "trait",
    "true",
    "try",
    "type",
    "use",
    "var",
    "void",
    "where",
    "while",
    "with",
    "yield",
];

/// The vocative particle `yā`, in code mode.
pub const VOCATIVE: &str = "yaa";

/// Prose mode to code mode. Spaces and hyphens become case boundaries. An
/// identifier that would start with a digit, or that would be a keyword, is
/// preceded by the vocative particle.
pub fn to_code(prose: &str, case: Case) -> String {
    let mut parts: Vec<String> = prose
        .split(|c: char| c.is_whitespace() || c == '-' || c == '_')
        .map(fold_morpheme)
        .filter(|s| !s.is_empty())
        .collect();
    let starts_with_digit = parts
        .first()
        .and_then(|p| p.chars().next())
        .is_some_and(|c| c.is_ascii_digit());
    let is_reserved = parts.len() == 1 && RESERVED.contains(&parts[0].to_lowercase().as_str());
    if starts_with_digit || is_reserved {
        parts.insert(0, VOCATIVE.to_string());
    }
    let parts: Vec<String> = match case {
        Case::Camel | Case::Pascal => parts.iter().map(|p| p.to_lowercase()).collect(),
        _ => parts,
    };
    case.join(&parts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_table_from_the_spec() {
        assert_eq!(fold_morpheme("ʿalā"), "3alaa");
        assert_eq!(fold_morpheme("taḥta"), "ta7ta");
        assert_eq!(fold_morpheme("nøkkel"), "noekkel");
        assert_eq!(fold_morpheme("åtte"), "aatte");
        assert_eq!(fold_morpheme("særs"), "saers");
        assert_eq!(fold_morpheme("suʾāl"), "su2aal");
        assert_eq!(fold_morpheme("fī"), "fii");
    }

    #[test]
    fn vocative_rescues_digit_initial_identifiers() {
        assert_eq!(to_code("ʿalā", Case::Snake), "yaa_3alaa");
        assert_eq!(
            to_code("ʿalā ad-det-klikket", Case::Camel),
            "yaa3alaaAdDetKlikket"
        );
        assert_eq!(to_code("type", Case::Snake), "yaa_type");
    }

    #[test]
    fn case_conventions() {
        let p = "uhnut namn ad-den-brukaren";
        assert_eq!(to_code(p, Case::Snake), "uhnut_namn_ad_den_brukaren");
        assert_eq!(to_code(p, Case::Camel), "uhnutNamnAdDenBrukaren");
        assert_eq!(to_code(p, Case::Pascal), "UhnutNamnAdDenBrukaren");
        assert_eq!(to_code(p, Case::Screaming), "UHNUT_NAMN_AD_DEN_BRUKAREN");
    }

    #[test]
    fn splitting() {
        let s = split_identifier("getUserName");
        assert_eq!(
            (s.words.clone(), s.case),
            (
                vec!["get".into(), "user".into(), "name".into()],
                Case::Camel
            )
        );
        assert_eq!(
            split_identifier("parseJSONData").words,
            vec!["parse", "json", "data"]
        );
        assert_eq!(split_identifier("MAX_RETRIES").case, Case::Screaming);
        assert_eq!(split_identifier("HTTPServer").words, vec!["http", "server"]);
        let p = split_identifier("_private_thing__");
        assert_eq!((p.prefix.as_str(), p.suffix.as_str()), ("_", "__"));
        assert_eq!(split_identifier("user2").words, vec!["user2"]);
    }
}
