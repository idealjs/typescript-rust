use tsox_lsp::fourslash::{self, Session};


#[test]
fn document_symbol_private_name() {
    let content = r#"// @Filename: first.ts
class A {
  #foo() {
    class B {
      #bar() {   
         function baz () {
         }
      }
    }
  }
}

class B {
	constructor(private prop: string) {}
}

// @Filename: second.ts
class Foo {
	#privateProp: string;
}
"#;
    let mut s = Session::new_for_test("documentSymbolPrivateName", content);
    // TODO: f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_file(&mut s, "second.ts");
    // TODO: f.VerifyBaselineDocumentSymbol(t)
}
