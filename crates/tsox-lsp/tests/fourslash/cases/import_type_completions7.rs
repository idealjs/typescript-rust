use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn import_type_completions7() {
    let content = r#"// @target: es2020
// @module: esnext
// @Filename: /foo.d.ts
declare namespace Foo {}
export = Foo;
// @Filename: /test.ts
[|import F/**/|]"#;
    let mut s = Session::new(content);
    fourslash::go_to_file(&mut s, "/test.ts");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
