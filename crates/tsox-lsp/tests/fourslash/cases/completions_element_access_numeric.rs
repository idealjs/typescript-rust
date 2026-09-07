use tsox_lsp::fourslash::{self, Session};

#[ignore = "generator: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_element_access_numeric() {
    // TODO: t.Skip("Known failing fourslash test")
    let content = r#"// @target: esnext
type Tup = [
    /**
     * The first label
     */
    lbl1: number,
    /**
     * The second label
     */
    lbl2: number
];
declare var x: Tup;
x[|./**/|]"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyCompletions"); // f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
