use tsox_lsp::fourslash::{self, Session};


#[test]
fn import_type_completions5() {
    let content = r#"// @allowSyntheticDefaultImports: false
// @esModuleInterop: false
// @module: commonjs
// @Filename: /foo.ts
interface Foo { };
export = Foo;
// @Filename: /bar.ts
 [|import type f/**/|]"#;
    let mut s = Session::new_for_test("importTypeCompletions5", content);
    fourslash::go_to_file(&mut s, "/bar.ts");
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
