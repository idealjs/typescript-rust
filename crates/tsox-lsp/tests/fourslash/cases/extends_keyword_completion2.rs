use tsox_lsp::fourslash::Session;


#[test]
fn extends_keyword_completion2() {
    let content = r#"function f1<T /*1*/>() {}
function f2<T ext/*2*/>() {}"#;
    let _s = Session::new_for_test("extendsKeywordCompletion2", content);
    // TODO: f.VerifyCompletions(t, []string{"1", "2"}, &fourslash.CompletionsExpectedList{
}
