use tsox_lsp::fourslash::{self, Session};


#[test]
fn syntactic_classifications_conflict_markers2() {
    let content = r#"<<<<<<< HEAD
class C { }
=======
class D { }
>>>>>>> Branch - a"#;
    let mut s = Session::new_for_test("syntacticClassificationsConflictMarkers2", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
