use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyCompletions"]
#[test]
fn completions_keywords_extends() {
    let content = r#"class C/*a*/ /*b*/ { }
class C e/*c*/ {}"#;
    let mut s = Session::new_for_test("completionsKeywordsExtends", content);
    fourslash::verify_completions_empty_at(&mut s, Some("a"));
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, []string{"b", "c"}, &fourslash.CompletionsExpectedList{
}
