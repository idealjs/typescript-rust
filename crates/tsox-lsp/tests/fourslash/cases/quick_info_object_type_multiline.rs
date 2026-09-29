use tsox_lsp::fourslash::Session;


#[test]
fn quick_info_object_type_multiline() {
    let content = r#"
type X/*1*/ = {
    a: number
    b: string
    c: C
}
type C = {}
"#;
    let _s = Session::new_for_test("quickInfoObjectTypeMultiline", content);
    // TODO: f.VerifyBaselineHoverWithVerbosity(t, map[string][]int{"1": {0, 1}})
}
