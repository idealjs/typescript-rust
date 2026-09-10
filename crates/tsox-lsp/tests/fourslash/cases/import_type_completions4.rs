use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn import_type_completions4() {
    let content = r#"// @esModuleInterop: true
// @Filename: /foo.ts
interface Foo { };
export = Foo;
// @Filename: /bar.ts
 [|import type f/**/|]"#;
    let mut s = Session::new_for_test("importTypeCompletions4", content);
    fourslash::go_to_file(&mut s, "/bar.ts");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
