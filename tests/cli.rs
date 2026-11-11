//! The binary: `--json` on every subcommand, exit codes, glossary flow.

use std::path::PathBuf;
use std::process::{Command, Output};

fn nbma(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_nbma"))
        .args(args)
        .output()
        .unwrap()
}

fn json(args: &[&str]) -> serde_json::Value {
    let mut all = args.to_vec();
    all.push("--json");
    let out = nbma(&all);
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{args:?}: not JSON ({e}): {}",
            String::from_utf8_lossy(&out.stdout)
        )
    })
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("nbma-test-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn every_subcommand_speaks_json() {
    assert_eq!(json(&["root", "skrive"])["root"], "S-K-R-V");
    assert_eq!(
        json(&["inflect", "hus", "--form", "def"])["value"],
        "ad-det-huset"
    );
    assert_eq!(
        json(&["inflect", "skrive"])["forms"]["present"],
        "yusakrivur"
    );
    assert_eq!(
        json(&["ident", "get_user_name", "--role", "fn"])["output"],
        "uhnut_namn_ad_den_brukaren"
    );
    assert_eq!(
        json(&["prose", "The user writes a file."])["output"],
        "Fa yusakrivur ad-den-brukaren fil."
    );
    assert_eq!(json(&["fold", "ʿalā"])["output"], "yaa_3alaa");
    assert_eq!(json(&["number", "22"])["numeral"], "to wa tjue");
    assert_eq!(
        json(&["lexicon", "check"])["errors"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let dir = temp_dir("json");
    assert_eq!(
        json(&["glossary", "check", dir.to_str().unwrap()])["ok"],
        true
    );
    std::fs::write(dir.join("ordbok.nbma.toml"), "[identifiers]\n").unwrap();
    assert!(json(&["untranslate", dir.to_str().unwrap(), "--dry-run"])["files_changed"].is_array());
}

#[test]
fn unknown_words_exit_3_with_a_json_error() {
    let out = nbma(&["ident", "frobnicate_widget", "--role", "fn", "--json"]);
    assert_eq!(out.status.code(), Some(3));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["error"], "unknown");
    assert_eq!(v["words"], serde_json::json!(["frobnicate", "widget"]));
}

#[test]
fn proposed_words_are_inflected_by_the_cli() {
    let dir = temp_dir("propose");
    let g = dir.join("ordbok.nbma.toml");
    let gs = g.to_str().unwrap();
    let out = nbma(&[
        "propose",
        "widget",
        "--pos",
        "noun",
        "--bm",
        "dings",
        "--nn",
        "dings",
        "--gender",
        "m",
        "--glossary",
        gs,
    ]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v = json(&[
        "ident",
        "get_widget",
        "--role",
        "fn",
        "--glossary",
        gs,
        "--record",
    ]);
    assert_eq!(v["output"], "uhnut_ad_den_dingsen");
    let src = std::fs::read_to_string(&g).unwrap();
    assert!(src.contains("[proposed.widget]"), "{src}");
    assert!(
        src.contains("get_widget = { to = \"uhnut_ad_den_dingsen\", role = \"fn\" }"),
        "{src}"
    );
    // Reruns read the glossary first.
    assert_eq!(
        json(&["ident", "get_widget", "--role", "fn", "--glossary", gs])["from_glossary"],
        true
    );
}

#[test]
fn proposals_that_collide_are_rejected() {
    let dir = temp_dir("collide");
    let g = dir.join("ordbok.nbma.toml");
    let out = nbma(&[
        "propose",
        "shine",
        "--pos",
        "verb",
        "--bm",
        "lyse",
        "--nn",
        "lyse",
        "--glossary",
        g.to_str().unwrap(),
    ]);
    assert!(!out.status.success());
}

#[test]
fn arabic_identifiers_need_consent() {
    let refused = nbma(&["fold", "ad-den-brukaren", "--script", "arabic", "--ident"]);
    assert_eq!(refused.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&refused.stderr).contains("CVE-2021-42574"));
    let allowed = nbma(&[
        "fold",
        "ad-den-brukaren",
        "--script",
        "arabic",
        "--ident",
        "--i-understand-bidi",
    ]);
    assert!(allowed.status.success());
    assert!(String::from_utf8_lossy(&allowed.stderr).contains("Trojan Source"));
}

#[test]
fn glossary_check_catches_names_already_in_the_repo() {
    let dir = temp_dir("check");
    std::fs::write(dir.join("app.py"), "brukar = 1\n").unwrap();
    std::fs::write(
        dir.join("ordbok.nbma.toml"),
        "[identifiers]\nuser = { to = \"brukar\", role = \"var\" }\n",
    )
    .unwrap();
    let out = nbma(&["glossary", "check", dir.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(1));
}
