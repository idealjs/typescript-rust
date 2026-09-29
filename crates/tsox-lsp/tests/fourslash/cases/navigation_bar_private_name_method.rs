use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_private_name_method() {
    let content = r#"class A {
  #foo() {
    class B {
      #bar() {
         function baz () {
         }
      }
    }
  }
}"#;
    let _s = Session::new_for_test("navigationBarPrivateNameMethod", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
