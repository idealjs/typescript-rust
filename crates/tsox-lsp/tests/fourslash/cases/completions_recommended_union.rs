use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_recommended_union() {
    let content = r#"// @strictNullChecks: true
const enum E { A = "A", B = "B" }
const enum E2 { X = "X", Y = "Y" }
const e: E | undefined = /*a*/
const e2: E | E2 = /*b*/"#;
    let mut s = Session::new_for_test("completionsRecommended_union", content);
    fourslash::go_to_marker(&mut s, "a");
    // TODO: f.VerifyCompletions(t, "a", &fourslash.CompletionsExpectedList{
    fourslash::go_to_marker(&mut s, "b");
    // TODO: f.VerifyCompletions(t, "b", &fourslash.CompletionsExpectedList{
}
