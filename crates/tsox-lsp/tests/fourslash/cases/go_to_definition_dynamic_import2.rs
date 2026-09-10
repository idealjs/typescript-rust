use tsox_lsp::fourslash::{self, Session};


#[ignore = "unimplemented: fourslash.VerifyBaselineGoToDefinition"]
#[test]
fn go_to_definition_dynamic_import2() {
    let content = r#"// @Filename: foo.ts
export function /*Destination*/bar() { return "bar"; }
var x = import("./foo");
x.then(foo => {
    foo.[|b/*1*/ar|](); 
})"#;
    let mut s = Session::new_for_test("goToDefinitionDynamicImport2", content);
    fourslash::unsupported("VerifyBaselineGoToDefinition"); // f.VerifyBaselineGoToDefinition(t, true, "1")
}
