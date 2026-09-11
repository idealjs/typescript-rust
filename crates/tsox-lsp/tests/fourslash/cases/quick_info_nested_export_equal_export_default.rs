use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_nested_export_equal_export_default() {
    let content = r#"export = (state, messages) => {
   export/*1*/ default/*2*/ {
   }
}"#;
    let mut s = Session::new_for_test("quickInfoNestedExportEqualExportDefault", content);
    // TODO: f.VerifyBaselineHover(t)
}
