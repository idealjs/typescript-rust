use tsox_lsp::fourslash::{self, Session};

#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_meta_property() {
    let content = r#"// @Filename: /a.ts
im/*1*/port.met/*2*/a;
function /*functionDefinition*/f() { n/*3*/ew.[|t/*4*/arget|]; }
// @Filename: /b.ts
im/*5*/port.m;
class /*classDefinition*/c { constructor() { n/*6*/ew.[|t/*7*/arget|]; } }"#;
    let mut s = Session::new(content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1", "2", "3", "4", "5", "6", "7")
}
