use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn syntactic_classifications_merge_conflict_marker1() {
    let content = r#"<<<<<<< HEAD
"AAAA"
=======
"BBBB"
>>>>>>> Feature"#;
    let mut s = Session::new_for_test("syntacticClassificationsMergeConflictMarker1", content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{})
}
