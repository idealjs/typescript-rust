use tsox_lsp::fourslash::{self, Session};


#[test]
fn syntactic_classifications_conflict_diff3_markers2() {
    let content = r#"<<<<<<< HEAD
class C { }
||||||| merged common ancestors
class E { }
=======
class D { }
>>>>>>> Branch - a"#;
    let mut s = Session::new_for_test("syntacticClassificationsConflictDiff3Markers2", content);
    // TODO: f.VerifySemanticTokens(t, []fourslash.SemanticToken{
}
