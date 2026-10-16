use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn hover_over_comment() {
    let content = r#"export function f() {}
//foo
/**///moo"#;
    let mut s = Session::new_for_test("hoverOverComment", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoIs(t, "", "")
    // TODO: f.VerifyBaselineFindAllReferences(t, "")
    // TODO: f.VerifyBaselineGoToDefinition(t, false, "")
}
