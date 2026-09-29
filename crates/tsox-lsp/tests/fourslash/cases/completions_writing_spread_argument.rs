use tsox_lsp::fourslash::{self, Session};


#[test]
fn completions_writing_spread_argument() {
    let content = r#"// @lib: es5

const [] = [Math.min(./*marker*/)]
"#;
    let mut s = Session::new_for_test("completionsWritingSpreadArgument", content);
    fourslash::go_to_marker(&mut s, "marker");
    fourslash::verify_completions_empty_at(&mut s, None);
    fourslash::insert(&mut s, ".");
    fourslash::verify_completions_empty_at(&mut s, None);
    fourslash::insert(&mut s, ".");
    // TODO: f.VerifyCompletions(t, nil, &fourslash.CompletionsExpectedList{
}
