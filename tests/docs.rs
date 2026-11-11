//! Every ```console block in the docs is executed against the real binary,
//! so no document can disagree with the CLI. A block looks like:
//!
//!     $ nbma inflect hus --form def
//!     ad-det-huset
//!
//! `NBMA_BLESS=1 cargo test --test docs` rewrites the outputs in place.

use std::path::Path;
use std::process::Command;

const DOCS: &[&str] = &[
    "SPEC.md",
    "README.md",
    "AGENTS.md",
    ".claude/skills/nybokmarabic/SKILL.md",
    "CORRECTIONS.md",
    "COLLISIONS.md",
];

/// Minimal shell-style splitting: whitespace, single and double quotes.
fn shell_words(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut has = false;
    for c in line.chars() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), c) => cur.push(c),
            (None, '"' | '\'') => {
                quote = Some(c);
                has = true;
            }
            (None, c) if c.is_whitespace() => {
                if has || !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                    has = false;
                }
            }
            (None, c) => cur.push(c),
        }
    }
    if has || !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn run(cmd: &str) -> String {
    let words = shell_words(cmd);
    assert_eq!(
        words.first().map(String::as_str),
        Some("nbma"),
        "doc command must start with nbma: {cmd}"
    );
    let out = Command::new(env!("CARGO_BIN_EXE_nbma"))
        .args(&words[1..])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    text.trim_end().to_string()
}

/// Returns (mismatches, blessed document).
fn check_doc(src: &str) -> (Vec<String>, String) {
    let mut out_lines = Vec::new();
    let mut failures = Vec::new();
    let mut lines = src.lines().peekable();
    let mut in_console = false;
    while let Some(line) = lines.next() {
        out_lines.push(line.to_string());
        if line.trim_start().starts_with("```console") {
            in_console = true;
            continue;
        }
        if in_console && line.trim_start().starts_with("```") {
            in_console = false;
            continue;
        }
        if !in_console {
            continue;
        }
        if let Some(cmd) = line.strip_prefix("$ ") {
            let mut expected = Vec::new();
            while let Some(next) = lines.peek() {
                if next.starts_with("$ ") || next.trim_start().starts_with("```") {
                    break;
                }
                expected.push(lines.next().unwrap().to_string());
            }
            let expected = expected.join("\n").trim_end().to_string();
            let got = run(cmd);
            if got != expected {
                failures.push(format!("$ {cmd}\n--- expected\n{expected}\n--- got\n{got}"));
            }
            out_lines.extend(got.lines().map(str::to_string));
        }
    }
    let mut blessed = out_lines.join("\n");
    if src.ends_with('\n') {
        blessed.push('\n');
    }
    (failures, blessed)
}

#[test]
fn documented_examples_match_the_cli() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bless = std::env::var("NBMA_BLESS").is_ok();
    let mut all = Vec::new();
    let mut checked = 0;
    for doc in DOCS {
        let path = root.join(doc);
        let Ok(src) = std::fs::read_to_string(&path) else {
            continue;
        };
        let (failures, blessed) = check_doc(&src);
        checked += src.matches("\n$ nbma").count();
        if bless && blessed != src {
            std::fs::write(&path, blessed).unwrap();
        } else {
            all.extend(failures.into_iter().map(|f| format!("{doc}:\n{f}")));
        }
    }
    assert!(
        all.is_empty(),
        "docs disagree with the CLI:\n\n{}",
        all.join("\n\n")
    );
    assert!(
        checked > 0 || !root.join("SPEC.md").exists(),
        "no console examples found"
    );
}

#[test]
fn shell_words_handles_quotes() {
    assert_eq!(
        shell_words(r#"nbma prose "The user writes." --json"#),
        vec!["nbma", "prose", "The user writes.", "--json"]
    );
    assert_eq!(
        shell_words("nbma fold 'ʿalā ad-det-huset'"),
        vec!["nbma", "fold", "ʿalā ad-det-huset"]
    );
}
