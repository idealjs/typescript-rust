use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_type_completions7() {
    let content = r#"// @target: es2020
// @module: esnext
// @Filename: /foo.d.ts
declare namespace Foo {}
export = Foo;
// @Filename: /test.ts
[|import F/**/|]"#;
    let mut s = Session::new_for_test("importTypeCompletions7", content);
    fourslash::go_to_file(&mut s, "/test.ts");
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
