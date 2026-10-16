use tsox_lsp::fourslash::{self, Session};


#[test]
fn recursive_class_reference() {
    let content = r#"declare namespace Thing { }

namespace Thing {
   var /**/x: Mode;
}

namespace Thing {
  export class Mode { }
}"#;
    let mut s = Session::new_for_test("recursiveClassReference", content);
    fourslash::go_to_marker(&mut s, "");
    // TODO: f.VerifyQuickInfoExists(t)
}
