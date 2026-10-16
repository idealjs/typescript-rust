use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_class_static_block() {
    let content = r#"class C {
  static {
    let x;
  }
}"#;
    let _s = Session::new_for_test("navigationBarClassStaticBlock", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
