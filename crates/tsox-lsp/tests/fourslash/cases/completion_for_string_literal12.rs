use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_string_literal12() {
    let content = r#"function foo(x: "bla"): void;
function foo(x: "bla"): void;
function foo(x: string) {}
foo("[|/**/|]")"#;
    let mut s = Session::new_for_test("completionForStringLiteral12", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
