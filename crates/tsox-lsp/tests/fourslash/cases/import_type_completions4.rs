use tsox_lsp::fourslash::{self, Session};


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
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
