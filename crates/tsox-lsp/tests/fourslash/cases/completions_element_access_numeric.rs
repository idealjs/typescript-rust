use tsox_lsp::fourslash::{self, Session};


#[ignore = "go: t.Skip('Known failing fourslash test')"]
#[test]
fn completions_element_access_numeric() {
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
    let mut s = Session::new_for_test("completionsElementAccessNumeric", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
