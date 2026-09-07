use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_recommended_union() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @strictNullChecks: true
const enum E { A = "A", B = "B" }
const enum E2 { X = "X", Y = "Y" }
const e: E | undefined = /*a*/
const e2: E | E2 = /*b*/"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
}
