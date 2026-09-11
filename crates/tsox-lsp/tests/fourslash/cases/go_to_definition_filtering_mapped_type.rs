use tsox_lsp::fourslash::{self, Session};


#[test]
fn go_to_definition_filtering_mapped_type() {
    let content = r#"const obj = { /*def*/a: 1, b: 2 };
const filtered: { [P in keyof typeof obj as P extends 'b' ? never : P]: 0; } = { a: 0 };
filtered.[|/*ref*/a|];"#;
    let mut s = Session::new_for_test("goToDefinition_filteringMappedType", content);
    // TODO: f.VerifyBaselineGoToDefinition(t, true, "ref")
}
