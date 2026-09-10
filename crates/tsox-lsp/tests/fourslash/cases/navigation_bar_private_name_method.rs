use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
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
    let mut s = Session::new_for_test("navigationBarPrivateNameMethod", content);
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
