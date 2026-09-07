use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completion_for_string_literal_import2() {
    let content = r#"// @typeRoots: my_typings
// @Filename: test.ts
/// <reference path="./[|some|]/*0*/
/// <reference types="[|some|]/*1*/
/// <reference path="./sub/[|some|]/*2*/" />
/// <reference types="[|some|]/*3*/" />
// @Filename: someFile.ts
/*someFile*/
// @Filename: sub/someOtherFile.ts
/*someOtherFile*/
// @Filename: my_typings/some-module/index.d.ts
export var x = 9;"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "0", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "3", &fourslash.CompletionsExpectedList{
}
