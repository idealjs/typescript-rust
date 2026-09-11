use tsox_lsp::fourslash::{self, Session};


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
    let mut s = Session::new_for_test("completionsElementAccessNumeric", content);
    // TODO: f.VerifyCompletions(t, "", &fourslash.CompletionsExpectedList{
}
