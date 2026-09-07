use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_before_rest_arg1() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @target: esnext
// @lib: esnext
const layers = Object.assign({}, /*1*/...[]);"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "1", &fourslash.CompletionsExpectedList{
}
