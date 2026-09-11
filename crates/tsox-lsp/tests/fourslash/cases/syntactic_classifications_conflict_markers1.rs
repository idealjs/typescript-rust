use tsox_lsp::fourslash::{self, Session};


#[test]
fn syntactic_classifications_conflict_markers1() {
    let content = r#"class C {
<<<<<<< HEAD
    v = 1;
=======
    v = 2;
>>>>>>> Branch - a
}"#;
    let mut s = Session::new_for_test("syntacticClassificationsConflictMarkers1", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
