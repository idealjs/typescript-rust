use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifySemanticTokens"]
#[test]
fn syntactic_classifications_conflict_diff3_markers1() {
    let content = r#"class C {
<<<<<<< HEAD
    v = 1;
||||||| merged common ancestors
    v = 3;
=======
    v = 2;
>>>>>>> Branch - a
}"#;
    let mut s = Session::new_for_test("syntacticClassificationsConflictDiff3Markers1", content);
    fourslash::unsupported("VerifySemanticTokens"); // f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
