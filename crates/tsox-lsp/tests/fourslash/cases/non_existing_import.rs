use tsox_lsp::fourslash::{self, Session};


#[test]
fn non_existing_import() {
    let content = r#"// @lib: es5
namespace m {
    import foo = module(_foo);
    var n: num/*1*/
}"#;
    let mut s = Session::new_for_test("nonExistingImport", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
