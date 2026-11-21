use konekode_run::Missing;

#[test]
fn a_missing_sentence_names_its_key() {
    assert_eq!(
        Missing("run.busy".to_owned()).to_string(),
        "no text for run.busy"
    );
}
