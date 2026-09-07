use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyNoErrors"]
#[test]
fn get_semantic_diagnostic_for_no_declaration() {
    let content = r#"// @module: CommonJS
interface privateInterface {}
export class Bar implements /*1*/privateInterface/*2*/{ }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyNoErrors"); // f.VerifyNoErrors(t)
}
