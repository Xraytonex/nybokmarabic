//! Golden identifier translations (tests/golden/identifiers.toml).
//! `NBMA_BLESS=1 cargo test --test golden` rewrites the expectations.

use nbma::ident::{translate, IdentError, Options, Role};
use nbma::lexicon::Lexicon;

const PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden/identifiers.toml");

fn run(input: &str, role: &str) -> Result<String, IdentError> {
    let role = Role::parse(role).expect("valid role in golden file");
    translate(Lexicon::core(), input, role, &Options::default()).map(|t| t.output)
}

#[test]
fn golden_identifiers() {
    let src = std::fs::read_to_string(PATH).unwrap();
    let doc: toml::Table = src.parse().unwrap();
    let cases = doc["case"].as_array().unwrap();
    assert!(cases.len() >= 50, "need at least 50 golden identifiers");
    let bless = std::env::var("NBMA_BLESS").is_ok();
    let mut blessed = src.clone();
    let mut failures = Vec::new();
    for case in cases {
        let t = case.as_table().unwrap();
        let input = t["input"].as_str().unwrap();
        let role = t["role"].as_str().unwrap();
        let got = run(input, role);
        match (
            t.get("expected").and_then(|v| v.as_str()),
            t.get("error").and_then(|v| v.as_str()),
            got,
        ) {
            (Some(want), _, Ok(got)) if want == got => {}
            (Some(want), _, Ok(got)) => {
                blessed = blessed.replace(
                    &format!("input = \"{input}\"\nrole = \"{role}\"\nexpected = \"{want}\""),
                    &format!("input = \"{input}\"\nrole = \"{role}\"\nexpected = \"{got}\""),
                );
                failures.push(format!("{input} ({role}): expected {want}, got {got}"));
            }
            (None, Some("unknown"), Err(IdentError::Unknown { .. })) => {}
            (None, Some("reserved"), Err(IdentError::Reserved { .. })) => {}
            (_, _, other) => failures.push(format!("{input} ({role}): unexpected {other:?}")),
        }
    }
    if bless && !failures.is_empty() {
        std::fs::write(PATH, blessed).unwrap();
        return;
    }
    assert!(
        failures.is_empty(),
        "golden mismatches:\n{}",
        failures.join("\n")
    );
}

#[test]
fn translation_is_deterministic() {
    for _ in 0..3 {
        assert_eq!(
            run("get_user_name", "fn").unwrap(),
            "uhnut_namn_ad_den_brukaren"
        );
    }
}
