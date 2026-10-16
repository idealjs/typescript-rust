use tsox_lsp::fourslash::{self, Session};


#[test]
fn const_enum_quick_info_and_completion_list() {
    let content = r#"const enum /*1*/e {
    a,
    b,
    c
}
/*2*/e.a;"#;
    let mut s = Session::new_for_test("constEnumQuickInfoAndCompletionList", content);
    fourslash::go_to_marker(&mut s, "2");
    // TODO: f.VerifyCompletions(t, "2", &fourslash.CompletionsExpectedList{
    fourslash::verify_quick_info_at(&mut s, "1", "const enum e", "");
    fourslash::verify_quick_info_at(&mut s, "2", "const enum e", "");
}
