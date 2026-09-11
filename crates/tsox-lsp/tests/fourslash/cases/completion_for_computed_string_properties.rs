use tsox_lsp::fourslash::{self, Session};


#[test]
fn completion_for_computed_string_properties() {
    let content = r#"const p2 = "p2";
interface A {
    ["p1"]: string;
    [p2]: string;
}
declare const a: A;
a[|./**/|]"#;
    let mut s = Session::new_for_test("completionForComputedStringProperties", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
