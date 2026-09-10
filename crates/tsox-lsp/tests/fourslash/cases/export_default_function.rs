use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn export_default_function() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"export default function func() {
    /*1*/
}
 /*2*/"#;
    let mut s = Session::new_for_test("exportDefaultFunction", content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, f.Markers(), &fourslash.CompletionsExpectedList{
}
