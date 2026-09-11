use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn export_default_class() {
    let content = r#"export default class C {
    method() { /*1*/ }
}
 /*2*/"#;
    let mut s = Session::new_for_test("exportDefaultClass", content);
    // TODO: f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
