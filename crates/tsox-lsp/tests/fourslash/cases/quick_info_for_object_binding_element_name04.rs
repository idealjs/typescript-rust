use tsox_lsp::fourslash::{self, Session};


#[test]
fn quick_info_for_object_binding_element_name04() {
    let content = r#"interface Options {
   /**
    * A description of 'a'
    */
    a: {
       /**
        * A description of 'b'
        */
       b: string;
   }
}

function f({ a, a: { b } }: Options) {
    a/*1*/;
    b/*2*/;
}"#;
    let mut s = Session::new_for_test("quickInfoForObjectBindingElementName04", content);
    // TODO: f.VerifyBaselineHover(t)
}
