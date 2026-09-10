use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn import_type_completions6() {
    let content = r#"// @module: esnext
// @Filename: /foo.ts
export const foo = { };
export interface Foo { };
// @Filename: /bar.ts
 [|import type * as f/**/|]"#;
    let mut s = Session::new_for_test("importTypeCompletions6", content);
    fourslash::go_to_file(&mut s, "/bar.ts");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
