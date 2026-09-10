use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
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
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
