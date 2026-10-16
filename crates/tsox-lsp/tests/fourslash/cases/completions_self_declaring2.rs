use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_self_declaring2() {
    let content = r#"// @lib: es5
function f1<T>(x: T) {}
f1({ [|abc|]/*1*/ });"#;
    let mut s = Session::new_for_test("completionsSelfDeclaring2", content);
    fourslash::go_to_marker(&mut s, "1");
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
