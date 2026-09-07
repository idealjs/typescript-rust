use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_js_doc_import_tag2() {
    let content = r#"// @allowJS: true
// @checkJs: true
// @Filename: /b.ts
/*2*/export interface A { }
// @Filename: /a.js
/**
 * @import { A } [|from/*1*/|]       "./b"
 */"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
