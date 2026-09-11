use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_new_target() {
    let content = r#"class C {
    constructor() {
        if (C === new./*1*/)
    }
}
class D {
    constructor() {
        if (D === new.target./*2*/)
    }
}"#;
    let mut s = Session::new_for_test("completionsNewTarget", content);
    fourslash::verify_completions_exact_at(&mut s, Some("1"), &["target"]);
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
}
