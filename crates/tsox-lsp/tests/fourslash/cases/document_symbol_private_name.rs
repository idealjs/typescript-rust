use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineDocumentSymbol"]
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
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
    fourslash::go_to_file(&mut s, "second.ts");
    fourslash::unsupported("VerifyBaselineDocumentSymbol"); // f.VerifyBaselineDocumentSymbol(t)
}
