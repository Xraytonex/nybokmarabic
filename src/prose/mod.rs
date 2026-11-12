//! Template-based translation of simple English sentences (SPEC §9–11, §17).
//!
//! Supported shapes, each optionally negated:
//!   SUBJECT VERB [OBJECT] [PREP NP]* [ADVERB]*      present or past
//!   SUBJECT is/are/was/were PREDICATE                verbless sentence
//!   SUBJECT was/were PARTICIPLE [by NP]              passive
//!   SUBJECT has/have OBJECT                          possession with ʿinda
//!   SUBJECT has [been] PARTICIPLE / is VERB-ing      perfect, progressive
//!   SUBJECT can/should/must/will/may VERB …          modals
//!   there is NP                                      existence with hunāka
//!   Does/Did SUBJECT VERB …? / Is SUBJECT PREDICATE? yes/no questions
//!   VERB [OBJECT] …  / Do not VERB …                 commands
//!   VERBs [OBJECT] … / Does not VERB …               docstrings
//! An if/when/while/because clause, a prepositional phrase or a participle
//! may come before (with a comma) or after the main clause; before/after/
//! until take a clause too. A noun may take a relative clause, a
//! participle, a possessive `'s` or a list; `-ing` and `to` make verbal
//! nouns. Anything else is rejected rather than guessed.

mod clause;
mod noun;
mod parser;
mod predicate;
mod verbal;

use serde::Serialize;

use crate::lexicon::Lexicon;
use crate::segment::read;
use parser::Parser;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum ProseError {
    Unknown { words: Vec<String> },
    Unsupported { sentence: String, reason: String },
}

impl std::fmt::Display for ProseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProseError::Unknown { words } => write!(f, "not in the lexicon: {}", words.join(", ")),
            ProseError::Unsupported { sentence, reason } => {
                write!(
                    f,
                    "cannot translate \"{sentence}\": {reason}. Simplify the sentence."
                )
            }
        }
    }
}

fn expand_contractions(sentence: &str) -> String {
    let mut s = format!(" {} ", sentence.to_lowercase());
    for (from, to) in [
        ("doesn't", "does not"),
        ("don't", "do not"),
        ("didn't", "did not"),
        ("isn't", "is not"),
        ("aren't", "are not"),
        ("wasn't", "was not"),
        ("weren't", "were not"),
        ("hasn't", "has not"),
        ("haven't", "have not"),
        ("can't", "can not"),
        ("it's", "it is"),
        ("there's", "there is"),
        ("cannot", "can not"),
        ("we're", "we are"),
        ("they're", "they are"),
    ] {
        s = s.replace(from, to);
    }
    s
}

/// Words, lower-cased, with contractions expanded and each comma as a word
/// of its own (it marks where a fronted clause ends).
fn tokens(sentence: &str) -> Vec<String> {
    let mut out = Vec::new();
    // Inside 'single quotes', a final apostrophe closes the quote.
    let mut quoted = false;
    for raw in expand_contractions(sentence)
        .replace(',', ", ")
        .split_whitespace()
    {
        let word = raw.trim_matches(|c: char| !c.is_alphanumeric() && c != '-');
        let core = raw.trim_end_matches([',', '.', '?', '!', ')', '"', ':', ';']);
        let opens = raw.trim_start_matches(['(', '"']).starts_with('\'');
        let closes = core.ends_with('\'') && (quoted || opens);
        quoted = (quoted || opens) && !closes;
        // `the user's name`, `the users' names`: the possessive is a word
        // of its own (§18.8).
        let owner = word.strip_suffix("'s").or_else(|| {
            (core.ends_with("s'") && !closes)
                .then_some(word)
                .filter(|w| w.ends_with('s'))
        });
        match owner {
            Some(owner) if !owner.is_empty() => {
                out.push(owner.to_string());
                out.push("'s".to_string());
            }
            _ if !word.is_empty() => out.push(word.to_string()),
            _ => {}
        }
        if raw.trim_end_matches(['"', '\'', ')']).ends_with(',') {
            out.push(",".to_string());
        }
    }
    out
}

fn sentence(lex: &Lexicon, raw: &str) -> Result<String, ProseError> {
    let trimmed = raw.trim();
    let terminal = trimmed
        .chars()
        .last()
        .filter(|c| matches!(c, '.' | '?' | '!'));
    // Abbreviations and compounds read as their words, as in identifiers.
    let words: Vec<String> = read(lex, &tokens(trimmed))
        .into_iter()
        .map(|r| r.word)
        .collect();
    let (out, question) = Parser::new(lex, words, trimmed).sentence()?;
    let end = if question { Some('?') } else { terminal };
    // Norwegian and ALA-LC romanization agree: a sentence starts with a
    // capital letter. ALA-LC capitalizes the letter after an initial ʿayn
    // or hamza (`ʿIndamā`).
    let mut s = String::new();
    let mut done = false;
    for c in out.join(" ").chars() {
        if !done && (c.is_lowercase() || c.is_uppercase()) {
            s.extend(c.to_uppercase());
            done = true;
        } else {
            s.push(c);
        }
    }
    if let Some(c) = end {
        s.push(c);
    }
    Ok(s)
}

/// Translate one or more simple English sentences.
pub fn translate(lex: &Lexicon, text: &str) -> Result<String, ProseError> {
    let mut sentences = Vec::new();
    let mut current = String::new();
    for c in text.chars() {
        current.push(c);
        if matches!(c, '.' | '?' | '!') {
            sentences.push(std::mem::take(&mut current));
        }
    }
    if !current.trim().is_empty() {
        sentences.push(current);
    }
    let mut out = Vec::new();
    for s in sentences.iter().filter(|s| !s.trim().is_empty()) {
        out.push(sentence(lex, s)?);
    }
    if out.is_empty() {
        return Err(ProseError::Unsupported {
            sentence: text.to_string(),
            reason: "there is nothing to translate".to_string(),
        });
    }
    Ok(out.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(s: &str) -> String {
        translate(Lexicon::core(), s).unwrap()
    }

    #[test]
    fn verb_first_after_fa() {
        assert_eq!(
            t("The user writes a file."),
            "Fa yusakrivur ad-den-brukaren fil."
        );
    }

    #[test]
    fn negation_is_a_circumfix() {
        assert_eq!(
            t("The user does not write a file."),
            "Fa mā yusakrivur ikkje ad-den-brukaren fil."
        );
    }

    #[test]
    fn neuter_subjects_take_feminine_agreement() {
        assert_eq!(t("The house reads."), "Fa talsuwur ad-det-huset.");
    }

    #[test]
    fn past_tense() {
        assert_eq!(
            t("The user wrote a file."),
            "Fa sakrava ad-den-brukaren fil."
        );
    }

    #[test]
    fn questions_start_with_hal() {
        assert_eq!(
            t("Does the user write a file?"),
            "Hal yusakrivur ad-den-brukaren fil?"
        );
        assert_eq!(t("Is the file empty?"), "Hal ad-den-fila tom?");
    }

    #[test]
    fn verbless_sentences_have_no_copula() {
        assert_eq!(t("The user is valid."), "Fa ad-den-brukaren gyldig.");
        assert_eq!(
            t("The user is not valid."),
            "Fa mā ikkje ad-den-brukaren gyldig."
        );
        assert_eq!(t("The house is big."), "Fa ad-det-huset stort.");
        assert_eq!(t("The users are valid."), "Fa ad-dei-brukarane gyldige.");
    }

    #[test]
    fn counting_follows_arabic() {
        assert_eq!(
            t("The user writes 2 files."),
            "Fa yusakrivur ad-den-brukaren filān."
        );
        assert_eq!(
            t("The user writes 22 files."),
            "Fa yusakrivur ad-den-brukaren to wa tjue fil."
        );
        assert_eq!(
            t("The user writes 3 files."),
            "Fa yusakrivur ad-den-brukaren tre filer."
        );
    }

    #[test]
    fn pronouns_keep_both_languages_happy() {
        assert_eq!(t("We read the file."), "Fa nalsuwur vi ad-den-fila.");
    }

    #[test]
    fn object_pronouns_are_marked_twice() {
        assert_eq!(
            t("The user reads me."),
            "Fa yalsuwur-nī ad-den-brukaren meg."
        );
        assert_eq!(t("We help them."), "Fa nuhajlipur-hum vi dei.");
    }

    #[test]
    fn prepositions() {
        assert_eq!(
            t("The user writes the name to the file."),
            "Fa yusakrivur ad-den-brukaren ad-det-namnet ilā ad-den-fila."
        );
    }

    #[test]
    fn commands_take_the_imperative() {
        assert_eq!(t("Return the user."), "Fa ratrin ad-den-brukaren.");
        assert_eq!(t("Read the file."), "Fa ulsuw ad-den-fila.");
        // A verb that is also a noun is still a command when it comes first.
        assert_eq!(
            t("Set the debug output level."),
            "Fa ustut nivå utdata ad-den-valsen."
        );
        // A plural noun first is a subject.
        assert_eq!(
            t("Logs record events."),
            "Fa talgugur hendingar ad-den-posten."
        );
        assert_eq!(
            t("Read and return a line from the stream."),
            "Fa ulsuw wa ratrin linje min ad-den-strømmen."
        );
    }

    #[test]
    fn prohibitions_use_the_circumfix_on_the_second_person() {
        assert_eq!(
            t("Do not read the file."),
            "Fa mā talsuwur ikkje ad-den-fila."
        );
        assert_eq!(
            t("Do not read or write the file."),
            "Fa mā talsuwur ikkje wa mā tusakrivur ikkje ad-den-fila."
        );
    }

    #[test]
    fn docstrings_drop_their_subject() {
        assert_eq!(t("Returns the user."), "Fa turatrinur ad-den-brukaren.");
        assert_eq!(
            t("Does not return the user."),
            "Fa mā turatrinur ikkje ad-den-brukaren."
        );
        assert_eq!(
            t("Reads and returns a line."),
            "Fa talsuwur wa turatrinur linje."
        );
        assert_eq!(
            t("Raised when the file is closed."),
            "Fa tuksatus ʿindamā ad-den-fila malkūk."
        );
    }

    #[test]
    fn conditions_put_the_subject_first_and_fa_after() {
        assert_eq!(
            t("If the file is empty, return the user."),
            "Idhā ad-den-fila tom, fa ratrin ad-den-brukaren."
        );
        // `idhā` takes the Arabic perfect whatever the English tense.
        assert_eq!(
            t("If the user writes a file, the file is valid."),
            "Idhā ad-den-brukaren sakrava fil, fa ad-den-fila gyldig."
        );
        assert_eq!(
            t("Return the user if the name is valid."),
            "Fa ratrin ad-den-brukaren idhā ad-det-namnet gyldig."
        );
        assert_eq!(
            t("If the user is not valid, do not read the file."),
            "Idhā ad-den-brukaren mā ikkje gyldig, fa mā talsuwur ikkje ad-den-fila."
        );
        // Subject first: the verb agrees in number too.
        assert_eq!(
            t("When the users write files, we read the files."),
            "ʿIndamā ad-dei-brukarane yusakrivūnar filer, fa nalsuwur vi ad-de-filene."
        );
        assert_eq!(
            t("Return the user unless the name is valid."),
            "Fa ratrin ad-den-brukaren illā idhā ad-det-namnet gyldig."
        );
    }

    #[test]
    fn passives_are_arabic_verbs_and_states_stay_participles() {
        assert_eq!(t("The file was deleted."), "Fa sultitat ad-den-fila.");
        assert_eq!(
            t("The file is read by the user."),
            "Fa tulsawust ad-den-fila min qibali ad-den-brukaren."
        );
        assert_eq!(
            t("The file was not deleted."),
            "Fa mā sultitat ikkje ad-den-fila."
        );
        assert_eq!(t("The file is closed."), "Fa ad-den-fila malkūk.");
    }

    #[test]
    fn relative_clauses() {
        assert_eq!(
            t("The user that writes the file is valid."),
            "Fa ad-den-brukaren alladhī yusakrivur ad-den-fila gyldig."
        );
        // The object comes back as a suffix on the verb.
        assert_eq!(
            t("Read the file that the user writes."),
            "Fa ulsuw ad-den-fila allatī ad-den-brukaren yusakrivur-hā."
        );
        assert_eq!(
            t("Read the files that the users wrote."),
            "Fa ulsuw ad-de-filene allātī ad-dei-brukarane sakravū-hunna."
        );
        // No Arabic relative pronoun after an indefinite noun: Norwegian
        // `som` fills the gap.
        assert_eq!(
            t("A user that reads files writes names."),
            "Fa yusakrivur brukar som yalsuwur filer namn."
        );
    }

    #[test]
    fn demonstratives_precede_the_definite_noun() {
        assert_eq!(
            t("These files are valid."),
            "Fa hāʾulāʾi ad-de-filene gyldige."
        );
        assert_eq!(
            t("Return the fields of this class."),
            "Fa ratrin felt hādhihi ad-den-klassa."
        );
        assert_eq!(t("This is the default."), "Fa hādhā as-standard.");
    }

    #[test]
    fn possessors_chain_and_keep_their_words() {
        assert_eq!(
            t("Return the name of the user of the file."),
            "Fa ratrin namn brukar ad-den-fila."
        );
        assert_eq!(
            t("Return the name of the big user."),
            "Fa ratrin namn ad-den-brukaren as-store."
        );
        assert_eq!(
            t("Return the names of these users."),
            "Fa ratrin namn hāʾulāʾi ad-dei-brukarane."
        );
        assert_eq!(
            t("Return the name of all users."),
            "Fa ratrin namn kull ad-dei-brukarane."
        );
    }

    #[test]
    fn subjects_take_prepositions_and_adverbs_come_before_the_verb() {
        assert_eq!(
            t("A committee with three members always has a majority."),
            "Fa ʿinda nemnd maʿa tre medlemar fleirtal alltid."
        );
        assert_eq!(
            t("The language jokes about grammar and about politics."),
            "Fa taspukur ad-det-språket ʿan grammatikk wa ʿan politikk."
        );
    }

    #[test]
    fn headings_stay_noun_phrases() {
        assert_eq!(
            t("Exception for empty headers."),
            "Unntak min ajli hovud tomme."
        );
        assert_eq!(
            t("True if the connection is closed."),
            "Sann idhā ad-den-tilkoplinga malkūk."
        );
    }

    #[test]
    fn participles_after_a_noun_are_arabic_adjectives() {
        assert_eq!(
            t("Return the object wrapped by the function."),
            "Fa ratrin ad-det-objektet al-mapkūke min qibali ad-den-funksjonen."
        );
        assert_eq!(
            t("Update the CRC using the given data."),
            "Fa papdit ad-det-CRC-et al-bārike ad-den-dataen al-magjūwe."
        );
        assert_eq!(
            t("The file deleted by the user is empty."),
            "Fa ad-den-fila al-musaltate min qibali ad-den-brukaren tom."
        );
        // No verb after the participle: it is the passive of the question.
        assert_eq!(
            t("Was the file deleted by the user?"),
            "Hal sultitat ad-den-fila min qibali ad-den-brukaren?"
        );
    }

    #[test]
    fn gerunds_and_infinitives_are_verbal_nouns() {
        assert_eq!(
            t("Check the number by reading the file."),
            "Fa ushkuk ad-det-talet bi-lasw ad-den-fila."
        );
        assert_eq!(
            t("Read the file to return the user."),
            "Fa ulsuw ad-den-fila li-ratrana ad-den-brukaren."
        );
        assert_eq!(
            t("Return the class to be used."),
            "Fa ratrin ad-den-klassa li-bark."
        );
        assert_eq!(
            t("Open the file for reading or writing."),
            "Fa upnuw ad-den-fila min ajli lasw aw sakrava."
        );
    }

    #[test]
    fn possessive_s_is_an_idafa() {
        assert_eq!(
            t("Return the user's name."),
            "Fa ratrin namn ad-den-brukaren."
        );
        assert_eq!(
            t("Return the users' names."),
            "Fa ratrin namn ad-dei-brukarane."
        );
        assert_eq!(
            t("Return this user's name."),
            "Fa ratrin namn hādhā ad-den-brukaren."
        );
        assert_eq!(
            t("Return the name of the user's file."),
            "Fa ratrin namn fil ad-den-brukaren."
        );
        // A closing quote is not a plural possessive.
        assert_eq!(
            t("Return 'the users' with the name."),
            "Fa ratrin ad-dei-brukarane maʿa ad-det-namnet."
        );
    }

    #[test]
    fn modals_take_an_and_the_subjunctive() {
        assert_eq!(
            t("The user can read the file."),
            "Fa yaknunur ad-den-brukaren an yalsuwa ad-den-fila."
        );
        assert_eq!(
            t("The users can not read the files."),
            "Fa mā yaknunur ikkje ad-dei-brukarane an yalsuwū ad-de-filene."
        );
        assert_eq!(
            t("The file should be empty."),
            "Fa tusaklilur ad-den-fila an tavruwa tom."
        );
        // In a relative clause the suffix goes on the verb after `an`.
        assert_eq!(
            t("The file that the user can read is empty."),
            "Fa ad-den-fila allatī ad-den-brukaren yaknunur an yalsuwa-hā tom."
        );
    }

    #[test]
    fn will_and_may_are_arabic_particles() {
        assert_eq!(
            t("The file will be deleted."),
            "Fa sa-tusaltatust ad-den-fila."
        );
        assert_eq!(
            t("The user will not write the file."),
            "Fa lan yusakriva ikkje ad-den-brukaren ad-den-fila."
        );
        assert_eq!(
            t("The file may be empty."),
            "Fa qad tavruwu ad-den-fila tom."
        );
    }

    #[test]
    fn perfect_and_progressive() {
        assert_eq!(
            t("The user has written the file."),
            "Fa qad sakrava ad-den-brukaren ad-den-fila."
        );
        assert_eq!(
            t("The file has not been deleted."),
            "Fa mā sultitat ikkje ad-den-fila."
        );
        assert_eq!(
            t("The user is writing a file."),
            "Fa yusakrivur ad-den-brukaren fil."
        );
    }

    #[test]
    fn more_subordinate_clauses() {
        assert_eq!(
            t("Return the user because the file is empty."),
            "Fa ratrin ad-den-brukaren li-anna ad-den-fila tom."
        );
        assert_eq!(
            t("Read the file before the user writes it."),
            "Fa ulsuw ad-den-fila qabla an ad-den-brukaren yusakrivar-hā det."
        );
        assert_eq!(
            t("After the user wrote the file, return the user."),
            "Baʿda an ad-den-brukaren sakrava ad-den-fila, fa ratrin ad-den-brukaren."
        );
        assert_eq!(
            t("Wait until the file is empty."),
            "Fa uvnut ḥattā ad-den-fila tom."
        );
        // `before` with a noun is still a preposition.
        assert_eq!(
            t("The tests pass before the translation."),
            "Fa yabsutur ad-dei-testane qabla ad-den-omsetjinga."
        );
    }

    #[test]
    fn that_and_whether_clauses() {
        assert_eq!(
            t("Check that the file is empty."),
            "Fa ushkuk anna ad-den-fila tom."
        );
        assert_eq!(
            t("Check whether the file is empty."),
            "Fa ushkuk hal ad-den-fila tom."
        );
    }

    #[test]
    fn there_is_and_no() {
        assert_eq!(t("There is a file."), "Fa hunāka fil.");
        assert_eq!(t("There is no file."), "Fa mā ikkje hunāka ayy fil.");
        assert_eq!(
            t("No user writes the file."),
            "Fa mā yusakrivur ikkje ayy brukar ad-den-fila."
        );
        assert_eq!(
            t("Read the file without a name."),
            "Fa ulsuw ad-den-fila dūna namn."
        );
    }

    #[test]
    fn lists_commas_and_prepositional_predicates() {
        assert_eq!(
            t("Read the file, the name and the user."),
            "Fa ulsuw ad-den-fila wa ad-det-namnet wa ad-den-brukaren."
        );
        assert_eq!(
            t("The user and the file are valid."),
            "Fa ad-den-brukaren wa ad-den-fila gyldige."
        );
        assert_eq!(
            t("Read the file, returning the user."),
            "Fa ulsuw ad-den-fila, muratrin ad-den-brukaren."
        );
        assert_eq!(
            t("In the stream, the file is empty."),
            "Fī ad-den-strømmen, fa ad-den-fila tom."
        );
        assert_eq!(
            t("The file is in the stream."),
            "Fa ad-den-fila fī ad-den-strømmen."
        );
        assert_eq!(t("Set up the file."), "Fa ustut opp ad-den-fila.");
    }

    #[test]
    fn possessive_pronouns_are_arabic_suffixes() {
        assert_eq!(t("Return its name."), "Fa ratrin namnet-hā.");
        assert_eq!(t("Return the name of its file."), "Fa ratrin namn fila-hā.");
        assert_eq!(
            t("Return their full names."),
            "Fa ratrin namna-hum al-fulle."
        );
    }

    #[test]
    fn adverbs_are_neuter_adjectives() {
        assert_eq!(
            t("Read the file recursively."),
            "Fa ulsuw ad-den-fila rekursivt."
        );
        assert_eq!(t("Copy the file safely."), "Fa ukpur ad-den-fila trygt.");
    }

    #[test]
    fn so_and_then_are_fa() {
        assert_eq!(
            t("The file is empty, so return the user."),
            "Fa ad-den-fila tom, fa ratrin ad-den-brukaren."
        );
        assert_eq!(
            t("If the file is empty, then return the user."),
            "Idhā ad-den-fila tom, fa ratrin ad-den-brukaren."
        );
    }

    #[test]
    fn unsupported_shapes_are_refused_not_guessed() {
        for s in [
            "The user writes and reads a file.",
            "If the file is empty return the user.",
            "The file that is empty is closed.",
            "The user reads the file, the name.",
            "Read the user's file with no name.",
            "The user may not write the file.",
            "We write so we read.",
            "The user reads the file, the name and no user.",
            "The file was deleted by no user.",
            "The file that the user reads the name is empty.",
            "The user does not read no file.",
            "No user does not read the file.",
            "The user is to read the file.",
            "Write the name in the.",
            "Return the name of that.",
            "Return the name of 3 users.",
            "Return the file, that the user writes.",
            "Does not return the user and does not read the file.",
            "",
            " . ",
        ] {
            assert!(
                matches!(
                    translate(Lexicon::core(), s),
                    Err(ProseError::Unsupported { .. })
                ),
                "{s}"
            );
        }
    }

    #[test]
    fn unknown_words_fail_loudly() {
        assert!(matches!(
            translate(Lexicon::core(), "The wombat writes."),
            Err(ProseError::Unknown { .. })
        ));
    }
}
