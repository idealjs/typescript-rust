use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn export_default_function() {
    let content = r#"export default function func() {
    /*1*/
}
 /*2*/"#;
    let mut s = Session::new_for_test("exportDefaultFunction", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
