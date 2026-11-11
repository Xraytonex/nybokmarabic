//! `nbma untranslate`: reverse a translation using `ordbok.nbma.toml`.
//!
//! Forward renames need semantic tooling because English names collide with
//! standard-library names. The reverse direction does not: the glossary
//! check guarantees every Nybokmarabic name is unique and was absent from the
//! original repo, so whole-word replacement of those names is exact.

use std::collections::HashMap;
use std::path::{Component, Path};

use serde::Serialize;

use crate::glossary::{identifier_words, walk, Glossary};

#[derive(Clone, Debug, Default, Serialize)]
pub struct Report {
    pub files_changed: Vec<String>,
    pub identifiers: usize,
    pub text_blocks: usize,
    pub files_renamed: Vec<String>,
    pub dry_run: bool,
}

/// Reverse one file's text in a single pass. At each position the longest
/// matching text block (comments, docstrings, docs) wins; restored text is
/// never matched again. Identifier words outside text blocks are then
/// mapped back whole.
pub fn reverse_text(
    text: &str,
    glossary: &Glossary,
    reverse: &HashMap<&str, &str>,
) -> (String, usize, usize) {
    let mut blocks: Vec<_> = glossary.text.iter().filter(|b| !b.to.is_empty()).collect();
    blocks.sort_by_key(|b| std::cmp::Reverse(b.to.len()));
    let mut out = String::with_capacity(text.len());
    let mut segment_start = 0;
    let mut i = 0;
    let (mut idents, mut block_count) = (0, 0);
    while i < text.len() {
        if let Some(b) = blocks.iter().find(|b| text[i..].starts_with(b.to.as_str())) {
            idents += reverse_words(&text[segment_start..i], reverse, &mut out);
            out.push_str(&b.from);
            block_count += 1;
            i += b.to.len();
            segment_start = i;
        } else {
            i += text[i..].chars().next().map_or(1, char::len_utf8);
        }
    }
    idents += reverse_words(&text[segment_start..], reverse, &mut out);
    (out, idents, block_count)
}

fn reverse_words(segment: &str, reverse: &HashMap<&str, &str>, out: &mut String) -> usize {
    let mut last = 0;
    let mut count = 0;
    for (start, word) in identifier_words(segment) {
        if let Some(original) = reverse.get(word) {
            out.push_str(&segment[last..start]);
            out.push_str(original);
            last = start + word.len();
            count += 1;
        }
    }
    out.push_str(&segment[last..]);
    count
}

/// A `[files]` path must be relative and stay inside the repo.
fn safe_relative(p: &str) -> Result<&Path, String> {
    let path = Path::new(p);
    let ok = !p.is_empty() && path.components().all(|c| matches!(c, Component::Normal(_)));
    if ok {
        Ok(path)
    } else {
        Err(format!("refusing unsafe path in [files]: {p}"))
    }
}

/// Write via a temporary file in the same directory, then rename, so a
/// failure never leaves a truncated source file.
fn write_atomic(path: &Path, contents: &str) -> Result<(), String> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let tmp = path.with_file_name(format!(".{name}.nbma-tmp"));
    std::fs::write(&tmp, contents).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Reverse map from translated name to original. Two originals sharing a
/// translation cannot be reversed, so that is an error.
fn reverse_map(glossary: &Glossary) -> Result<HashMap<&str, &str>, String> {
    let mut reverse = HashMap::new();
    for (from, r) in glossary
        .identifiers
        .iter()
        .filter(|(from, r)| *from != r.to)
    {
        if let Some(other) = reverse.insert(r.to.as_str(), from.as_str()) {
            return Err(format!(
                "{other} and {from} both translate to {}; cannot reverse",
                r.to
            ));
        }
    }
    Ok(reverse)
}

pub fn untranslate(root: &Path, glossary: &Glossary, dry_run: bool) -> Result<Report, String> {
    let reverse = reverse_map(glossary)?;
    let mut report = Report {
        dry_run,
        ..Report::default()
    };
    // Pre-flight every rename before touching anything.
    let mut renames = Vec::new();
    for (from, to) in &glossary.files {
        let (from_rel, to_rel) = (safe_relative(from)?, safe_relative(to)?);
        let (source, target) = (root.join(to_rel), root.join(from_rel));
        if !source.exists() {
            continue;
        }
        if target.exists() {
            return Err(format!(
                "{from} already exists; refusing to overwrite it with {to}"
            ));
        }
        renames.push((source, target, format!("{to} -> {from}")));
    }
    for (source, target, label) in renames {
        if !dry_run {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| format!("{}: {e}", parent.display()))?;
            }
            std::fs::rename(&source, &target).map_err(|e| format!("{}: {e}", source.display()))?;
        }
        report.files_renamed.push(label);
    }
    for path in walk(root)? {
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        let (reversed, idents, blocks) = reverse_text(&text, glossary, &reverse);
        if reversed != text {
            if !dry_run {
                write_atomic(&path, &reversed)?;
            }
            let rel = path.strip_prefix(root).unwrap_or(&path);
            report.files_changed.push(rel.display().to_string());
            report.identifiers += idents;
            report.text_blocks += blocks;
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverses_identifiers_and_text_but_not_substrings() {
        let g = Glossary::parse(
            r##"
[identifiers]
user = { to = "brukar", role = "var" }
get_user = { to = "uhnut_ad_den_brukaren", role = "fn" }

[[text]]
from = "# Return the user."
to = "# fa …"
"##,
        )
        .unwrap();
        let reverse = g
            .identifiers
            .iter()
            .map(|(f, r)| (r.to.as_str(), f.as_str()))
            .collect();
        let src = "# fa …\ndef uhnut_ad_den_brukaren(brukar):\n    return brukar.brukarar\n";
        let (out, idents, blocks) = reverse_text(src, &g, &reverse);
        assert_eq!(
            out,
            "# Return the user.\ndef get_user(user):\n    return user.brukarar\n"
        );
        assert_eq!((idents, blocks), (3, 1));
    }

    #[test]
    fn restored_text_is_not_matched_again() {
        let g = Glossary::parse(
            "[[text]]\nfrom = \"fa\"\nto = \"fa x\"\n[[text]]\nfrom = \"zzz\"\nto = \"fa\"\n",
        )
        .unwrap();
        let (out, _, blocks) = reverse_text("fa x", &g, &HashMap::new());
        assert_eq!((out.as_str(), blocks), ("fa", 1));
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("nbma-untr-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn renames_never_overwrite_or_escape_the_repo() {
        let dir = scratch("files");
        std::fs::write(dir.join("users.py"), "ORIGINAL").unwrap();
        std::fs::write(dir.join("brukarar.py"), "TRANSLATED").unwrap();
        let clash = Glossary::parse("[files]\n\"users.py\" = \"brukarar.py\"\n").unwrap();
        assert!(untranslate(&dir, &clash, false).is_err());
        assert_eq!(
            std::fs::read_to_string(dir.join("users.py")).unwrap(),
            "ORIGINAL"
        );
        for bad in [
            "\"../x.py\" = \"brukarar.py\"",
            "\"/tmp/x.py\" = \"brukarar.py\"",
        ] {
            let g = Glossary::parse(&format!("[files]\n{bad}\n")).unwrap();
            assert!(untranslate(&dir, &g, false).is_err(), "{bad}");
        }
        assert!(dir.join("brukarar.py").exists());
    }

    #[test]
    fn ambiguous_glossaries_are_refused() {
        let g = Glossary::parse(
            "[identifiers]\na = { to = \"x\", role = \"var\" }\nb = { to = \"x\", role = \"var\" }\n",
        )
        .unwrap();
        assert!(untranslate(&scratch("dup"), &g, true).is_err());
    }
}
