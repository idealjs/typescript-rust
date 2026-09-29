use tsox_lsp::fourslash::Session;


#[test]
fn navigation_bar_private_name() {
    let content = r#"class A {
  #foo: () => {
    class B {
      #bar: () => {   
         function baz () {
         }
      }
    }
  }
}"#;
    let _s = Session::new_for_test("navigationBarPrivateName", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
