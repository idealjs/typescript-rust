use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_before_rest_arg1() {
    let content = r#"// @target: esnext
// @lib: esnext
const layers = Object.assign({}, /*1*/...[]);"#;
    let mut s = Session::new_for_test("completionsBeforeRestArg1", content);
    // TODO: f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
