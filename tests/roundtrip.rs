//! The example project: `nbma untranslate examples/after` must restore
//! `examples/before` byte for byte, and the glossary must still pass its
//! pre-flight check against the original.

use std::path::{Path, PathBuf};
use std::process::Command;

use nbma::glossary::{walk, Glossary, FILE_NAME};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples")
}

fn copy_dir(from: &Path, to: &Path) {
    for file in walk(from).unwrap() {
        let target = to.join(file.strip_prefix(from).unwrap());
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::copy(&file, &target).unwrap();
    }
    std::fs::copy(from.join(FILE_NAME), to.join(FILE_NAME)).unwrap();
}

#[test]
fn untranslate_restores_the_original_byte_for_byte() {
    let work = std::env::temp_dir().join(format!("nbma-roundtrip-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&work);
    copy_dir(&root().join("after"), &work);
    let status = Command::new(env!("CARGO_BIN_EXE_nbma"))
        .arg("untranslate")
        .arg(&work)
        .status()
        .unwrap();
    assert!(status.success());
    let before = root().join("before");
    let originals = walk(&before).unwrap();
    assert!(!originals.is_empty());
    for file in &originals {
        let rel = file.strip_prefix(&before).unwrap();
        let restored = std::fs::read(work.join(rel)).unwrap();
        assert!(
            std::fs::read(file).unwrap() == restored,
            "{} differs",
            rel.display()
        );
    }
    assert_eq!(
        walk(&work).unwrap().len(),
        originals.len(),
        "extra files after untranslate"
    );
}

#[test]
fn example_glossary_passes_the_preflight_check() {
    let glossary = Glossary::load(&root().join("after").join(FILE_NAME)).unwrap();
    assert!(glossary.identifiers.len() > 20);
    assert_eq!(
        glossary.check(&root().join("before")).unwrap(),
        Vec::<String>::new()
    );
}

#[test]
fn translated_names_actually_changed() {
    let after = std::fs::read_to_string(root().join("after/registry.py")).unwrap();
    assert!(after.contains("def uhnut_ad_den_brukaren(self, namn):"));
    assert!(after.contains("feltaan = tuple("), "pairs must be dual");
    assert!(!after.contains("def get_user"));
}
