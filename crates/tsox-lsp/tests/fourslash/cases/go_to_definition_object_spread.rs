use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_object_spread() {
    let content = r#"interface A1 { /*1*/a: number };
interface A2 { /*2*/a?: number };
let a1: A1;
let a2: A2;
let a12 = { ...a1, ...a2 };
a12.[|a/*3*/|];"#;
    let mut s = Session::new_for_test("goToDefinitionObjectSpread", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "3")
}
