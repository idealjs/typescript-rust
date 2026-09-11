use tsox_lsp::fourslash::{self, Session};


#[test]
fn syntactic_classifications_merge_conflict_marker1() {
    let content = r#"<<<<<<< HEAD
"AAAA"
=======
"BBBB"
>>>>>>> Feature"#;
    let mut s = Session::new_for_test("syntacticClassificationsMergeConflictMarker1", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{})
}
