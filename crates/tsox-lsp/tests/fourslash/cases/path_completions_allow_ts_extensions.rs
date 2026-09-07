use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn path_completions_allow_ts_extensions() {
    let content = r#"// @moduleResolution: bundler
// @allowImportingTsExtensions: true
// @noEmit: true
// @Filename: /project/foo.ts
export const foo = 0;
// @Filename: /project/main.ts
import {} from ".//**/""#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
    fourslash::insert(&mut s, "foo.ts\"\nimport {} from \"./");
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
