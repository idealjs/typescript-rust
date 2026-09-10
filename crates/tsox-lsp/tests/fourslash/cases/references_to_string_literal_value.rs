use tsox_lsp::fourslash::{self, Session};


#[ignore = "generator: f.MarkTestAsStradaServer()"]
#[test]
fn references_to_string_literal_value() {
    let content = r#"// @lib: es5
const s: string = "some /*1*/ string";"#;
    let mut s = Session::new_for_test("referencesToStringLiteralValue", content);
    // TODO: f.MarkTestAsStradaServer()
    fourslash::unsupported("VerifyBaselineFindAllReferences"); // f.VerifyBaselineFindAllReferences(t, "1")
}
